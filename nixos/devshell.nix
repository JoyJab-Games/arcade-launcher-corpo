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
  ];
}
