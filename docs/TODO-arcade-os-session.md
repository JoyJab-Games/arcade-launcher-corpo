# arcade-os: gamescope session — status

**Repo:** `JoyJab-Games/arcade-os` (separate repo, private).
**Branch:** `gamescope-session-arcade-launcher` — pushed, not yet a PR:
https://github.com/JoyJab-Games/arcade-os/pull/new/gamescope-session-arcade-launcher

## What's already done (2026-08-24 night session)

The module is written, pushed, and **the full NixOS system closure builds
successfully end to end** — confirmed via
`nix build .#nixosConfigurations.geekom-a6-arcade-boot.config.system.build.toplevel`
(kernel, initrd, systemd units, greetd config, the session wrapper
script, all of it). Also confirmed by hand: `greetd.toml`'s
`default_session.command` points at the right store path, and the
session wrapper's `PATH` correctly includes both `gamescope` and the real
`arcade-launcher-0.1.0` package built from `arcade-launcher-corpo`'s
current `main` (commit `9fcc6c6`) via a cross-repo flake input.

Concretely, on that branch:

- Dropped Jovian entirely (input + `jovian.steam`/`programs.steam`).
- `modules/roles/steam-boot/` → `modules/roles/arcade-boot/` (rename —
  the role has nothing to do with Steam anymore). `admin.nix` and
  `plymouth-joyjab-arcade/` carried over unchanged (SSH/Tailscale admin
  access and boot splash branding were never Jovian-specific).
- New `modules/roles/arcade-boot/gamescope-session.nix` +
  `gamescope-session.sh`: `services.greetd` autologins straight into
  `gamescope --steam` (embedded mode, direct DRM/KMS), waits on
  gamescope's own startup-socket readiness handshake, then runs
  `arcade-launcher` inside it. Loosely adapted from
  [ChimeraOS's `gamescope-session`](https://github.com/ChimeraOS/gamescope-session)
  (same thing [`gamescope-session-opengamepadui`](https://github.com/ShadowBlip/gamescope-session-opengamepadui)
  builds on) — see that file's own comments for exactly what was kept
  (the readiness handshake, `--steam` for the X11-atom control surface)
  vs. dropped (Steam Deck/handheld-specific HDR/VRR tuning, mangoapp,
  ibus, hardware quirks — none of it relevant to this cabinet's one fixed
  AMD GPU).
- `flake.nix`: `arcade-launcher` input via `git+ssh://` (not `github:` —
  `arcade-launcher-corpo` is private, and the `github:` shorthand needs a
  GitHub API token in `nix.conf` that nothing here should have to depend
  on; SSH key access is already the expected baseline). Machine renamed
  `geekom-a6-arcade-boot`.
- Had to widen `nixpkgs.config.allowUnfreePredicate` to include
  `steam-unwrapped`/`steam-run` (confirmed via a real eval failure,
  not guessed) — these come in as a transitive dependency of
  `pkgs.gamescope` itself, nothing to do with the real Steam client
  actually running anywhere in this session.

## What's NOT done / genuinely still open

- **Never run on real hardware.** This sandbox has no GPU/display, so
  none of the actual *runtime* gamescope behavior is verified — does the
  readiness handshake actually work, does focus really swap when
  `arcade-launcher` launches a game, does the session recover cleanly if
  `arcade-launcher` crashes. Only the Nix-level configuration (does it
  evaluate, does it build) is confirmed.
- **No PR opened yet** — branch exists and is pushed, that's it.
- **Nested-gamescope local testing is built (`nix run .#dev-gamescope` in
  `arcade-launcher-corpo`) but confirmed broken under Hyprland
  specifically** — not a flag issue, a real unresolved upstream gamescope
  bug. Confirmed by hand: even with `--backend wayland` forced (not the
  default `auto`, which was actually picking `headless` - no window at
  all), gamescope's nested window never registers with Hyprland at
  all — checked via both `hyprctl clients` and `hyprctl layers`, neither
  shows it. See https://github.com/ValveSoftware/gamescope/issues/1707
  and related issues (nested gamescope under wlroots/tiling compositors,
  Hyprland specifically called out) - no known fix as of writing. If
  you're not on Hyprland this may just work; if you are, this path is
  currently a dead end and real-hardware testing via `arcade-os` is the
  only way to actually exercise the compositor integration for now.
- No crash-vs-clean-exit distinction yet in `arcade_core::launch` (a
  crashed game and a normal quit look identical to the launcher).
- `arcade-os`'s `configs/steam-boot.nix` still looks like dead/orphaned
  code (references module paths that don't exist in the current tree,
  `flake.nix` never references it) — left alone, not this task's job to
  clean up, but worth a look before it confuses someone.
- `bootstrap.sh` still clones `Project-JoyBoxOS.git`, an old repo name —
  pre-existing inconsistency, unrelated to this change, not touched.
