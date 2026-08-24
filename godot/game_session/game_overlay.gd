## Pause-style overlay shown on top of a running game (see
## GameRunningScreen, which pushes this on a ui_overview/Home press).
## Gamescope keeps the game itself running the whole time underneath (see
## arcade_core::gamescope) - enter()/exit() just swap which one gamescope
## actually shows: this launcher while the overlay is open, the game again
## once it closes. GameAction.BACK (B) already closes it for free, via
## Screen's own default handling; ui_overview does too (see
## _unhandled_input), matching its "open/close in-game overview" doc
## comment in GameAction.
##
## Also keeps polling for the game exiting on its own (crashing, or "Spiel
## Beenden" below) while showing - ScreenRouter pauses GameRunningScreen's
## own _process() once this is pushed on top of it, so exit-detection
## would otherwise stop working for as long as the overlay's up.
class_name GameOverlay
extends Screen

## Scenes FrontFlow re-resolves through if the game exits while this is
## showing - same shape as GameRunningScreen's own exports (which set
## these when pushing this scene).
@export var needs_help_scene: PackedScene
@export var selection_scene: PackedScene
@export var game_running_scene: PackedScene
@export var game_overlay_scene: PackedScene

@onready var _quit_row: Control = %QuitButton
@onready var _shutdown_row: Control = %ShutdownButton
@onready var _restart_row: Control = %RestartButton


func _ready() -> void:
	initial_focus = _quit_row
	_quit_row.activated.connect(_on_quit_pressed)
	_shutdown_row.activated.connect(_on_shutdown_pressed)
	_restart_row.activated.connect(_on_restart_pressed)


func enter(context: Dictionary = {}) -> void:
	super.enter(context)
	GameRoster.focus_launcher()


func exit() -> void:
	super.exit()
	GameRoster.focus_game()


func _process(_delta: float) -> void:
	if GameRoster.poll_game_exited():
		FrontFlow.resolve(needs_help_scene, selection_scene, game_running_scene, game_overlay_scene, false)


func _unhandled_input(event: InputEvent) -> void:
	if event.is_action_pressed(GameAction.OVERVIEW):
		get_viewport().set_input_as_handled()
		close_requested.emit()
		return
	super._unhandled_input(event)


func _on_quit_pressed() -> void:
	GameRoster.stop_game()
	# No explicit navigation here - _process()'s poll_game_exited() check
	# picks up the exit and resolves the same way a crash or the player
	# quitting from inside the game itself would.


func _on_shutdown_pressed() -> void:
	GameRoster.shutdown()


func _on_restart_pressed() -> void:
	GameRoster.reboot()
