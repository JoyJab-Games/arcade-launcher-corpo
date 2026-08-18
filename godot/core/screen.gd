## Base class every screen managed by ScreenRouter extends — mark the node
## that should get initial focus as "Access as Unique Name" -> %InitialFocus
## (optional — screens with nothing focusable can skip it), override
## enter()/exit() (calling super()) for anything else.
##
## GameAction.BACK (B, red — always means the same thing everywhere) is
## handled here by default: it emits close_requested, whether or not this
## screen shows a Back hint pill for it. A screen with different back
## semantics (e.g. the password screen, where a tap deletes a character and
## only a 700ms hold cancels) overrides _unhandled_input itself instead of
## relying on this default.
##
## Node-typed @export vars don't reliably resolve when hand-authoring .tscn
## and were flaky in testing even via normal scene setup — %UniqueName +
## @onready is the reliable pattern here, use it for any other node
## references a screen needs too.
class_name Screen
extends Control

## Node that receives focus the first time this screen is entered. On later
## re-entries, ScreenRouter restores whatever was last focused instead.
## Subclasses may also assign this directly at runtime (e.g.
## GameSelectionScreen picks the first rebuilt card).
@onready var initial_focus: Control = get_node_or_null("%InitialFocus") as Control

## Emitted by the screen itself (e.g. on a Back/B-button action) to ask
## ScreenRouter to leave this screen. The screen does not pop itself.
signal close_requested

var _last_focus: Control


## Called by ScreenRouter when this screen becomes the active (topmost,
## visible, input-enabled) screen. Override to refresh content on entry;
## call super() to keep the focus-restore behavior.
func enter(_context: Dictionary = {}) -> void:
	if is_instance_valid(_last_focus):
		_last_focus.grab_focus()
	elif is_instance_valid(initial_focus):
		initial_focus.grab_focus()


## Called by ScreenRouter right before this screen is hidden/deactivated.
## Override for cleanup; call super() to keep the focus-restore behavior.
func exit() -> void:
	_last_focus = get_viewport().gui_get_focus_owner()


func _unhandled_input(event: InputEvent) -> void:
	if event.is_action_pressed(GameAction.BACK):
		get_viewport().set_input_as_handled()
		close_requested.emit()
