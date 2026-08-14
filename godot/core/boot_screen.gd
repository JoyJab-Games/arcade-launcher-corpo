## The ~2 second black screen shown at cold start. Gives a brief window to
## reach the admin prompt via a single short Guide-button press before
## falling through to FrontFlow.resolve(). Jesco's scene just needs to
## extend this and be black — no other content required.
class_name BootScreen
extends Screen

## How long to wait before falling through to FrontFlow.resolve().
@export var boot_duration_sec: float = 2.0

## Scenes FrontFlow routes to once the boot window elapses.
@export var needs_help_scene: PackedScene
@export var selection_scene: PackedScene

var _timer: Timer


func enter(_context: Dictionary = {}) -> void:
	super.enter(_context)
	_timer = Timer.new()
	_timer.one_shot = true
	_timer.wait_time = boot_duration_sec
	_timer.timeout.connect(_on_boot_window_elapsed)
	add_child(_timer)
	_timer.start()


# TODO(Phase 2): call this from the Rust evdev Guide-button listener when a
# single short press happens while Boot is on top of the stack. Needs the
# AdminPrompt screen to exist first (also Phase 2).
func _on_admin_shortcut_during_boot() -> void:
	if _timer:
		_timer.stop()
	# push(admin_prompt_scene) once AdminPrompt exists.


func _on_boot_window_elapsed() -> void:
	FrontFlow.resolve(needs_help_scene, selection_scene)
