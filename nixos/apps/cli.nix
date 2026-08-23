# `nix run .#cli -- <args>` — runs the `arcade` admin CLI (source rebuilds
# via `cargo run`, so no separate build step) against the same
# project-local, gitignored `.data/` folder that `nix run .#dev` (see
# ../apps/dev.nix) points the editor at. Arguments after `--` are forwarded
# to the `arcade` binary, e.g.:
#   nix run .#cli -- install
#   nix run .#cli -- list
{ pkgs }:
let
  rustToolchain = pkgs.rust-bin.stable.latest.default;
in
{
  type = "app";
  meta.description = "Run the arcade admin CLI against the project-local dev data dir";
  program = toString (pkgs.writeShellApplication {
    name = "cli";
    # steamcmd: `arcade install`/`update` shell out to it for real Steam
    # depot downloads (see rust/core/src/sources/steam.rs) — must be on
    # PATH here too, not just in the `nix develop` shell.
    runtimeInputs = [ rustToolchain pkgs.steamcmd ];
    text = ''
      repo_root="$(git rev-parse --show-toplevel)"
      export ARCADE_DATA_DIR="$repo_root/.data"
      cargo run --quiet --manifest-path "$repo_root/rust/Cargo.toml" -p arcade-cli -- "$@"
    '';
  }) + "/bin/cli";
}
