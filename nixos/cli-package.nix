# The `arcade` admin CLI as a real installable package, for the deployed
# cabinet (`apps/cli.nix`'s `nix run .#cli` is the dev-only equivalent —
# `cargo run` against a project-local `.data/` dir, see its own comment).
{ pkgs }:
let
  workspace = import ./rust-build.nix { inherit pkgs; };
in
pkgs.stdenv.mkDerivation {
  pname = "arcade-cli";
  version = "0.1.0";
  dontUnpack = true;

  nativeBuildInputs = [ pkgs.makeWrapper ];

  installPhase = ''
    runHook preInstall
    mkdir -p $out/bin
    # steamcmd: `arcade install`/`update` shell out to it — see
    # rust/core/src/sources/steam.rs and sources/proton.rs.
    makeWrapper ${workspace}/bin/arcade $out/bin/arcade \
      --prefix PATH : ${pkgs.lib.makeBinPath [ pkgs.steamcmd ]}
    runHook postInstall
  '';
}
