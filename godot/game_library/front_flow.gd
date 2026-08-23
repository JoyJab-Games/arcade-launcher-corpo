## Shared decision logic for what to show at the "front" of the app: called
## once after Boot's window elapses, and again every time a running game
## exits back to the launcher (see GameRunningScreen, which is what
## actually calls resolve() a second time). Not a Screen itself, just
## routing logic, so both call sites stay in sync.
##
## Also the only place that ties GameRoster (the data source) to
## GameSelectionScreen (a dumb resolver — see its own doc comment) and to
## launch() (what "a game got picked" actually means): fetches the roster,
## hands it to the screen as prepackaged context, and connects the screen's
## game_selected event back to launch().
class_name FrontFlow
extends RefCounted


static func resolve(needs_help_scene: PackedScene, selection_scene: PackedScene, game_running_scene: PackedScene) -> void:
	var released := GameRoster.get_released_games()
	if released.is_empty():
		ScreenRouter.replace(needs_help_scene)
	elif released.size() == 1:
		launch(released[0], needs_help_scene, selection_scene, game_running_scene)
	else:
		var screen := ScreenRouter.replace(selection_scene, {"games": released}) as GameSelectionScreen
		screen.game_selected.connect(launch.bind(needs_help_scene, selection_scene, game_running_scene))


# TODO(Phase 1): crash auto-restart, and Proton/umu-run wrapping edge cases
# (arcade_core::launch already refuses a Proton game whose build isn't
# predownloaded outright rather than guessing). Process monitoring and
# on-cabinet compositor focus handover are both done now - see
# arcade_core::session/arcade_core::gamescope and GameRunningScreen, which
# is what this pushes on a successful launch and what notices the game
# exiting again. Called both directly from resolve() (cold-boot skip when
# exactly one game is released) and via the game_selected connection
# resolve() sets up, once a player activates a card on GameSelectionScreen.
static func launch(
	game: Dictionary,
	needs_help_scene: PackedScene,
	selection_scene: PackedScene,
	game_running_scene: PackedScene,
) -> void:
	var name: String = game.get("name", "")
	if name.is_empty():
		push_warning("FrontFlow: launch() called with a game that has no name: ", game)
		return
	if not GameRoster.launch_game(name):
		push_warning("FrontFlow: couldn't launch '%s' — see the error above for why." % name)
		return

	var screen := ScreenRouter.replace(game_running_scene) as GameRunningScreen
	screen.needs_help_scene = needs_help_scene
	screen.selection_scene = selection_scene
	screen.game_running_scene = game_running_scene
