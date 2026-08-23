## Icon+label badge for the bottom hint bar, for actions shown as a hold
## instead of a tap (e.g. "hold B to cancel"). Same decoration-only
## contract as IconButton: no focus, no mouse, no selection state - it
## purely mirrors a HoldAction on `action`, scaling up and pausing at the
## peak while held, with %ProgressFill growing toward `hold_duration`.
## Releasing early resets both with no effect - the badge never causes
## anything itself, it just shows a hold happening elsewhere is in
## progress (see HoldAction's own doc comment for that split).
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
const HOLD_LABEL_PREFIX := "HALTEN: "

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

## Which physical action this badge reflects, e.g. GameAction.BACK. Also
## configures the internal HoldAction, so - unlike `icon`/`label_text` -
## this one only takes effect if set before the node enters the tree (see
## _ready()).
@export var action: StringName = &"":
	set(value):
		action = value
		if is_node_ready():
			_apply_icon()

@export var hold_duration: float = 0.7

@onready var _icon_rect: TextureRect = %Icon
@onready var _label: Label = %Label
@onready var _progress_fill: Control = %ProgressFill

var _hold_action: HoldAction
var _press_tween: Tween


func _ready() -> void:
	focus_mode = Control.FOCUS_NONE
	mouse_filter = Control.MOUSE_FILTER_IGNORE
	theme_type_variation = &"PanelCard"

	_progress_fill.anchor_right = 0.0

	# action/duration configure the internal HoldAction once here, at
	# _ready() - fine for the normal case (both set as instance overrides
	# in the scene, applied before this ever runs), but changing either at
	# runtime after the node is in the tree won't reconfigure it.
	_hold_action = HoldAction.new()
	_hold_action.action = action
	_hold_action.duration = hold_duration
	_hold_action.progress_changed.connect(_on_progress_changed)
	add_child(_hold_action)

	_apply_icon()
	_apply_label()


func _apply_icon() -> void:
	var tex := icon if icon else _DEFAULT_ICON.get(action) as Texture2D
	_icon_rect.texture = tex
	_icon_rect.visible = tex != null


func _apply_label() -> void:
	_label.text = HOLD_LABEL_PREFIX + label_text.to_upper()
	_label.visible = not label_text.is_empty()


func _on_progress_changed(value: float) -> void:
	_progress_fill.anchor_right = value
	_tween_scale(PRESS_SCALE if value > 0.0 else 1.0)


func _tween_scale(target_factor: float) -> void:
	if _press_tween:
		_press_tween.kill()
	_press_tween = create_tween().set_trans(Tween.TRANS_QUAD).set_ease(Tween.EASE_OUT)
	_press_tween.tween_property(self, "scale", Vector2.ONE * target_factor, PRESS_TWEEN_DURATION)
