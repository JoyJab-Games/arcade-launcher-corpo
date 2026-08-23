class_name FocusEffect
extends Node

const TWEEN_DURATION = 0.12

static func apply_focus_effect(node: Control, focused: bool, scale_factor: float = 1.0):
	if not node.is_inside_tree():
		return
	var tween = node.create_tween().set_parallel(true).set_trans(Tween.TRANS_CUBIC).set_ease(Tween.EASE_OUT)

	if focused:
		node.z_index = 1
		if node.get_parent():
			node.get_parent().move_child(node, node.get_parent().get_child_count() - 1)

		tween.tween_property(node, "scale", Vector2.ONE * scale_factor, TWEEN_DURATION)
	else:
		node.z_index = 0
		tween.tween_property(node, "scale", Vector2.ONE, TWEEN_DURATION)

static func setup_focus_signals(node: Control, scale_factor: float = 1.0):
	node.pivot_offset = node.size / 2.0
	node.item_rect_changed.connect(func(): node.pivot_offset = node.size / 2.0)

	node.focus_entered.connect(func(): apply_focus_effect(node, true, scale_factor))
	node.focus_exited.connect(func(): apply_focus_effect(node, false, scale_factor))
