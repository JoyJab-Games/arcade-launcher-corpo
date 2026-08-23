## Icon+label badge for the bottom hint bar - decoration, not a real UI
## control: no focus, not mouse-clickable, no selection state at all. It
## only mirrors `action` (e.g. GameAction.CONFIRM) being pressed anywhere
## on screen with a small scale-up, holding at the peak for as long as the
## physical button stays down, then easing back on release.
##
## Listens globally via _unhandled_input (same pattern as HoldAction)
## rather than gui_input: this isn't a focus target, so it has to react to
## the actual physical button regardless of what currently holds focus.
##
## No controlling parent needed - just drop instances straight into a
## screen's hint bar and set `action`/`label_text` per instance. `icon`
## defaults to the right colored dot for `action` on its own (see
## _DEFAULT_ICON); only set `icon` explicitly to override that.
##
## @tool so `icon`/`label_text` show up while editing.
@tool
extends PanelContainer

const PRESS_SCALE := 1.08
const PRESS_TWEEN_DURATION := 0.08

const _DEFAULT_ICON := {
	GameAction.CONFIRM: preload("res://theme/icons/dot_green.png"),
	GameAction.BACK: preload("res://theme/icons/dot_red.png"),
	GameAction.SUBMIT: preload("res://theme/icons/dot_blue.png"),
}

@export var icon: Texture2D:
	set(value):
		icon = value
		if is_node_ready():
			_apply_icon()

@export var label_text: String = "":
	set(value):
		label_text = value
		if is_node_ready():
			_apply_label()

## Which physical action this badge reflects, e.g. GameAction.CONFIRM.
## Left empty, it just sits static - useful while editing in isolation.
@export var action: StringName = &"":
	set(value):
		action = value
		if is_node_ready():
			_apply_icon()

@onready var _icon_rect: TextureRect = %Icon
@onready var _label: Label = %Label

var _press_tween: Tween


func _ready() -> void:
	focus_mode = Control.FOCUS_NONE
	mouse_filter = Control.MOUSE_FILTER_IGNORE
	theme_type_variation = &"PanelCard"

	_apply_icon()
	_apply_label()


func _apply_icon() -> void:
	var tex := icon if icon else _DEFAULT_ICON.get(action) as Texture2D
	_icon_rect.texture = tex
	_icon_rect.visible = tex != null


func _apply_label() -> void:
	_label.text = label_text.to_upper()
	_label.visible = not label_text.is_empty()


func _unhandled_input(event: InputEvent) -> void:
	if action == &"":
		return
	if event.is_action_pressed(action):
		_tween_scale(PRESS_SCALE)
	elif event.is_action_released(action):
		_tween_scale(1.0)


func _tween_scale(target_factor: float) -> void:
	if _press_tween:
		_press_tween.kill()
	_press_tween = create_tween().set_trans(Tween.TRANS_QUAD).set_ease(Tween.EASE_OUT)
	_press_tween.tween_property(self, "scale", Vector2.ONE * target_factor, PRESS_TWEEN_DURATION)
