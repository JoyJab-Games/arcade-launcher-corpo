# Builds the whole rust/ workspace in release mode once — arcade-gdext's
# cdylib (godot/package.nix's export needs it) and the `arcade` CLI binary
# (cli-package.nix wraps it) share this one build rather than compiling
# twice. `doCheck` (default true) runs the workspace's own `cargo test` as
# part of the build, so a broken test fails the package build the same way
# it'd fail CI — every test added so far is filesystem/tempdir-only, no
# network, so this works offline in the Nix sandbox too.
{ pkgs }:
let
  rustToolchain = import ./rust-toolchain.nix { inherit pkgs; };
  rustPlatform = pkgs.makeRustPlatform {
    cargo = rustToolchain;
    rustc = rustToolchain;
  };
in
rustPlatform.buildRustPackage {
  pname = "arcade-workspace";
  version = "0.1.0";
  src = ../rust;

  cargoLock = {
    lockFile = ../rust/Cargo.lock;
    # gamescope-x11-client (see rust/core/src/gamescope.rs) is a git
    # dependency, not on crates.io - importCargoLock needs an explicit
    # vendoring hash for it, the same way a fixed-output derivation needs
    # one for any non-crates.io source.
    outputHashes = {
      "gamescope-x11-client-0.1.0" = "sha256-nNd1fCRw9qsP/QM7ks/vIpyMsmMF7MJVLBKNjpLyr7I=";
    };
  };

  # buildRustPackage's default installPhase only knows how to pick up
  # `[[bin]]` outputs (the `arcade` CLI) — it would silently drop
  # arcade-gdext's cdylib entirely, which is the whole reason this
  # derivation exists. Install both explicitly instead.
  #
  # `find` rather than a hardcoded `target/release/...` path: buildRustPackage
  # builds against an explicit target triple, which puts real output under
  # `target/<triple>/release/` instead of cargo's own no-`--target` shortcut.
  installPhase = ''
    runHook preInstall
    mkdir -p $out/bin $out/lib
    cp "$(find target -type f -path '*/release/arcade')" $out/bin/
    cp "$(find target -type f -path '*/release/libarcade_gdext.so')" $out/lib/
    runHook postInstall
  '';
}
