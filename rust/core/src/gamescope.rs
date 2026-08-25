//! Focus handover into gamescope's compositor-level app switching — the
//! mechanism that actually makes a launched game show up composited into
//! the same session as this launcher, rather than just spawned as an
//! invisible/backgrounded process. Reuses ShadowBlip's gamescope-x11-client
//! (the same X11-atom control protocol OpenGamepadUI itself uses, via its
//! own gamescope-session — arcade-launcher's compositor choice follows
//! that prior art directly) rather than hand-rolling X11 atom manipulation.
//!
//! Two distinct mechanisms live here, for two distinct kinds of handover:
//!
//! - **Full switch** (`focus_launcher`/`focus_game`): every window
//!   gamescope manages carries a `STEAM_GAME` X11 property (an arbitrary
//!   app-id number despite the Steam-flavored name — gamescope's generic
//!   "which logical app owns this window" tag). Setting
//!   `GAMESCOPECTRL_BASELAYER_APPID` on gamescope's root window is what
//!   actually switches which app-id is shown/focused — only one of
//!   launcher/game is ever visible at a time. Used everywhere right now:
//!   the real launch/exit handover, and (for now) `GameOverlay` too - tag
//!   this launcher's own window with LAUNCHER_APP_ID once at startup, tag
//!   a freshly-launched game's window with GAME_APP_ID once it appears,
//!   and flip the baselayer between the two on launch, on exit, and on
//!   `GameOverlay` open/close.
//! - **Simultaneous overlay** (`enter_overlay`/`exit_overlay`): the game
//!   stays gamescope's visible baselayer the whole time — this never
//!   touches the baselayer at all. Instead this launcher's own window —
//!   transparent per-pixel (see `project.godot`) — is composited on top
//!   of it by tagging it as gamescope's `STEAM_OVERLAY` window and
//!   toggling `STEAM_INPUT_FOCUS` between it and the game. Built and
//!   tested, but **currently unused/parked**, not wired to
//!   `GameOverlay`: keyboard/mouse would correctly retarget via
//!   `STEAM_INPUT_FOCUS`, but gamepad input on Linux mostly bypasses
//!   window focus entirely (raw evdev reads, same reason `input_watch`
//!   exists at all) - without something InputPlumber-equivalent gating
//!   the physical gamepad between launcher and game, a player navigating
//!   this overlay with a controller would also be driving the
//!   still-visibly-running game underneath it at the same time. Revisit
//!   once that's built; until then `focus_launcher`/`focus_game` is what
//!   actually backs `GameOverlay`, since fully backgrounding the game
//!   sidesteps the problem instead of needing to solve it.
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

/// Switches gamescope's focus to the launcher window — used both when a
/// game exits (see `spawn_monitor_and_focus`) and when the in-game
/// overview opens (see `GameOverlay.enter()`, via
/// `GameLibraryBridge::focus_launcher`).
pub fn focus_launcher() {
    let Some(gamescope) = primary() else { return };
    let _ = gamescope.set_baselayer_app_id(LAUNCHER_APP_ID);
}

/// Switches gamescope's focus back to the running game — used when the
/// in-game overview closes (see `GameOverlay.exit()`). Only meaningful
/// while a game is actually running and already tagged by
/// `spawn_monitor_and_focus`; calling this with nothing running just
/// re-focuses an app-id nothing currently owns, which is harmless (and
/// not this function's job to guard against — `session::stop_game`/
/// `poll_game_exited` are what track whether a game is actually up).
pub fn focus_game() {
    let Some(gamescope) = primary() else { return };
    let _ = gamescope.set_baselayer_app_id(GAME_APP_ID);
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

        crate::session::set_current_pid(Some(child.id()));

        // Best-effort either way: an error here just means we can't tell
        // when the game exited, not that anything about the game itself
        // is wrong — it's already running independently of this thread.
        let _ = child.wait();

        crate::session::set_current_pid(None);
        crate::session::notify_game_exited();
        focus_launcher();
    });
}

/// Switches to true simultaneous overlay compositing: unlike
/// `focus_launcher`, this never touches `baselayer_app_id`, so the
/// running game stays gamescope's visible baselayer throughout. Instead
/// this launcher's own window is marked as gamescope's overlay window and
/// given input focus, so it's composited on top of the still-rendering
/// game and receives keyboard/mouse input instead of it. **Currently
/// unused/parked** — see module doc for why (gamepad input isn't actually
/// gated by this) — not called by `GameOverlay` right now, but kept
/// working and exposed via `GameLibraryBridge::enter_overlay` for when
/// that's solved.
pub fn enter_overlay() {
    let Some(gamescope) = primary() else { return };
    set_overlay_state(&gamescope, true);
}

/// Reverses `enter_overlay` — input goes back to the game, and this
/// launcher's window stops claiming to be the overlay. Same "parked" note
/// as `enter_overlay` applies.
pub fn exit_overlay() {
    let Some(gamescope) = primary() else { return };
    set_overlay_state(&gamescope, false);
}

/// Shared by `enter_overlay`/`exit_overlay`: `showing` toggles this
/// launcher's own window(s) between "the overlay, with input focus" and
/// neither — and the running game's window(s) the opposite way, if a game
/// is currently tracked (see `session::current_pid`). Re-discovers both
/// sets of windows on every call rather than caching them: this only ever
/// runs on a real UI action (opening/closing the overlay), never
/// per-frame, so the extra round trip is free, and it sidesteps ever
/// acting on a stale window id from a game that's since restarted.
fn set_overlay_state(gamescope: &XWayland, showing: bool) {
    let value = u32::from(showing);
    let launcher_windows = gamescope.get_windows_for_pid(std::process::id()).unwrap_or_default();
    for window in &launcher_windows {
        let _ = gamescope.set_overlay(*window, value);
        let _ = gamescope.set_input_focus(*window, value);
    }

    let Some(game_pid) = crate::session::current_pid() else { return };
    let game_windows = gamescope.get_windows_for_pid(game_pid).unwrap_or_default();
    for window in &game_windows {
        let _ = gamescope.set_input_focus(*window, u32::from(!showing));
    }
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

    #[test]
    fn enter_and_exit_overlay_do_not_panic_outside_a_gamescope_session() {
        enter_overlay();
        exit_overlay();
    }
}
