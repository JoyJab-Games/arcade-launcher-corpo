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
##
## set_game() populates title/description/tags/cover from the same
## Dictionary shape GameRoster.get_released_games() returns (see
## rust/gdext/src/game_library.rs::game_to_dictionary) - "name" doubles as
## the display title (GameConfig has no separate display-name field; the
## admin's own save-name, chosen at install time, is meant to already be
## presentable - see arcade install's "Save as which name?" prompt).
class_name GameCard
extends Control

const UNFOCUSED_STYLE := &"PanelCard"
const FOCUSED_STYLE := &"PanelCardFocused"
const FOCUSED_SCALE := Vector2(1.06, 1.06)
const SCALE_DURATION := 0.12

signal activated

@onready var _title_label: Label = %TitleLabel
@onready var _description_label: Label = %DescriptionLabel
@onready var _tag_row: Control = %TagRow
@onready var _cover_texture: TextureRect = %CoverTexture

var _scale_tween: Tween


func _ready() -> void:
	GameAction.setup_confirm(self, func(): activated.emit())
	pivot_offset = size / 2.0
	resized.connect(func(): pivot_offset = size / 2.0)
	focus_entered.connect(_on_focus_entered)
	focus_exited.connect(_on_focus_exited)
	_apply_focus_visuals(false)


func set_game(game: Dictionary) -> void:
	_title_label.text = game.get("name", "")
	_description_label.text = game.get("description", "")

	for child in _tag_row.get_children():
		child.queue_free()
	for tag in game.get("tags", []):
		var pill := PanelContainer.new()
		pill.theme_type_variation = &"PanelPill"
		var label := Label.new()
		label.theme_type_variation = &"LabelMicro"
		label.text = tag
		label.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
		label.vertical_alignment = VERTICAL_ALIGNMENT_CENTER
		pill.add_child(label)
		_tag_row.add_child(pill)

	# image_path is an absolute filesystem path (the game's own install
	# folder, outside the Godot project entirely - see image_abs_path in
	# rust/core/src/game_config.rs), not a res://. Plain load() only
	# understands the project's own imported resources (.import
	# sidecars) - an arbitrary external file needs Image.load() (format
	# sniffing/decoding directly) wrapped in an ImageTexture instead. No
	# image fetched at install/update time (or a load failure) just
	# leaves whatever cover is already in the scene rather than clearing
	# to nothing.
	var image_path: String = game.get("image_path", "")
	if not image_path.is_empty():
		var image := Image.new()
		if image.load(image_path) == OK:
			_cover_texture.texture = ImageTexture.create_from_image(image)


func _on_focus_entered() -> void:
	_apply_focus_visuals(true)


func _on_focus_exited() -> void:
	_apply_focus_visuals(false)


func _apply_focus_visuals(is_focused: bool) -> void:
	theme_type_variation = FOCUSED_STYLE if is_focused else UNFOCUSED_STYLE
	_description_label.visible = is_focused and not _description_label.text.is_empty()
	_tag_row.visible = is_focused and _tag_row.get_child_count() > 0

	if _scale_tween:
		_scale_tween.kill()
	_scale_tween = create_tween()
	_scale_tween.set_trans(Tween.TRANS_BACK).set_ease(Tween.EASE_OUT)
	_scale_tween.tween_property(self, ^"scale", FOCUSED_SCALE if is_focused else Vector2.ONE, SCALE_DURATION)
