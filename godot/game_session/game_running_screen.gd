## Shown for as long as a game is running. Blacked out and minimal on
## purpose - once the real compositor integration is running (see
## arcade_core::gamescope), the actual game is what a player sees, not
## this; this is only what's still technically active in ScreenRouter's
## stack underneath it, and what's briefly visible during the handover to
## and back from the game.
##
## Polls GameRoster.poll_game_exited() (see arcade_core::session) once per
## _process() frame while active - cheap enough (a single non-blocking
## channel try_recv on the Rust side) not to need a timer instead, and
## ScreenRouter already stops _process() from running at all once this
## screen isn't the active one (see its _deactivate), so there's no
## wasted polling once a game's actually left this screen. The moment an
## exit is detected, asks FrontFlow to resolve() again, same as Boot does
## on cold start (but never auto-launching straight back into the same
## single game - see resolve()'s auto_launch_single_game - a player who
## just quit shouldn't be confused by it immediately reopening) - so the
## library reflects whatever's actually released now, not whatever it was
## when this game launched.
##
## Also listens for ui_overview (Home) to open GameOverlay on top of the
## game - see GameAction.OVERVIEW's own doc comment ("open/close in-game
## overview").
class_name GameRunningScreen
extends Screen

## Scenes FrontFlow re-resolves through once the running game exits - same
## shape as BootScreen's own needs_help_scene/selection_scene/
## game_running_scene/game_overlay_scene exports (this screen included,
## for the case where resolve() decides to auto-launch straight into
## another game).
@export var needs_help_scene: PackedScene
@export var selection_scene: PackedScene
@export var game_running_scene: PackedScene
@export var game_overlay_scene: PackedScene


func _process(_delta: float) -> void:
	if GameRoster.poll_game_exited():
		FrontFlow.resolve(needs_help_scene, selection_scene, game_running_scene, game_overlay_scene, false)


func _unhandled_input(event: InputEvent) -> void:
	if event.is_action_pressed(GameAction.OVERVIEW):
		get_viewport().set_input_as_handled()
		_open_overlay()
	else:
		super._unhandled_input(event)


func _open_overlay() -> void:
	var overlay := ScreenRouter.push(game_overlay_scene) as GameOverlay
	overlay.needs_help_scene = needs_help_scene
	overlay.selection_scene = selection_scene
	overlay.game_running_scene = game_running_scene
	overlay.game_overlay_scene = game_overlay_scene
