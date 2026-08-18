extends HBoxContainer

func _ready():
	alignment = BoxContainer.ALIGNMENT_CENTER
	add_theme_constant_override("separation", 26)
	set_anchors_and_offsets_preset(Control.PRESET_BOTTOM_WIDE)
	# Position at bottom 46-52
