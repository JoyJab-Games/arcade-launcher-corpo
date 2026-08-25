//! Focus-independent detection of the "open in-game overview" button
//! (gamepad Guide / keyboard F1) — reads raw evdev events directly,
//! bypassing whatever window currently has gamescope's input focus.
//!
//! The problem this solves: the moment a launched game becomes gamescope's
//! baselayer (see `gamescope::spawn_monitor_and_focus`), this launcher's
//! own window stops receiving keyboard/joypad input entirely — X11/
//! gamescope routes it to the focused (game) window instead. Godot's own
//! `ui_overview` input action (`godot/project.godot`) already covers
//! opening the overview whenever this launcher's window is actually
//! focused; this module exists purely to cover the case where it isn't —
//! a game is running and holds all normal input — by watching the same
//! physical devices directly at the kernel level instead of going through
//! whichever window the compositor currently favors.
//!
//! Deliberately non-exclusive: devices are opened for reading only, never
//! grabbed (`EVIOCGRAB`), so the same button press still also reaches the
//! game/X11 exactly as if this module didn't exist. Guide is
//! conventionally OS/overlay-reserved and essentially never bound by
//! games; F1 is a dev-machine convenience for testing without a real
//! gamepad plugged in and isn't expected to double-fire against anything
//! on the actual cabinet, which has no keyboard in normal play.
//!
//! Best-effort in the sense that a device this process can't open never
//! blocks startup or panics — same stance as `gamescope::primary()`, the
//! front-end degrades fine without this (the button just does nothing
//! while a game holds focus, same as before this module existed). But
//! *silent* failure here specifically has bitten this project before (see
//! `docs/system-requirements.md`): `/dev/input/event*` access requires
//! the `input` group — unlike graphics/audio/USB devices, systemd's
//! seat-ACL mechanism deliberately excludes raw keyboard/gamepad nodes
//! from automatic access, precisely because that access is equivalent to
//! keylogging. A user not in that group produces `EACCES` on every single
//! device, every time, indistinguishable from "no gamepad plugged in"
//! unless something says so out loud - so `spawn_watch` does exactly
//! that, once, via stderr (see `report_permission_issue`).
use std::io::ErrorKind;
use std::path::Path;

use evdev::{Device, EventSummary, KeyCode};

use crate::session;

/// Discovers every currently-present input device that looks like a
/// gamepad (exposes `BTN_MODE`, the Guide button) or a keyboard (exposes
/// `KEY_F1`) and starts watching each on its own background thread. Call
/// once at startup (see `GameLibraryBridge::init`) — never blocks the
/// caller. Controllers plugged in after this call aren't picked up; that's
/// a fine v1 limitation since the cabinet's gamepads are wired in at boot,
/// not hot-plugged mid-session.
pub fn spawn_watch() {
    std::thread::spawn(|| {
        let (watched, permission_denied) = discover_devices();
        if watched.is_empty() && permission_denied > 0 {
            report_permission_issue(permission_denied);
        }
        for device in watched {
            std::thread::spawn(move || watch_device(device));
        }
    });
}

/// Walks `/dev/input` directly (rather than `evdev::enumerate()`, which
/// throws away *why* a device couldn't be opened) so a permission problem
/// can be told apart from "nothing plugged in" — see `spawn_watch`, the
/// only caller, for what it does with that distinction.
fn discover_devices() -> (Vec<Device>, usize) {
    let mut watched = Vec::new();
    let mut permission_denied = 0;

    let Ok(entries) = std::fs::read_dir("/dev/input") else { return (watched, permission_denied) };
    for path in entries.filter_map(|entry| entry.ok()).map(|entry| entry.path()) {
        if !is_event_node(&path) {
            continue;
        }
        match Device::open(&path) {
            Ok(device) if watches_key(&device) => watched.push(device),
            Ok(_) => {}
            Err(e) if e.kind() == ErrorKind::PermissionDenied => permission_denied += 1,
            Err(_) => {}
        }
    }

    (watched, permission_denied)
}

fn is_event_node(path: &Path) -> bool {
    path.file_name().and_then(|name| name.to_str()).is_some_and(|name| name.starts_with("event"))
}

/// True if the device exposes either key this module cares about.
fn watches_key(device: &Device) -> bool {
    device.supported_keys().is_some_and(|keys| keys.contains(KeyCode::BTN_MODE) || keys.contains(KeyCode::KEY_F1))
}

/// Prints a one-time, actionable explanation to stderr instead of leaving
/// this indistinguishable from "no gamepad plugged in" - visible in
/// `nix run .#dev`'s terminal and in the packaged app's systemd journal
/// alike. Deliberately not a hard failure (see module doc): the launcher
/// and any running game both still work fully otherwise, just without the
/// in-game overview shortcut.
fn report_permission_issue(device_count: usize) {
    eprintln!(
        "arcade-launcher: found {device_count} input device(s) under /dev/input but couldn't open \
         any of them (permission denied) - the in-game overview button (Guide/F1) won't work while \
         a game is running. Fix: add this user to the 'input' group and log out/in (group membership \
         is applied at login). See docs/system-requirements.md."
    );
}

/// Reads events from a single device until it disappears (unplugged) or a
/// read error occurs, calling `session::notify_overview_requested` on
/// each qualifying key-down. Blocks its own dedicated thread by design —
/// `fetch_events` blocks until events are available, which is exactly
/// what a thread with nothing else to do should do.
fn watch_device(mut device: Device) {
    loop {
        let events = match device.fetch_events() {
            Ok(events) => events,
            Err(_) => return,
        };
        for event in events {
            // value 1 = key down; 0 = key up, 2 = autorepeat - only the
            // initial press should trigger anything.
            if let EventSummary::Key(_, code, 1) = event.destructure() {
                if code == KeyCode::BTN_MODE || code == KeyCode::KEY_F1 {
                    session::notify_overview_requested();
                }
            }
        }
    }
}
