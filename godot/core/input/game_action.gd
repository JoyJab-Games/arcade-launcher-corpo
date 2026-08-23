## Central vocabulary for the launcher's 6 gamepad actions (see project docs,
## "Tastenbelegung"), so nothing else hardcodes raw InputMap action-name
## strings. Physical button/key bindings live in project.godot's [input]
## section; this is just the shared names plus the one bit of behavior every
## composite, non-Button component needs: reacting to a confirm press while
## focused.
##
## ui_up/down/left/right and focus traversal are NOT redefined here — those
## stay Godot's built-in ui_* actions, which already drive Control focus
## movement (grid/list navigation) for free.
class_name GameAction
extends RefCounted

const CONFIRM := &"ui_confirm" # A, btn_green — confirm / start / type a character
const BACK := &"ui_back" # B, btn_red — back; tap = delete in text entry
const SUBMIT := &"ui_submit" # Y, btn_blue — connect / submit
const PAGE_PREV := &"ui_page_prev" # LB, btn_white — previous page/group
const PAGE_NEXT := &"ui_page_next" # RB, btn_white — next page/group
const OVERVIEW := &"ui_overview" # Home — open/close in-game overview


## Wires "confirm" activation for a composite Control-based component (a
## GameCard, ListRow, ...) that isn't a Button and so gets no
## built-in press behavior. While `control` holds focus, a ui_confirm press
## calls `callback`. One line from the component's _ready() — see
## GameCard._ready() for the reference use.
static func setup_confirm(control: Control, callback: Callable) -> void:
	control.focus_mode = Control.FOCUS_ALL
	control.gui_input.connect(func(event: InputEvent) -> void:
		if event.is_action_pressed(CONFIRM):
			control.accept_event()
			callback.call()
	)
