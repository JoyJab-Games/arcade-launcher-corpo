//! Thin bridge for the game library / front-end flow — mirrors the CLI
//! (`rust/cli/src/commands`) in that respect: all the real logic (what's
//! released, where a game's files/art live, how to launch it) is
//! `arcade_core`'s job. This file only exists to convert between that and
//! Godot's Variant/VarDictionary/Array types.
use godot::prelude::*;

use arcade_core::{GameConfig, GameStore};

/// Backs the GDScript "GameRoster" autoload (`godot/game_library/game_roster.gd`),
/// which just delegates straight through to this.
#[derive(GodotClass)]
#[class(base=RefCounted)]
pub struct GameLibraryBridge {
    base: Base<RefCounted>,
}

#[godot_api]
impl IRefCounted for GameLibraryBridge {
    fn init(base: Base<RefCounted>) -> Self {
        // Tags this launcher's own window as gamescope's base/launcher app
        // (see arcade_core::gamescope) - once, here, since GameRoster's
        // autoload construction (`GameLibraryBridge.new()`) only ever runs
        // once per session. No-op outside a real gamescope session.
        arcade_core::gamescope::spawn_tag_self_as_launcher();
        // Starts the evdev overview-button watcher (see
        // arcade_core::input_watch) - same "once per session" reasoning.
        // Harmless if no matching device is present (e.g. a dev machine
        // with no gamepad plugged in).
        arcade_core::input_watch::spawn_watch();
        Self { base }
    }
}

#[godot_api]
impl GameLibraryBridge {
    /// Every game currently released for players, as Dictionaries — see
    /// `game_to_dictionary` for the shape.
    #[func]
    fn get_released_games(&self) -> Array<VarDictionary> {
        let store = GameStore::open(GameStore::default_root());
        store
            .list_released()
            .unwrap_or_default()
            .iter()
            .map(game_to_dictionary)
            .collect()
    }

    /// Spawns `name`'s process (see `arcade_core::launch`). Returns false —
    /// and logs why, via Godot's own error reporting rather than a silent
    /// failure — if it couldn't be started (unknown name, no executable
    /// resolved, or its pinned Proton build isn't downloaded yet).
    #[func]
    fn launch_game(&self, name: GString) -> bool {
        let name = name.to_string();
        let store = GameStore::open(GameStore::default_root());
        let Some(game) = store.get(&name) else {
            godot_error!("GameLibraryBridge: no game named '{name}'");
            return false;
        };

        match arcade_core::launch(&game) {
            Ok(()) => true,
            Err(e) => {
                godot_error!("GameLibraryBridge: couldn't launch '{name}': {e}");
                false
            }
        }
    }

    /// True exactly once per game exit (see `arcade_core::session`) — meant
    /// to be polled regularly (e.g. from a Screen's `_process()`) while a
    /// "game is running" screen is active, not called just once.
    #[func]
    fn poll_game_exited(&self) -> bool {
        arcade_core::session::poll_game_exited()
    }

    /// True exactly once per "open the in-game overview" request detected
    /// by the evdev button watcher (see `arcade_core::input_watch`) while
    /// a game is running and holds normal input focus away from this
    /// launcher's own window. Meant to be polled regularly (e.g. from
    /// `GameRunningScreen`'s `_process()`), same shape as
    /// `poll_game_exited`.
    #[func]
    fn poll_overview_requested(&self) -> bool {
        arcade_core::session::poll_overview_requested()
    }

    /// Switches gamescope's compositor focus to the launcher (see
    /// `arcade_core::gamescope::focus_launcher`) — called when the in-game
    /// overlay opens, so it's actually visible over the running game.
    #[func]
    fn focus_launcher(&self) {
        arcade_core::gamescope::focus_launcher();
    }

    /// Switches gamescope's compositor focus back to the running game (see
    /// `arcade_core::gamescope::focus_game`) — called when the in-game
    /// overlay closes.
    #[func]
    fn focus_game(&self) {
        arcade_core::gamescope::focus_game();
    }

    /// Switches to true simultaneous overlay compositing (see
    /// `arcade_core::gamescope::enter_overlay`) — unlike `focus_launcher`,
    /// the running game stays visible the whole time; this just puts the
    /// launcher's own (transparent) window on top of it and gives it
    /// input focus. **Currently unused/parked**, not called by
    /// `GameOverlay` — see `arcade_core::gamescope`'s module doc for why
    /// (gamepad input isn't actually gated by this).
    #[func]
    fn enter_overlay(&self) {
        arcade_core::gamescope::enter_overlay();
    }

    /// Reverses `enter_overlay` (see `arcade_core::gamescope::exit_overlay`).
    /// Same "parked" note as `enter_overlay` applies.
    #[func]
    fn exit_overlay(&self) {
        arcade_core::gamescope::exit_overlay();
    }

    /// Asks the currently running game to quit (see
    /// `arcade_core::session::stop_game`) — false if none is running.
    /// `poll_game_exited` is still what confirms it actually has.
    #[func]
    fn stop_game(&self) -> bool {
        arcade_core::session::stop_game()
    }

    /// Powers the device off (see `arcade_core::power::shutdown`).
    #[func]
    fn shutdown(&self) -> bool {
        match arcade_core::power::shutdown() {
            Ok(()) => true,
            Err(e) => {
                godot_error!("GameLibraryBridge: couldn't shut down: {e}");
                false
            }
        }
    }

    /// Reboots the device (see `arcade_core::power::reboot`).
    #[func]
    fn reboot(&self) -> bool {
        match arcade_core::power::reboot() {
            Ok(()) => true,
            Err(e) => {
                godot_error!("GameLibraryBridge: couldn't reboot: {e}");
                false
            }
        }
    }
}

fn game_to_dictionary(game: &GameConfig) -> VarDictionary {
    let mut dict = VarDictionary::new();
    dict.set("name", game.name.as_str());
    dict.set("source", game.source.as_str());
    dict.set("source_ref", game.source_ref.as_str());
    dict.set("description", game.description.as_deref().unwrap_or_default());
    dict.set("tags", game.tags.iter().map(|tag| GString::from(tag.as_str())).collect::<PackedStringArray>());
    if let Some(path) = game.image_abs_path() {
        dict.set("image_path", path.to_string_lossy().to_string());
    }
    dict
}
