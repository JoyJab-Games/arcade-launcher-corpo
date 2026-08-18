## Three-dot loading indicator, used under LogoLockup on Boot (§5.20) and
## anywhere else a "still working" status needs one (e.g. wifi scan). Each
## dot bounces up and down (not the mockup's opacity pulse — a vertical
## bounce instead, per Jesco), staggered 0.2s apart across a 1.2s
## ease-in-out cycle, looping forever.
##
## @tool so it animates while editing too, not just at runtime — _ready()
## doesn't normally run in-editor otherwise.
@tool
extends HBoxContainer

const CYCLE_DURATION := 1.2
const STAGGER := 0.2
const BOUNCE_HEIGHT := 8.0


func _ready() -> void:
	for i in get_child_count():
		_animate_dot(get_child(i), i * STAGGER)


func _animate_dot(dot: Control, delay: float) -> void:
	var base_y := dot.position.y
	# The initial stagger has to happen *before* the looping tween starts,
	# not as the first step inside it — otherwise set_loops() repeats the
	# delay every cycle too, giving each dot a different total period
	# (1.2s/1.4s/1.6s) instead of the same 1.2s cycle just phase-shifted,
	# so they drift in and out of sync instead of staying evenly offset.
	if delay > 0.0:
		await get_tree().create_timer(delay).timeout

	var tween := create_tween().set_loops()
	tween.set_trans(Tween.TRANS_SINE).set_ease(Tween.EASE_IN_OUT)
	tween.tween_property(dot, "position:y", base_y - BOUNCE_HEIGHT, CYCLE_DURATION / 2.0)
	tween.tween_property(dot, "position:y", base_y, CYCLE_DURATION / 2.0)
