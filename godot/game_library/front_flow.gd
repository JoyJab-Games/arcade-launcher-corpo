## Shared decision logic for what to show at the "front" of the app: called
## once after Boot's window elapses, and (once game-exit handling exists —
## Phase 1) every time a running game exits back to the launcher. Not a
## Screen itself, just routing logic, so both call sites stay in sync.
class_name FrontFlow
extends RefCounted


static func resolve(needs_help_scene: PackedScene, selection_scene: PackedScene) -> void:
	var released := GameRoster.get_released_games()
	if released.is_empty():
		ScreenRouter.replace(needs_help_scene)
	elif released.size() == 1:
		launch(released[0])
	else:
		ScreenRouter.replace(selection_scene)


# TODO(Phase 1): real launch path (native binary / umu-run + Proton-GE),
# process monitoring, crash auto-restart. Currently just logs, so the
# decision flow (0 / 1 / 2+ released games) is already testable end-to-end.
# Called both from resolve() (cold-boot skip when exactly one game is
# released) and from GameSelectionScreen when a player activates a card.
static func launch(game: Dictionary) -> void:
	print("FrontFlow: would launch ", game.get("name", game))
