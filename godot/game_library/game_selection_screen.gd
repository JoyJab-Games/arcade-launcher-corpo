## Shows every game currently released for players, all equal weight — no
## autostart-style emphasis on any single one. A on a card launches it.
## This is where a running game returns to when it exits (once that hook
## exists — Phase 1), so the roster is rebuilt fresh on every entry in case
## an admin changed what's released while a game was running.
##
## Jesco's scene provides `card_container` (any Container to add cards
## into) and `card_scene` (a PackedScene whose root extends GameCard).
class_name GameSelectionScreen
extends Screen

@export var card_container: Container
@export var card_scene: PackedScene

var _cards: Array[GameCard] = []


func enter(_context: Dictionary = {}) -> void:
	_rebuild_cards()
	super.enter(_context)


func _rebuild_cards() -> void:
	for card in _cards:
		card.queue_free()
	_cards.clear()

	for game in GameRoster.get_released_games():
		var card := card_scene.instantiate() as GameCard
		assert(card != null, "GameSelectionScreen: card_scene root must extend GameCard")
		card.set_game(game)
		card.activated.connect(FrontFlow.launch.bind(game))
		card_container.add_child(card)
		_cards.append(card)

	if not _cards.is_empty():
		initial_focus = _cards[0]
