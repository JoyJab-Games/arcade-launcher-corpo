## Boot-screen logo mark: white pill panel (styled via the LogoLockup theme
## type variation in default_theme.tres — not hardcoded here, see
## LoadingDot/LabelMicro for the same pattern) that gently floats up and down
## forever. Matches the mockup's `bob` keyframe exactly: ±12px, 2.6s total,
## ease-in-out, infinite.
##
## The logo image itself also comes from the theme (LogoLockup/icons/logo),
## not a hardcoded texture reference on the child TextureRect — a
## per-cabinet override theme.tres can swap the whole mark (panel style +
## logo) this way. Height is fixed at 132px per spec; width is derived from
## the icon's real aspect ratio so a differently-shaped whitelabel logo
## doesn't get stretched.
##
## @tool so the logo actually shows — and bobs — while editing this scene
## (or boot_screen.tscn) in the editor, not just at runtime; neither
## _ready() nor the animation normally run in-editor otherwise.
@tool
extends PanelContainer

@export var logo_height := 132.0:
	set(value):
		logo_height = value
		if is_node_ready():
			_apply_logo_icon()
@export var bob_height := 12.0
@export var bob_half_duration := 1.3

@onready var _logo: TextureRect = $Logo


func _ready() -> void:
	_apply_logo_icon()
	_start_bob_animation()


func _apply_logo_icon() -> void:
	# No explicit type-variation name here on purpose: with the argument
	# omitted, Godot resolves against whatever theme_type_variation is set
	# on this node itself, so a rename in the scene doesn't also require a
	# matching hardcoded string here.
	var icon := get_theme_icon(&"logo")
	if not icon:
		return
	_logo.texture = icon
	var aspect := float(icon.get_width()) / float(icon.get_height())
	_logo.custom_minimum_size = Vector2(logo_height * aspect, logo_height)


func _start_bob_animation() -> void:
	var tween := create_tween().set_loops()
	tween.set_trans(Tween.TRANS_SINE).set_ease(Tween.EASE_IN_OUT)
	tween.tween_property(self, "position:y", position.y - bob_height, bob_half_duration)
	tween.tween_property(self, "position:y", position.y, bob_half_duration)
