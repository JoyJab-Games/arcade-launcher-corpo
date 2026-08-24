# `nix run .#dev-gamescope` — exactly `nix run .#dev` (cargo-watch +
# the Godot editor), just nested inside a gamescope session.
#
# Gamescope has a genuine nested mode: run as an ordinary Wayland client
# inside any other already-running compositor, the same way people test
# gamescope under GNOME/Sway on a normal desktop. That means this is
# enough to exercise the *entire* real compositor integration
# (rust/core/src/gamescope.rs — window tagging, baselayer focus swap on
# game launch/exit, the in-game overlay) for real, on a plain dev
# machine, without touching arcade-os/greetd/embedded mode/real cabinet
# hardware at all — embedded vs. nested only changes who owns DRM/KMS,
# not the X11-atom control protocol gamescope.rs actually talks to. See
# docs/TODO-arcade-os-session.md.
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
# --backend wayland: explicit rather than the default `auto`, which in
# testing actually picked gamescope's `headless` backend (no window at
# all) in a nested Hyprland session - forcing `wayland` is still the more
# correct choice in general even though it turned out not to be the whole
# story here (see below).
#
# KNOWN BROKEN under Hyprland specifically, confirmed by hand: even with
# --backend wayland forced, gamescope's nested window never registers with
# Hyprland at all (checked via both `hyprctl clients` and `hyprctl
# layers` - neither shows it, so this isn't "hidden/unfocused", nothing is
# created). This is a real, unresolved upstream issue, not something
# fixable from here - see
# https://github.com/ValveSoftware/gamescope/issues/1707 and related
# issues (nested gamescope under wlroots/tiling compositors, Hyprland
# specifically called out, no known fix as of writing). If you're not on
# Hyprland this may well just work; if you are, this app is currently a
# dead end for local testing - fall back to testing on real hardware via
# arcade-os instead (see docs/TODO-arcade-os-session.md).
{ pkgs }:
let
  devApp = import ./dev.nix { inherit pkgs; };
in
{
  type = "app";
  meta.description = "Same as .#dev, nested inside gamescope - for testing the real compositor integration without a cabinet (currently broken under Hyprland, see comments)";
  program = toString (pkgs.writeShellApplication {
    name = "dev-gamescope";
    runtimeInputs = [ pkgs.gamescope ];
    text = ''
      exec gamescope --steam --backend wayland -W 1920 -H 1080 -f -- ${devApp.program} "$@"
    '';
  }) + "/bin/dev-gamescope";
}
