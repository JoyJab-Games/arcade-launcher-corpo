## Generic "hold this action for N seconds" gesture — e.g. B held 700ms to
## cancel out of the password screen while a tap of the same button deletes
## a character. Not tied to any one action or screen; instantiate as a
## child node, set `action`/`duration`, and read the three signals:
##
## - progress_changed(value: float): 0..1 while held, for a HoldBar fill.
## - tapped: action was released before `duration` elapsed.
## - held: action stayed pressed for the full `duration`.
##
## Only one of `tapped`/`held` ever fires per press.
class_name HoldAction
extends Node

signal progress_changed(value: float)
signal tapped
signal held

@export var action: StringName = &""
@export var duration: float = 0.7

var _pressed := false
var _fired := false
var _elapsed := 0.0


func _unhandled_input(event: InputEvent) -> void:
	if event.is_action_pressed(action):
		_pressed = true
		_fired = false
		_elapsed = 0.0
	elif event.is_action_released(action):
		if _pressed and not _fired:
			tapped.emit()
		_pressed = false
		progress_changed.emit(0.0)


func _process(delta: float) -> void:
	if not _pressed or _fired:
		return

	_elapsed += delta
	var t := clampf(_elapsed / duration, 0.0, 1.0)
	progress_changed.emit(t)

	if t >= 1.0:
		_fired = true
		held.emit()
