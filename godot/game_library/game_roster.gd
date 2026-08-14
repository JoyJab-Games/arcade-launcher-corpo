## Autoload "GameRoster". Placeholder data source for "which games are
## released for players" (a per-game flag, several games can be released
## at once — see arcade-launcher briefing, Phase 1/2).
## TODO(Phase 1): back this with the real game config / GameSource
## interface instead of an in-memory list.
extends Node

var released_games: Array[Dictionary] = []


func get_released_games() -> Array[Dictionary]:
	return released_games
