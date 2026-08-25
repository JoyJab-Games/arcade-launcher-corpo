//! Tracks whether a launched game is still running and notifies once it
//! exits — the hook the front-end needs to know when to leave its "game is
//! running" screen and return to the library (see arcade_gdext's
//! `poll_game_exited` and godot/game_session/game_running_screen.gd, which
//! polls it once per frame while active). Also relays the "open the
//! in-game overview" request from `input_watch`'s evdev listener, for the
//! same reason: the front-end can't hear about it any other way while a
//! game holds gamescope's input focus (see `poll_overview_requested`).
//! Deliberately separate from both `launch` (spawning the process) and
//! `gamescope` (compositor-level focus) — this is purely "does the
//! front-end need to react to something", nothing about *how* the game
//! runs, is displayed, or is controlled.
use std::process::Command;
use std::sync::mpsc::{self, Receiver, SyncSender};
use std::sync::{Mutex, OnceLock};

static CHANNEL: OnceLock<(SyncSender<()>, Mutex<Receiver<()>>)> = OnceLock::new();
static OVERVIEW_CHANNEL: OnceLock<(SyncSender<()>, Mutex<Receiver<()>>)> = OnceLock::new();
static CURRENT_PID: Mutex<Option<u32>> = Mutex::new(None);

fn channel() -> &'static (SyncSender<()>, Mutex<Receiver<()>>) {
    CHANNEL.get_or_init(|| {
        let (tx, rx) = mpsc::sync_channel(1);
        (tx, Mutex::new(rx))
    })
}

fn overview_channel() -> &'static (SyncSender<()>, Mutex<Receiver<()>>) {
    OVERVIEW_CHANNEL.get_or_init(|| {
        let (tx, rx) = mpsc::sync_channel(1);
        (tx, Mutex::new(rx))
    })
}

/// Called once a launched game's process has exited — see
/// `gamescope::spawn_monitor_and_focus`, the only caller, which already
/// waits on the child for its own compositor-focus-handback purposes and
/// just reports the same fact here too.
pub(crate) fn notify_game_exited() {
    let (tx, _) = channel();
    // A full channel here would mean a second exit fired before anyone
    // polled the first one - structurally shouldn't happen (a new launch
    // can't start until the front-end has already reacted to the previous
    // exit), but try_send rather than send either way: never block the
    // monitor thread over this, a dropped duplicate notification is
    // harmless.
    let _ = tx.try_send(());
}

/// True exactly once per game exit — meant to be polled regularly (e.g.
/// from Godot's `_process()`) rather than blocked on. False the rest of
/// the time, including "no game has ever run yet" and "a game is still
/// running".
pub fn poll_game_exited() -> bool {
    let (_, rx) = channel();
    rx.lock().expect("exit channel mutex shouldn't be poisoned").try_recv().is_ok()
}

/// Records which process is the currently running game (or clears it,
/// `None`) — see `gamescope::spawn_monitor_and_focus`, the only caller,
/// which already tracks the child's lifetime for its own focus-handback
/// purposes. Exists so `stop_game` has something to act on without
/// needing ownership of the actual `Child` (which
/// `spawn_monitor_and_focus`'s own thread already owns, to `wait()` on).
pub(crate) fn set_current_pid(pid: Option<u32>) {
    *CURRENT_PID.lock().expect("current-pid mutex shouldn't be poisoned") = pid;
}

/// The PID of whatever game is currently tracked as running, if any — see
/// `set_current_pid`. Exists for `gamescope::enter_overlay`/`exit_overlay`,
/// which need to find the running game's window(s) without gamescope.rs
/// having to separately track that itself.
pub(crate) fn current_pid() -> Option<u32> {
    *CURRENT_PID.lock().expect("current-pid mutex shouldn't be poisoned")
}

