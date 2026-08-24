# Dev environment: `nix develop` from the repo root.
{ pkgs }:
let
  rustToolchain = (import ./rust-toolchain.nix { inherit pkgs; }).override {
    extensions = [ "rust-src" "rust-analyzer" ];
  };
in
pkgs.mkShell {
  packages = [
    rustToolchain
    pkgs.godot_4
    pkgs.cargo-watch
    # `arcade install`/`update` shell out to this for real Steam depot
    # downloads (see rust/core/src/sources/steam.rs). Unfree-licensed —
    # config.allowUnfree is set where `pkgs` is constructed, see default.nix.
    pkgs.steamcmd
    # `arcade_core::launch` shells out to `umu-run` (its `PROTONPATH` set to
    # a build `arcade install`/`update` predownloaded via steamcmd) to
    # launch Proton/Windows games — see rust/core/src/launch.rs.
    pkgs.umu-launcher
    # For manually testing the gamescope compositor integration
    # (rust/core/src/gamescope.rs) nested inside a normal desktop session —
    # see apps/dev-gamescope.nix, which wraps this for you; kept here too
    # for ad-hoc `gamescope -- ...` use.
    pkgs.gamescope
  ];
}
