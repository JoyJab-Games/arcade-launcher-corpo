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


## Starts `game_name`'s process (see arcade_core::launch). False means it
## couldn't be started (unknown name, no executable resolved, a Proton
## game — not implemented yet) — the bridge already logged why via Godot's
## own error reporting, so callers only need to react, not explain.
## Parameter isn't called `name` - that'd shadow Node's own built-in `name`
## property, since this (like every autoload) extends Node.
func launch_game(game_name: String) -> bool:
	return _bridge.launch_game(game_name)


## True exactly once per game exit (see arcade_core::session) - meant to be
## polled regularly (e.g. from GameRunningScreen's _process()) while a game
## is running, not called just once.
func poll_game_exited() -> bool:
	return _bridge.poll_game_exited()


## Switches gamescope's compositor focus to the launcher, so it's actually
## visible over the running game - see GameOverlay.enter().
func focus_launcher() -> void:
	_bridge.focus_launcher()


## Switches gamescope's compositor focus back to the running game - see
## GameOverlay.exit().
func focus_game() -> void:
	_bridge.focus_game()


## Asks the currently running game to quit. False if none is running -
## poll_game_exited() is still what confirms it actually has.
func stop_game() -> bool:
	return _bridge.stop_game()


func shutdown() -> bool:
	return _bridge.shutdown()


func reboot() -> bool:
	return _bridge.reboot()
