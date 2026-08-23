extends PanelContainer

func _ready():
	# Allow focus
	focus_mode = FOCUS_ALL

	var style = StyleBoxFlat.new()
	style.bg_color = Color("#FFFFFF")
	style.set_corner_radius_all(26)
	style.content_margin_all(14)
	style.shadow_color = Color(0.18, 0.14, 0.10, 0.10) # sh_card
	style.shadow_size = 15
	style.shadow_offset = Vector2(0, 6)
	add_theme_stylebox_override("panel", style)

	FocusEffect.setup_focus_signals(self, 1.09)
