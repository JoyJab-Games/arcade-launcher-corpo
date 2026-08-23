## Autoload "GameRoster". Thin GDScript wrapper around the native
## GameLibraryBridge (rust/gdext/src/game_library.rs), which itself is a
## thin wrapper around arcade_core::GameStore — the actual "what's
## installed/released" logic lives there, once only, shared with the
## `arcade` CLI (see arcade-launcher-admin-cli). This file exists only so
## the rest of the Godot side has a stable autoload name to call, same as
## before this was wired up to real data.
extends Node

var _bridge: GameLibraryBridge = GameLibraryBridge.new()


func get_released_games() -> Array[Dictionary]:
	return _bridge.get_released_games()


## Starts `name`'s process (see arcade_core::launch). False means it
## couldn't be started (unknown name, no executable resolved, a Proton
## game — not implemented yet) — the bridge already logged why via Godot's
## own error reporting, so callers only need to react, not explain.
func launch_game(name: String) -> bool:
	return _bridge.launch_game(name)
