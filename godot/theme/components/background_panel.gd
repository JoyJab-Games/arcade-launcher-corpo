extends PanelContainer

@export var show_pattern: bool = true

func _ready():
	var stylebox = get_theme_stylebox("panel").duplicate()
	stylebox.bg_color = Color("#FFF9EF") # background/surface (joyjab.games)
	stylebox.corner_radius_top_left = 26
	stylebox.corner_radius_top_right = 26
	stylebox.corner_radius_bottom_right = 26
	stylebox.corner_radius_bottom_left = 26
	add_theme_stylebox_override("panel", stylebox)

	if show_pattern:
		_add_pattern()

func _add_pattern():
	var pattern = TextureRect.new()
	# Pattern logic from spec: 72x72, 4px dots at 30%/30%
	# We would typically use a pre-rendered noise or dot texture.
	# For now, we'll create a placeholder or assume a texture exists.
	pattern.stretch_mode = TextureRect.STRETCH_TILE
	pattern.modulate = Color(0.18, 0.14, 0.10, 0.045) # pattern_ink
	pattern.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	pattern.show_behind_parent = true
	add_child(pattern)
