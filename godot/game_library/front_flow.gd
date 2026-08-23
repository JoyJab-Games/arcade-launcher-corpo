## Shared decision logic for what to show at the "front" of the app: called
## once after Boot's window elapses, and (once game-exit handling exists —
## Phase 1) every time a running game exits back to the launcher. Not a
## Screen itself, just routing logic, so both call sites stay in sync.
##
## Also the only place that ties GameRoster (the data source) to
## GameSelectionScreen (a dumb resolver — see its own doc comment) and to
## launch() (what "a game got picked" actually means): fetches the roster,
## hands it to the screen as prepackaged context, and connects the screen's
## game_selected event back to launch().
class_name FrontFlow
extends RefCounted


static func resolve(needs_help_scene: PackedScene, selection_scene: PackedScene) -> void:
	var released := GameRoster.get_released_games()
	if released.is_empty():
		ScreenRouter.replace(needs_help_scene)
	elif released.size() == 1:
		launch(released[0])
	else:
		var screen := ScreenRouter.replace(selection_scene, {"games": released}) as GameSelectionScreen
		screen.game_selected.connect(launch)


# TODO(Phase 1): process monitoring + crash auto-restart, Proton/umu-run
# wrapping (arcade_core::launch already refuses a Proton game outright
# rather than guessing), and the actual on-cabinet focus handover to the
# running game (blocked on the still-open compositor choice — cage vs
# gamescope — per arcade-launcher open questions). Called both directly
# from resolve() (cold-boot skip when exactly one game is released) and via
# the game_selected connection resolve() sets up, once a player activates a
# card on GameSelectionScreen.
static func launch(game: Dictionary) -> void:
	var name: String = game.get("name", "")
	if name.is_empty():
		push_warning("FrontFlow: launch() called with a game that has no name: ", game)
		return
	if not GameRoster.launch_game(name):
		push_warning("FrontFlow: couldn't launch '%s' — see the error above for why." % name)
