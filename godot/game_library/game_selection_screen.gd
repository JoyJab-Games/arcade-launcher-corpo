## Shows every game currently released for players, all equal weight — no
## autostart-style emphasis on any single one. A on a card asks whoever
## pushed this screen to launch it, via game_selected — this screen is a
## dumb resolver: it doesn't know where game data comes from or what
## "launch" means, only how to display a list and report which one got
## picked. That keeps it testable standalone (push it with a fixed
## context["games"] list, activate a card, assert the signal) with no
## GameRoster/FrontFlow dependency.
##
## Whoever pushes this screen must supply context["games"] (Array[Dictionary]
## — same shape GameRoster.get_released_games() returns) — prepackaged at
## push time, not fetched here. This is where a running game returns to when
## it exits (once that hook exists — Phase 1); FrontFlow re-supplies
## context["games"] fresh on every push in case an admin changed what's
## released while a game was running. Re-entries that don't pass "games"
## (e.g. plain back-navigation once nested screens exist) keep whatever's
## already built.
##
## The scene provides a GridContainer(columns=4) marked "Access as Unique
## Name" as %CardContainer — GridContainer specifically, so Godot's
## automatic focus-neighbor resolution gives correct up/down/left/right grid
## navigation for free — and sets `card_scene` (a PackedScene whose root
## extends GameCard) in the Inspector.
class_name GameSelectionScreen
extends Screen

## Emitted when a card is activated. Carries the same Dictionary shape it
## was given via context["games"] — the caller decides what launching means.
signal game_selected(game: Dictionary)

@onready var card_container: Container = %CardContainer
@export var card_scene: PackedScene

var _cards: Array[GameCard] = []


func enter(context: Dictionary = {}) -> void:
	if context.has("games"):
		_rebuild_cards(context["games"])
	super.enter(context)


func _rebuild_cards(games: Array[Dictionary]) -> void:
	for card in _cards:
		card.queue_free()
	_cards.clear()

	for game in games:
		var card := card_scene.instantiate() as GameCard
		assert(card != null, "GameSelectionScreen: card_scene root must extend GameCard")
		card.set_game(game)
		card.activated.connect(game_selected.emit.bind(game))
		card_container.add_child(card)
		_cards.append(card)

	if not _cards.is_empty():
		initial_focus = _cards[0]
