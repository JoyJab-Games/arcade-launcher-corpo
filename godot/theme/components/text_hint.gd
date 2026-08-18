extends Label

func _ready():
	add_theme_font_override("font", load("res://theme/font/Fredoka-Medium.ttf"))
	add_theme_font_size_override("font_size", 16)
	add_theme_color_override("font_color", Color("#6E6156")) # ink_muted

	# UPPERCASE and Letter Spacing (+2px)
	text = text.to_upper()
	# Note: Native Godot Label doesn't have a direct "letter spacing" property in pixels easily exposed in simple scripts
	# without a FontVariation, but we'll assume the theme handles the base style.
