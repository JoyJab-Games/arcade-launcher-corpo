extends PanelContainer

func _ready():
	focus_mode = FOCUS_ALL

	var style = StyleBoxFlat.new()
	style.bg_color = Color("#FFFFFF")
	style.set_corner_radius_all(45) # Fully round for height ~90
	style.content_margin_left = 44
	style.content_margin_right = 44
	style.shadow_color = Color(0.18, 0.14, 0.10, 0.08) # sh_rest
	style.shadow_size = 12
	add_theme_stylebox_override("panel", style)

	FocusEffect.setup_focus_signals(self, 1.02)
	focus_entered.connect(_on_focus_entered)
	focus_exited.connect(_on_focus_exited)

func _on_focus_entered():
	var style = get_theme_stylebox("panel")
	style.bg_color = Color("#FFB702") # accent_yellow
	style.shadow_color = Color(0.18, 0.14, 0.10, 0.22) # sh_focus_row
	style.shadow_size = 23
	style.shadow_offset = Vector2(0, 11)

func _on_focus_exited():
	var style = get_theme_stylebox("panel")
	style.bg_color = Color("#FFFFFF")
	style.shadow_color = Color(0.18, 0.14, 0.10, 0.08)
	style.shadow_size = 12
