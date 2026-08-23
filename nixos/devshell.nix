# Dev environment: `nix develop` from the repo root.
# Rust version is pinned via rust-overlay — `stable.latest` resolves against
# whatever rust-overlay revision flake.lock is pinned to, so it's fixed and
# reproducible until someone deliberately runs `nix flake update`.
{ pkgs }:
let
  rustToolchain = pkgs.rust-bin.stable.latest.default.override {
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
  ];
}
