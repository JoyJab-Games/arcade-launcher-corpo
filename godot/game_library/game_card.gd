## Base class for the per-game card GameSelectionScreen instantiates one of
## per released game. The visual card (cover, title, focus styling) is built
## on top of this; this defines what GameSelectionScreen needs from it: a
## way to feed it game data, and a signal for "this card was activated".
##
## Activation is wired here (GameAction.CONFIRM while focused) rather than
## per-visual-subclass, since GameCard is Control, not Button — see
## GameAction.setup_confirm. Subclasses that override _ready() must call
## super() to keep it.
##
## Focus styling also lives here rather than a subclass: game_card.tscn is
## currently the only concrete card visual, and attaches this script
## directly. It swaps the panel's theme_type_variation, reveals the
## description/tags (marked %DescriptionLabel/%TagRow, "Access as Unique
## Name") only while focused, and scales the card up so it reads as
## "selected" on a controller.
class_name GameCard
extends Control

const UNFOCUSED_STYLE := &"PanelCard"
const FOCUSED_STYLE := &"PanelCardFocused"
const FOCUSED_SCALE := Vector2(1.06, 1.06)
const SCALE_DURATION := 0.12

signal activated

@onready var _description_label: Label = %DescriptionLabel
@onready var _tag_row: Control = %TagRow

var _scale_tween: Tween


func _ready() -> void:
	GameAction.setup_confirm(self, func(): activated.emit())
	pivot_offset = size / 2.0
	resized.connect(func(): pivot_offset = size / 2.0)
	focus_entered.connect(_on_focus_entered)
	focus_exited.connect(_on_focus_exited)
	_apply_focus_visuals(false)


func set_game(_game: Dictionary) -> void:
	pass # Override to populate cover/title/etc.


func _on_focus_entered() -> void:
	_apply_focus_visuals(true)


func _on_focus_exited() -> void:
	_apply_focus_visuals(false)


func _apply_focus_visuals(is_focused: bool) -> void:
	theme_type_variation = FOCUSED_STYLE if is_focused else UNFOCUSED_STYLE
	_description_label.visible = is_focused
	_tag_row.visible = is_focused

	if _scale_tween:
		_scale_tween.kill()
	_scale_tween = create_tween()
	_scale_tween.set_trans(Tween.TRANS_BACK).set_ease(Tween.EASE_OUT)
	_scale_tween.tween_property(self, ^"scale", FOCUSED_SCALE if is_focused else Vector2.ONE, SCALE_DURATION)
