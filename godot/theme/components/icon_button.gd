## Composite icon+label button for controller-only UI (bottom bars, in-game
## overview menu, ...). Deliberately not a native Button: Button's
## icon_normal_color/icon_hover_color/... theme properties tint the icon to
## match the button's text color, which we don't want — the icon should
## keep its own colors. Built the same way as GameCard instead:
## PanelContainer background (swaps PanelCard/PanelCardFocused on focus,
## same as GameCard) with independent icon/label children, activation
## wired through GameAction.setup_confirm rather than Button's own signals.
extends PanelContainer

const UNFOCUSED_STYLE := &"PanelCard"
const FOCUSED_STYLE := &"PanelCardFocused"

signal activated

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

@onready var _icon_rect: TextureRect = %Icon
@onready var _label: Label = %Label


func _ready() -> void:
	GameAction.setup_confirm(self, func(): activated.emit())
	FocusEffect.setup_focus_signals(self, 1.06)
	focus_entered.connect(func(): theme_type_variation = FOCUSED_STYLE)
	focus_exited.connect(func(): theme_type_variation = UNFOCUSED_STYLE)
	theme_type_variation = UNFOCUSED_STYLE

	_apply_icon()
	_apply_label()


func _apply_icon() -> void:
	_icon_rect.texture = icon
	_icon_rect.visible = icon != null


func _apply_label() -> void:
	_label.text = label_text
	_label.visible = not label_text.is_empty()
