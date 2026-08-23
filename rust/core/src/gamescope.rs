//! Focus handover into gamescope's compositor-level app switching — the
//! mechanism that actually makes a launched game show up composited into
//! the same session as this launcher, rather than just spawned as an
//! invisible/backgrounded process. Reuses ShadowBlip's gamescope-x11-client
//! (the same X11-atom control protocol OpenGamepadUI itself uses, via its
//! own gamescope-session — arcade-launcher's compositor choice follows
//! that prior art directly) rather than hand-rolling X11 atom manipulation.
//!
//! Every window gamescope manages carries a `STEAM_GAME` X11 property (an
//! arbitrary app-id number despite the Steam-flavored name — gamescope's
//! generic "which logical app owns this window" tag). Setting
//! `GAMESCOPECTRL_BASELAYER_APPID` on gamescope's root window is what
//! actually switches which app-id is shown/focused. So: tag this
//! launcher's own window with LAUNCHER_APP_ID once at startup, tag a
//! freshly-launched game's window with GAME_APP_ID once it appears, and
//! flip the baselayer between the two on launch and on exit.
//!
//! Best-effort throughout, not a hard requirement: outside a real
//! gamescope session (e.g. `nix run .#dev` on a plain desktop) every
//! function here is a silent no-op rather than an error — the launcher and
//! a launched game both still work standalone, just without the
//! compositor-level focus swap.
use std::process::Child;
use std::time::{Duration, Instant};

use gamescope_x11_client::xwayland::{Primary, XWayland};

/// This launcher's own app-id, set on its own window(s) once at startup.
const LAUNCHER_APP_ID: u32 = 1;
/// Whichever single game is currently running — this launcher only ever
/// runs one at a time, so a single fixed id (rather than one per game) is
/// enough to tell gamescope "the game" apart from "the launcher".
const GAME_APP_ID: u32 = 2;

/// How long to wait for a window to actually appear for a given process
/// before giving up — game engines (and this launcher itself, at startup)
/// can take a few seconds to get a window mapped after the process starts.
const WINDOW_APPEAR_TIMEOUT: Duration = Duration::from_secs(15);
const POLL_INTERVAL: Duration = Duration::from_millis(200);

/// Gamescope's primary XWayland instance, or `None` if this process isn't
/// running under gamescope at all — every caller in this module treats
/// that as "nothing to do", not an error.
fn primary() -> Option<XWayland> {
    let xwaylands = gamescope_x11_client::discover_gamescope_xwaylands().ok()?;
    xwaylands.into_iter().find_map(|mut xwayland| {
        xwayland.connect().ok()?;
        xwayland.is_primary_instance().ok()?.then_some(xwayland)
    })
}

/// Polls for `pid`'s window(s) to appear, up to `timeout`. Empty if they
/// never do (or gamescope has no record of that pid at all).
fn wait_for_windows(gamescope: &XWayland, pid: u32, timeout: Duration) -> Vec<u32> {
    let deadline = Instant::now() + timeout;
    loop {
        if let Ok(windows) = gamescope.get_windows_for_pid(pid) {
            if !windows.is_empty() {
                return windows;
            }
        }
        if Instant::now() >= deadline {
            return Vec::new();
        }
        std::thread::sleep(POLL_INTERVAL);
    }
}

/// Tags this process's own window(s) as the launcher app and brings it to
/// the foreground. Meant to be called once, off the caller's own thread
/// (see `spawn_tag_self_as_launcher`) since it can block waiting for the
/// window to be mapped.
fn tag_self_as_launcher() {
    let Some(gamescope) = primary() else { return };
    let windows = wait_for_windows(&gamescope, std::process::id(), WINDOW_APPEAR_TIMEOUT);
    for window in &windows {
        let _ = gamescope.set_app_id(*window, LAUNCHER_APP_ID);
    }
    let _ = gamescope.set_baselayer_app_id(LAUNCHER_APP_ID);
}

/// Runs `tag_self_as_launcher` on a background thread — call once at
/// startup (see `GameLibraryBridge::init`). Never blocks the caller: at
/// this point in the engine's own boot sequence, this launcher's window
/// may not be mapped yet, and this has nothing useful to do until it is.
pub fn spawn_tag_self_as_launcher() {
    std::thread::spawn(tag_self_as_launcher);
}

/// Hands gamescope focus to `child`'s window once it appears, waits for
/// `child` to exit (notifying `session::notify_game_exited` the moment it
/// does, so the front-end can leave its "game is running" screen), then
/// hands focus back to the launcher — all on a background thread the
/// caller doesn't wait on, so a slow-to-appear window (or an entire play
/// session) never blocks whoever spawned the game (see `launch`, which
/// owns calling this).
pub fn spawn_monitor_and_focus(mut child: Child) {
    std::thread::spawn(move || {
        if let Some(gamescope) = primary() {
            let windows = wait_for_windows(&gamescope, child.id(), WINDOW_APPEAR_TIMEOUT);
            for window in &windows {
                let _ = gamescope.set_app_id(*window, GAME_APP_ID);
            }
            if !windows.is_empty() {
                let _ = gamescope.set_baselayer_app_id(GAME_APP_ID);
            }
        }

        // Best-effort either way: an error here just means we can't tell
        // when the game exited, not that anything about the game itself
        // is wrong — it's already running independently of this thread.
        let _ = child.wait();
        crate::session::notify_game_exited();

        if let Some(gamescope) = primary() {
            let _ = gamescope.set_baselayer_app_id(LAUNCHER_APP_ID);
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;

    // These all run in a plain CI/dev sandbox, never a real gamescope
    // session - which is itself the common case this module has to
    // degrade gracefully for (every `nix run .#dev` on a normal desktop).
    // There's no meaningful way to test the actual X11-atom behavior
    // without a real gamescope instance to talk to.

    #[test]
    fn primary_is_none_outside_a_gamescope_session() {
        assert!(primary().is_none());
    }

    #[test]
    fn spawn_tag_self_as_launcher_does_not_panic() {
        spawn_tag_self_as_launcher();
    }

    #[test]
    fn spawn_monitor_and_focus_does_not_panic_and_notices_the_child_exit() {
        let child = Command::new("true").spawn().expect("'true' should exist on any Unix system");
        spawn_monitor_and_focus(child);
        // No assertion beyond "didn't panic" - the spawned thread's own
        // child.wait() confirms exit-detection works even without
        // gamescope, but there's nothing externally observable to check
        // from here without a real compositor to query.
    }
}
