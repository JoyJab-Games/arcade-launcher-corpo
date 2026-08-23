# `nix run .#dev` — starts `cargo watch` in the background so the Rust
# GDExtension gets rebuilt on save, then opens the Godot editor in the
# foreground. `rust.gdextension` has `reloadable = true`, so the editor
# hot-reloads the extension automatically once cargo watch finishes a
# rebuild — no manual `cargo build` step needed during normal development.
# Killing/closing the editor stops the watch process too.
#
# `ARCADE_DATA_DIR` is pointed at a project-local, gitignored `.data/`
# folder so editor runs never touch a real `~/.local/share/arcade-launcher`
# on the dev machine. `nix run .#cli` (see ../apps/cli.nix) points at the
# same folder, so installs made via the CLI and state read in the editor
# agree on one place.
{ pkgs }:
{
  type = "app";
  meta.description = "Auto-rebuild the Rust GDExtension on save (cargo watch) and open the Godot editor";
  program = toString (pkgs.writeShellApplication {
    name = "dev";
    runtimeInputs = [ pkgs.cargo-watch pkgs.godot_4 ];
    text = ''
      repo_root="$(git rev-parse --show-toplevel)"
      cd "$repo_root"
      export ARCADE_DATA_DIR="$repo_root/.data"

      cargo watch --manifest-path rust/Cargo.toml -x build &
      watch_pid=$!
      trap 'kill "$watch_pid" 2>/dev/null || true' EXIT

      godot4 --editor godot/project.godot
    '';
  }) + "/bin/dev";
}
