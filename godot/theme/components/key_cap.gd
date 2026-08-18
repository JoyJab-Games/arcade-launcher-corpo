extends PanelContainer

func _ready():
	focus_mode = FOCUS_ALL

	var style = StyleBoxFlat.new()
	style.bg_color = Color("#FFFFFF")
	style.set_corner_radius_all(22)
	style.shadow_color = Color(0.18, 0.14, 0.10, 0.08) # sh_flat
	style.shadow_size = 10
	add_theme_stylebox_override("panel", style)

	FocusEffect.setup_focus_signals(self, 1.14)
	focus_entered.connect(_on_focus_entered)
	focus_exited.connect(_on_focus_exited)

func _on_focus_entered():
	get_theme_stylebox("panel").bg_color = Color("#FFB702") # accent_yellow
	# Shadow token sh_focus_key
	var style = get_theme_stylebox("panel")
	style.shadow_color = Color(0.18, 0.14, 0.10, 0.26)
	style.shadow_size = 22
	style.shadow_offset = Vector2(0, 11)

func _on_focus_exited():
	get_theme_stylebox("panel").bg_color = Color("#FFFFFF")
	var style = get_theme_stylebox("panel")
	style.shadow_color = Color(0.18, 0.14, 0.10, 0.08)
	style.shadow_size = 10
