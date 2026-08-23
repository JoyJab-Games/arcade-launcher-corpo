## The ~2 second black screen shown at cold start. Gives a brief window to
## reach WiFi setup via a single short Guide-button press before falling
## through to FrontFlow.resolve(). Jesco's scene just needs to extend this
## and be black — no other content required.
##
## Admin tasks (install/remove/release games, etc.) moved to the `arcade`
## CLI over SSH — there's no in-app admin menu/password anymore. The only
## thing that still needs on-device UI is WiFi setup (has to work before
## SSH access is even possible). The secret shortcut (both Guide buttons
## held 3s elsewhere, a single short press here during boot) now leads
## straight to WiFi setup, no password.
class_name BootScreen
extends Screen

## How long to wait before falling through to FrontFlow.resolve().
@export var boot_duration_sec: float = 2.0

## Scenes FrontFlow routes to once the boot window elapses.
@export var needs_help_scene: PackedScene
@export var selection_scene: PackedScene
@export var game_running_scene: PackedScene

var _timer: Timer


func enter(_context: Dictionary = {}) -> void:
	print("boot window elapsed")
	super.enter(_context)
	_timer = Timer.new()
	_timer.one_shot = true
	_timer.wait_time = boot_duration_sec
	_timer.timeout.connect(_on_boot_window_elapsed)
	add_child(_timer)
	_timer.start()


# TODO(Phase 3): call this from the Rust evdev Guide-button listener when a
# single short press happens while Boot is on top of the stack. Needs the
# WifiSetupScreen to exist first.
func _on_admin_shortcut_during_boot() -> void:
	if _timer:
		_timer.stop()
	# push(wifi_setup_scene) once WifiSetupScreen exists.


func _on_boot_window_elapsed() -> void:
	print("boot window elapsed")
	FrontFlow.resolve(needs_help_scene, selection_scene, game_running_scene)
