## The engine's actual `run/main_scene` — deliberately minimal, no visible
## content of its own. Every screen is owned by ScreenRouter (see its own
## doc comment: "only the top of the stack is visible/active") and rendered
## through its `_host` container, which lives under the ScreenRouter
## autoload, not under whatever main_scene happens to be. This node's only
## job is the one thing nothing else can do: kick off ScreenRouter's very
## first push. Godot calls _ready() on it directly (the normal engine
## lifecycle), unlike every screen after this one, which goes through the
## custom Screen.enter()/exit() pair ScreenRouter drives instead.
extends Node

## Which screen the game actually starts on — configurable here rather
## than hardcoded, same reasoning as BootScreen's own needs_help_scene/
## selection_scene exports: keeps this generic bootstrap node reusable
## instead of baking in knowledge of boot_screen.tscn specifically.
@export var first_screen: PackedScene = preload("res://boot/boot_screen.tscn")


func _ready() -> void:
	ScreenRouter.push(first_screen)
