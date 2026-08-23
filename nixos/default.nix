# Assembles all flake outputs. This is the only file flake.nix imports —
# every other .nix file in this directory (or nixosModules elsewhere) is
# wired together from here. Packages and the NixOS module land in later
# phases; for now this just exposes the dev shell and the `dev` app.
{ nixpkgs, rust-overlay, ... }:
let
  system = "x86_64-linux";
  pkgs = import nixpkgs {
    inherit system;
    overlays = [ rust-overlay.overlays.default ];
    # steamcmd (rust/core's real Steam download path, see devshell.nix) is
    # unfree-licensed.
    config.allowUnfree = true;
  };
in
{
  devShells.${system}.default = import ./devshell.nix { inherit pkgs; };
  apps.${system} = {
    dev = import ./apps/dev.nix { inherit pkgs; };
    cli = import ./apps/cli.nix { inherit pkgs; };
  };
}
