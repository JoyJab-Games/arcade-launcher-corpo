# `nix run .#dev-gamescope` — exactly `nix run .#dev` (cargo-watch +
# the Godot editor), run inside a gamescope session. Enough to exercise
# the *entire* real compositor integration (rust/core/src/gamescope.rs —
# window tagging, baselayer focus swap on game launch/exit, the in-game
# overlay) for real, on a plain dev machine, without touching
# arcade-os/greetd/real cabinet hardware at all. See
# docs/TODO-arcade-os-session.md.
#
# Picks gamescope's backend at runtime based on how it's invoked:
#
# - From an ordinary desktop terminal (WAYLAND_DISPLAY or DISPLAY set):
#   `--backend wayland` - gamescope runs as a nested Wayland *client*
#   inside whatever compositor the terminal is already in, the same way
#   people test gamescope under GNOME/Sway on a normal desktop.
#   KNOWN BROKEN under Hyprland specifically, confirmed by hand: even
#   with --backend wayland forced (plain `auto` actually picked
#   gamescope's `headless` backend instead - no window at all - in a
#   nested Hyprland session), gamescope's nested window never registers
#   with Hyprland at all (checked via both `hyprctl clients` and
#   `hyprctl layers` - neither shows it, so this isn't
#   "hidden/unfocused", nothing is created). Real, unresolved upstream
#   issue, not fixable from here - see
#   https://github.com/ValveSoftware/gamescope/issues/1707 (nested
#   gamescope under wlroots/tiling compositors, Hyprland specifically
#   called out, no known fix as of writing).
#
# - From a bare TTY with neither set (switch to one with Ctrl+Alt+F<n>,
#   log in there, `cd` into the repo, run this): `--backend drm` -
#   gamescope's genuine embedded mode, taking DRM/KMS directly - the
#   *same mode the real cabinet uses* (see gamescope-session.sh in
#   arcade-os). VT-switch/DRM-master handoff itself works fine (Hyprland
#   correctly releases DRM master on switch-away, confirmed by hand) and
#   gamescope's connector/EDID/mode detection all succeed for real -
#   but on this dev machine's Intel Arc A750 (DG2, i915 driver),
#   ALSO CONFIRMED BROKEN: every framebuffer submission fails with
#   `drmModeAddFB2WithModifiers failed: Invalid argument`, so nothing
#   ever reaches the screen (Xwayland/Godot/pipewire all still come up
#   looking healthy internally - a silent black screen, not a crash).
#   --force-composition (below) does not fix it: that only stops
#   *client* buffers from going straight to a plane, but gamescope's own
#   composited backbuffer submission hits the identical AddFB2 failure,
#   so this is gamescope failing to negotiate a working DRM format
#   modifier with this driver/kernel combo at all, not a client-specific
#   issue. Matches a real, unresolved upstream bug on Intel iGPUs launched
#   from a TTY - see https://github.com/ValveSoftware/gamescope/issues/1738
#   (open since Feb 2025, no fix, no workaround from maintainers) - not
#   fixable from here. The real cabinet's GPU is AMD, not Intel (see
#   docs/TODO-arcade-os-session.md), so this is plausibly a dev-machine-only
#   dead end rather than something that'll recur on real hardware, but
#   that's untested - real-hardware testing via arcade-os remains the
#   only confirmed-working way to exercise this for now.
#
# `--steam`: not literal Steam - this is what turns on gamescope's
# X11-atom control-protocol surface (GAMESCOPECTRL_BASELAYER_APPID,
# STEAM_GAME, etc.) gamescope.rs depends on. Same flag arcade-os's real
# session passes, for the same reason — see
# modules/roles/arcade-boot/gamescope-session.sh over there.
#
# -W/-H/-f: without an explicit size, gamescope guesses (observed: it
# picked a handheld-shaped 800x1280 profile with nothing to base a real
# guess on). Sized to match project.godot's own viewport (1920x1080) and
# forced fullscreen for an unambiguous, correctly-sized window.
#
# --force-composition: disables gamescope's direct scan-out (handing a
# client's buffer straight to a DRM plane, bypassing its own
# compositing). Needed on this machine's Intel Arc A750 (DG2) in
# --backend drm mode - without it, DRM init completes fine (real
# connectors/EDID/mode all detected correctly) but every actual
# framebuffer submission fails with `drmModeAddFB2WithModifiers failed:
# Invalid argument`, so nothing ever reaches the screen even though
# Xwayland/Godot/pipewire all come up looking healthy - a silent black
# screen, not a crash. Root cause is gamescope negotiating a DRM
# format/modifier combo this GPU's driver won't accept for direct
# scan-out; forcing composition sidesteps that by always presenting a
# framebuffer gamescope itself controls. Only meaningful for --backend
# drm (nested --backend wayland never does direct scan-out anyway) but
# harmless to pass either way.
{ pkgs }:
let
  devApp = import ./dev.nix { inherit pkgs; };
in
{
  type = "app";
  meta.description = "Same as .#dev, inside gamescope - nested (desktop terminal) or embedded/DRM (bare TTY) depending on how it's invoked; both currently broken on this dev machine for unrelated reasons, see comments";
  program = toString (pkgs.writeShellApplication {
    name = "dev-gamescope";
    runtimeInputs = [ pkgs.gamescope ];
    text = ''
      backend=drm
      if [ -n "''${WAYLAND_DISPLAY:-}" ] || [ -n "''${DISPLAY:-}" ]; then
        backend=wayland
      fi
      echo "dev-gamescope: using --backend $backend" >&2
      exec gamescope --steam --backend "$backend" --force-composition -W 1920 -H 1080 -f -- ${devApp.program} "$@"
    '';
  }) + "/bin/dev-gamescope";
}
