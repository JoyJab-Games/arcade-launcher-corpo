extends PanelContainer

func _ready():
	# ColorDot implementation: 24px-40px circle
	var style = StyleBoxFlat.new()
	style.set_corner_radius_all(20) # Round circle
	style.draw_center = true
	# Color is set via script or variation
	add_theme_stylebox_override("panel", style)

func set_button_color(color: Color):
	get_theme_stylebox("panel").bg_color = color
