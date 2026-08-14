## Autoload singleton ("ScreenRouter"). Owns a stack of instantiated Screen
## nodes; only the top of the stack is visible/active. Screens are never
## destroyed just for being hidden — they stay alive underneath so state
## (scroll position, list contents, ...) survives navigating back to them.
extends Node

## Emitted whenever the active (topmost) screen changes, e.g. so a status/
## button-hint bar can react to what's currently on screen.
signal screen_changed(screen: Screen)

var _stack: Array[Screen] = []
var _host: Control


func _ready() -> void:
	_host = Control.new()
	_host.name = "ScreenHost"
	_host.set_anchors_preset(Control.PRESET_FULL_RECT)
	_host.mouse_filter = Control.MOUSE_FILTER_IGNORE
	add_child(_host)


## Returns the currently active screen, or null if nothing has been pushed yet.
func current() -> Screen:
	return _stack.back() if not _stack.is_empty() else null


## Pushes a new screen on top of the stack. The previous top is deactivated
## (hidden, input disabled) but stays alive underneath.
func push(screen_scene: PackedScene, context: Dictionary = {}) -> Screen:
	if not _stack.is_empty():
		_deactivate(_stack.back())
	return _push_new(screen_scene, context)


## Removes the current top screen and reactivates the one beneath it.
## Refuses to pop the last screen — there must always be one on the stack.
func pop() -> void:
	if _stack.size() <= 1:
		push_warning("ScreenRouter.pop() called with only one screen on the stack — ignored.")
		return

	var top: Screen = _stack.pop_back()
	top.exit()
	top.queue_free()

	_activate(_stack.back(), {})


## Pops the current top and pushes a new screen in its place — for
## transitions with no back-history (e.g. Autostart -> running game).
func replace(screen_scene: PackedScene, context: Dictionary = {}) -> Screen:
	if not _stack.is_empty():
		var old_top: Screen = _stack.pop_back()
		old_top.exit()
		old_top.queue_free()
	return _push_new(screen_scene, context)


## Pops screens until only the base (first-pushed) screen remains, e.g. when
## returning to Home after a game exits. Intermediate screens are freed
## directly, without briefly reactivating each one.
func pop_to_root() -> void:
	if _stack.size() <= 1:
		return
	while _stack.size() > 1:
		var top: Screen = _stack.pop_back()
		top.exit()
		top.queue_free()
	_activate(_stack.back(), {})


func _push_new(screen_scene: PackedScene, context: Dictionary) -> Screen:
	var screen := screen_scene.instantiate() as Screen
	assert(screen != null, "ScreenRouter: scene root must extend Screen")
	_host.add_child(screen)
	screen.close_requested.connect(pop)
	_stack.push_back(screen)
	_activate(screen, context)
	return screen


func _activate(screen: Screen, context: Dictionary) -> void:
	screen.visible = true
	screen.process_mode = Node.PROCESS_MODE_INHERIT
	screen.enter(context)
	screen_changed.emit(screen)


func _deactivate(screen: Screen) -> void:
	screen.exit()
	screen.visible = false
	screen.process_mode = Node.PROCESS_MODE_DISABLED
