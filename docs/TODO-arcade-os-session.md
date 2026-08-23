# TODO: wire this launcher into `arcade-os` as a gamescope session

**Repo:** `JoyJab-Games/arcade-os` (separate repo — this is a handoff note
for whoever/whatever picks this up, not something actionable inside
`arcade-launcher-corpo` itself). Public repo, clone with
`git clone https://github.com/JoyJab-Games/arcade-os.git`.

**Do this in a new branch of `arcade-os`** — this replaces the current
`steam-boot` role's entire session layer, significant enough not to land
directly on whatever the default branch is.

## Context (read this before touching anything)

`arcade-os` currently boots straight into real Steam Big Picture via
**Jovian-NixOS** (`modules/roles/steam-boot/arcade.nix`:
`jovian.steam.enable/autoStart`, `desktopSession = "gamescope-wayland"`).
That's never been connected to this launcher project at all — the cabinet
today runs Steam's own UI, not `arcade-launcher`.

Decisions already made (across a long back-and-forth — don't re-litigate
these, just execute):

1. **Drop Jovian entirely.** It was pulled in purely for its Steam
   autostart wrapper, which we don't want (no Steam UI should ever be
   shown to players — this launcher replaces it). Its AMD/hardware quirk
   patches aren't needed either, since gamescope itself doesn't require
   Jovian to run.
2. **Use gamescope directly, embedded mode** (talks straight to DRM/KMS,
   no parent compositor) — the exact same shape
   [ChimeraOS's `gamescope-session`](https://github.com/ChimeraOS/gamescope-session)
   and, built on top of it,
   [`gamescope-session-opengamepadui`](https://github.com/ShadowBlip/gamescope-session-opengamepadui)
   use. OpenGamepadUI is the closest prior art to this whole project (also
   a Godot 4 app, also a gamepad-native game launcher) — we're deliberately
   copying its proven architecture rather than inventing our own.
3. **NOT cage.** Cage was considered (see git history/conversation log if
   you need the reasoning) and explicitly rejected for the actual
   game-hosting role: cage is a single-client kiosk compositor with no
   concept of a second client, so it can't host "launcher UI + a running
   game, switchable" the way gamescope can. Gamescope is what actually
   provides that (via its `GAMESCOPECTRL_BASELAYER_APPID` /
   `STEAM_GAME` X11-atom control protocol — see
   `arcade-launcher-corpo/rust/core/src/gamescope.rs` for exactly how this
   launcher already talks to it).
4. **`arcade_launcher_corpo`'s Rust side is already gamescope-aware.**
   `rust/core/src/gamescope.rs` tags its own window and the running game's
   window via `gamescope-x11-client`
   (https://github.com/ShadowBlip/gamescope-x11-client, the same crate
   OpenGamepadUI's author built), and flips
   `GAMESCOPECTRL_BASELAYER_APPID` to hand focus between them. **This only
   does anything when actually running inside a real gamescope session —
   it's a silent no-op everywhere else** (confirmed: exported binary runs
   fine headlessly with none of this code erroring). So this TODO is
   purely about getting the launcher running *inside* gamescope at all —
   no further launcher-side code changes needed for the compositor
   integration itself.

## What to actually build

Replace `arcade.nix`'s Jovian-based session with a `gamescope-session`
-shaped role:

- Drop the `jovian` flake input from `arcade-os/flake.nix` (unless
  something else in the repo still needs it — check first).
- New session role, modeled on ChimeraOS's `gamescope-session` script:
  `greetd` (or your own systemd unit) autologins the kiosk user straight
  into `gamescope` (embedded, direct DRM/KMS), with `CLIENTCMD` pointed at
  this launcher's built package.
- `steamcmd` and `umu-launcher` stay as plain packages in
  `environment.systemPackages` (unrelated to the compositor — the
  `arcade` CLI/launcher already shell out to these directly, see
  `arcade-launcher-corpo/nixos/package.nix`/`cli-package.nix`).
- **This launcher needs to be an input to `arcade-os`'s flake.** It
  already has real, working package outputs to consume:
  - `packages.x86_64-linux.default` (alias `arcade-launcher`) — the
    exported Godot binary, wrapped with `umu-run`/`steamcmd` on `PATH`.
    This is `CLIENTCMD`.
  - `packages.x86_64-linux.arcade-cli` — the `arcade` admin CLI, wrapped
    the same way. Put this in `environment.systemPackages` for SSH admin
    access (`admin.nix` already sets up the SSH/Tailscale side of that).
  - Both are verified working: built via `nix build .#default` /
    `.#arcade-cli` in this repo, smoke-tested headlessly (extension
    loads, boot flow runs, clean exit).
- `admin.nix` (SSH, Tailscale, your key) and `modules/common`
  (locale/audio/graphics/NetworkManager) and `modules/hardware/geekom-a6`
  (disko/GRUB/kernel) all stay exactly as they are — none of this is
  Jovian-specific.

## Testing without real hardware

Raised as an open question, worth solving before this ships blind to a
physical cabinet: **can gamescope be spun up nested, from the dev flow,
for local testing?**

Gamescope supports a genuine **nested mode** (runs as an ordinary Wayland
client inside any other compositor — this is how people test gamescope
under GNOME/Sway on a normal desktop already). That means, in principle:

```
gamescope -- ${arcade-launcher}/bin/arcade-launcher
```

run from an already-logged-in desktop session (nested gamescope, a window
on your existing screen) should be enough to exercise the *entire*
gamescope integration path — window tagging, baselayer focus swap on
launch/exit — for real, without touching `greetd`/embedded mode/actual
display hardware at all. Embedded-vs-nested only changes who owns
DRM/KMS; the X11-atom control protocol `gamescope.rs` talks to is the same
either way.

Concretely, worth adding to this repo (`arcade-launcher-corpo`) as a new
`nix run .#dev-gamescope`-style app: same idea as `apps/dev.nix`, but
wrapping the built `arcade-launcher` package (or even the editor, for
faster iteration) in a nested `gamescope --` invocation, with `godot_4`
added to gamescope's own dependency closure. Add `pkgs.gamescope` to
`nixos/devshell.nix` first. This would let the whole "does focus actually
swap when a game launches" question get answered on a dev machine, before
ever touching `arcade-os`/real hardware — do this *before* spending time
debugging on the cabinet itself if the real thing doesn't work first try.

## Known remaining gaps (not blocking this task, just don't be surprised)

- No crash-vs-clean-exit distinction yet in `arcade_core::launch` — a
  crashed game and a normal quit look identical to the launcher right now.
- `arcade-os`'s `configs/steam-boot.nix` looks like dead/orphaned code
  (references module paths that don't exist in the current tree,
  `flake.nix` never references it) — probably safe to delete once you've
  confirmed it's genuinely unused, but not this task's job to fix.
