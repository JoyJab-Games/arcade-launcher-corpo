# System requirements — host setup this app depends on

Nix pins every *package* dependency (`nixos/devshell.nix`, `nixos/apps/`),
but a couple of things this app needs live outside what a flake's
`packages`/`devShells`/`apps` outputs can express on their own: OS-level
group membership granting access to specific device files. Both have
bitten someone by being silently missing before this file existed — it
exists so that stops happening. If you hit a new one, add it here rather
than only fixing it locally.

## `input` group — required for the in-game overview button

`rust/core/src/input_watch.rs` reads raw evdev events directly from
`/dev/input/event*` (gamepad Guide button / keyboard F1) so the in-game
overview can open even while a running game holds gamescope's input
focus — see that file's own doc comment for the full reasoning.

Unlike graphics/audio/USB devices, systemd's seat-ACL mechanism
(`uaccess`) deliberately does **not** auto-grant the logged-in desktop
user access to raw keyboard/gamepad `event*` nodes, even on an active
logind seat session — that access is functionally equivalent to a
keylogger, so it stays gated behind static `input` group membership
instead of dynamic per-session ACLs.

Without it, `input_watch` gets `EACCES` opening every device. This fails
soft by design (the launcher and any running game both still work fully,
just without the in-game overview shortcut while a game holds focus) —
but `spawn_watch` prints an explanation to stderr the first time this
happens, specifically so it's never silently indistinguishable from "no
gamepad plugged in":

```
arcade-launcher: found N input device(s) under /dev/input but couldn't open
any of them (permission denied) - the in-game overview button (Guide/F1)
won't work while a game is running. Fix: add this user to the 'input' group
and log out/in (group membership is applied at login). See docs/system-requirements.md.
```

**Fix**, on whichever NixOS config governs the machine this runs on
(dev box or the real cabinet alike):

```nix
users.users.<you>.extraGroups = [ "input" ];
```

then `nixos-rebuild switch` and **log out/in** (or reboot) — group
membership is resolved at login, a rebuild alone doesn't apply it to an
already-running session. Not something this flake can set on your
behalf yet: it has no `nixosModules` output (see the note at the top of
`nixos/default.nix` — "the NixOS module lands in later phases"), and even
once it does, that only helps whatever NixOS config actually imports it —
your personal dev machine's own system flake almost certainly doesn't.

## `dialout` — not actually this repo's requirement

For programming the cabinet's arcade controllers (flashing their
firmware over serial/USB) — part of the wider arcade project, but
unrelated to arcade-launcher-corpo itself. There's no serial/TTY code
anywhere in this repo (checked: no `/dev/tty*` access, nothing
serial-related in `rust/core` or `godot/`), which matches that. Noted
here only so it doesn't get mistaken for one of this app's requirements
later.
