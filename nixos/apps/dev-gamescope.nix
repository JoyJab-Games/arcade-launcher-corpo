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
{ pkgs }:
let
  devApp = import ./dev.nix { inherit pkgs; };
in
{
  type = "app";
  meta.description = "Same as .#dev, nested inside gamescope - for testing the real compositor integration without a cabinet";
  program = toString (pkgs.writeShellApplication {
    name = "dev-gamescope";
    runtimeInputs = [ pkgs.gamescope ];
    text = ''
      exec gamescope --steam -- ${devApp.program} "$@"
    '';
  }) + "/bin/dev-gamescope";
}
