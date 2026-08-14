## Base class for the per-game card GameSelectionScreen instantiates one of
## per released game. Jesco builds the actual visual card (cover, title,
## focus styling); this just defines what GameSelectionScreen needs from it:
## a way to feed it game data, and a signal for "this card was activated".
class_name GameCard
extends Control

signal activated

func set_game(_game: Dictionary) -> void:
	pass # Override to populate cover/title/etc.
