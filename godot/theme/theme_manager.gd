## Autoload singleton ("ThemeManager"). Applies the shipped default theme
## (theme/default_theme.tres) to the whole scene tree at boot, then checks
## for a per-cabinet override file in the user data directory. If present
## and a valid Theme resource, it's merged on top of the default
## (Theme.merge_with — a partial override, e.g. just recolored buttons,
## doesn't leave unset properties blank). Missing or invalid override files
## are not an error: the shipped default is always a safe fallback, since a
## broken theme file must never blank/crash the cabinet.
##
## Named "ThemeManager", not "Theme" — an autoload called "Theme" would
## shadow the built-in Theme resource class everywhere in the project.
##
## Must run before ScreenRouter (see [autoload] order in project.godot) so
## nothing is drawn with the editor's default theme first.
##
## Override file location mirrors rust/core/src/lib.rs::data_dir() exactly
## (ARCADE_DATA_DIR env var, else ~/.local/share/arcade-launcher/) so
## admins only need to remember one directory for both games.json and
## theme.tres. One cabinet = one override file; no multi-theme switching.
extends Node

const DEFAULT_THEME_PATH := "res://theme/default_theme.tres"
const OVERRIDE_FILENAME := "theme.tres"

## Emitted once at boot after the final theme (default, or default+override)
## has been applied to the tree root.
signal theme_applied(is_override: bool)

var active_theme: Theme


func _ready() -> void:
	var default_theme := _load_default()
	var override_theme := _load_override()

	if override_theme:
		active_theme = default_theme.duplicate()
		active_theme.merge_with(override_theme)
		get_tree().root.theme = active_theme
		print("ThemeManager: applied override theme from ", _override_path())
		theme_applied.emit(true)
	else:
		active_theme = default_theme
		get_tree().root.theme = active_theme
		theme_applied.emit(false)


func _load_default() -> Theme:
	var theme := ResourceLoader.load(DEFAULT_THEME_PATH, "Theme") as Theme
	assert(theme != null, "ThemeManager: shipped default theme failed to load from " + DEFAULT_THEME_PATH)
	return theme


## Returns null (not an error) whenever there's no usable override — caller
## falls back to the default theme either way.
func _load_override() -> Theme:
	var path := _override_path()
	if not FileAccess.file_exists(path):
		return null

	var loaded := ResourceLoader.load(path, "Theme", ResourceLoader.CACHE_MODE_IGNORE)
	if loaded == null or not (loaded is Theme):
		push_warning("ThemeManager: override at %s is missing or not a Theme resource — falling back to default." % path)
		return null
	return loaded


## Same $ARCADE_DATA_DIR convention as rust/core/src/lib.rs::data_dir().
func _data_dir() -> String:
	var env := OS.get_environment("ARCADE_DATA_DIR")
	if env != "":
		return env
	var home := OS.get_environment("HOME")
	if home == "":
		home = "."
	return home.path_join(".local/share/arcade-launcher")


func _override_path() -> String:
	return _data_dir().path_join(OVERRIDE_FILENAME)