/// Called by `input_watch` whenever it sees the overview button (gamepad
/// Guide / keyboard F1) pressed on a raw evdev device. Only meaningful
/// while a game is actually running — gated on `CURRENT_PID` rather than
/// firing unconditionally — because with no game running, this launcher's
/// own window already has normal input focus and Godot's `ui_overview`
/// input action handles the same button directly; forwarding it here too
/// would just open the overview a second, redundant way.
pub(crate) fn notify_overview_requested() {
    if CURRENT_PID.lock().expect("current-pid mutex shouldn't be poisoned").is_none() {
        return;
    }
    let (tx, _) = overview_channel();
    // Same reasoning as `notify_game_exited`: try_send, never block the
    // evdev-reading thread over this, and a dropped duplicate (someone
    // mashing the button before anyone's polled) is harmless.
    let _ = tx.try_send(());
}

/// True exactly once per overview request — meant to be polled regularly
/// (e.g. from `GameRunningScreen`'s `_process()`) while a game is running.
/// See `notify_overview_requested` for what feeds this.
pub fn poll_overview_requested() -> bool {
    let (_, rx) = overview_channel();
    rx.lock().expect("overview channel mutex shouldn't be poisoned").try_recv().is_ok()
}

/// Asks the currently running game to quit — SIGTERM, not SIGKILL, so it
/// gets a chance to save state, same as closing it normally would. False
/// if there's no game running right now (nothing to stop) or the signal
/// couldn't be sent; either way, `poll_game_exited` is still what tells
/// the front-end the game has actually gone, this only asks.
pub fn stop_game() -> bool {
    let Some(pid) = *CURRENT_PID.lock().expect("current-pid mutex shouldn't be poisoned") else {
        return false;
    };
    Command::new("kill").arg("-TERM").arg(pid.to_string()).status().is_ok_and(|status| status.success())
}

#[cfg(test)]
mod tests {
    use super::*;

    // One test, not three: `poll_game_exited`/`notify_game_exited` share a
    // single process-global channel by design (every caller across the
    // whole binary needs to reach the same one), which means separate
    // #[test] fns here would race each other under Cargo's default
    // parallel test execution. Keeping every assertion sequential in one
    // test sidesteps that entirely.
    #[test]
    fn poll_reflects_exactly_one_pending_notification_at_a_time() {
        // Drain first so this test doesn't depend on run order relative
        // to anything else that happens to touch this same channel.
        while poll_game_exited() {}

        assert!(!poll_game_exited(), "false with nothing notified yet");

        notify_game_exited();
        assert!(poll_game_exited(), "true exactly once after a notification");
        assert!(!poll_game_exited(), "false again immediately after consuming it");

        notify_game_exited();
        notify_game_exited();
        assert!(poll_game_exited(), "a second notification before polling is dropped, not queued");
        assert!(!poll_game_exited());
    }

    // Also one test, not two: `stop_game` and `notify_overview_requested`
    // both read/write the same shared `CURRENT_PID` global, so a separate
    // #[test] fn for each would race it the same way split-up channel
    // tests would (see above).
    #[test]
    fn current_pid_state_gates_stop_game_and_overview_requests() {
        // Drain first so this doesn't depend on run order relative to
        // anything else that happens to touch this same channel.
        while poll_overview_requested() {}

        set_current_pid(None);
        notify_overview_requested();
        assert!(!poll_overview_requested(), "no game running - request should be dropped, not queued");

        let mut child = Command::new("sleep").arg("30").spawn().expect("'sleep' should exist on any Unix system");
        set_current_pid(Some(child.id()));

        notify_overview_requested();
        assert!(poll_overview_requested(), "a game is running - request should be queued");
        assert!(!poll_overview_requested(), "false again immediately after consuming it");

        assert!(stop_game(), "should report success sending SIGTERM to a real running process");

        let status = child.wait().expect("child should exit once signalled");
        assert!(!status.success(), "SIGTERM should end it, not a clean exit");

        set_current_pid(None);
        assert!(!stop_game(), "false once nothing is tracked as running");
    }
}
