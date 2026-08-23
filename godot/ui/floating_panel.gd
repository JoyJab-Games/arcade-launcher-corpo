extends PanelContainer

func _ready():
	var style = StyleBoxFlat.new()
	style.bg_color = Color("#FFFFFF") # surface_white
	style.set_corner_radius_all(46)
	style.content_margin_left = 64
	style.content_margin_right = 64
	style.content_margin_top = 56
	style.content_margin_bottom = 56

	style.shadow_color = Color(0.18, 0.14, 0.10, 0.14) # sh_panel
	style.shadow_size = 35 # CSS Blur 70 / 2
	style.shadow_offset = Vector2(0, 13) # y/2

	add_theme_stylebox_override("panel", style)
