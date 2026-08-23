extends PanelContainer

@onready var title_label = $VBox/Title
@onready var tagline_label = $VBox/Tagline
@onready var tag_container = $VBox/Tags
@onready var focus_dot = $VBox/TitleHBox/FocusDot

func _ready():
	# Base style already handled by card.gd or theme
	tagline_label.visible = false
	tag_container.visible = false
	if focus_dot: focus_dot.visible = false

	focus_entered.connect(_on_focus_entered)
	focus_exited.connect(_on_focus_exited)

func _on_focus_entered():
	var style = get_theme_stylebox("panel")
	style.set_corner_radius_all(30)
	style.shadow_color = Color(0.18, 0.14, 0.10, 0.26) # sh_focus_card
	style.shadow_size = 32
	style.shadow_offset = Vector2(0, 15)

	tagline_label.visible = true
	tag_container.visible = true
	if focus_dot: focus_dot.visible = true

	title_label.add_theme_font_override("font", load("res://theme/font/Fredoka-Bold.ttf"))

func _on_focus_exited():
	var style = get_theme_stylebox("panel")
	style.set_corner_radius_all(26)
	style.shadow_color = Color(0.18, 0.14, 0.10, 0.10) # sh_card
	style.shadow_size = 15

	tagline_label.visible = false
	tag_container.visible = false
	if focus_dot: focus_dot.visible = false

	title_label.add_theme_font_override("font", load("res://theme/font/Fredoka-SemiBold.ttf"))
