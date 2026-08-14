# `nix run .#dev` — starts `cargo watch` in the background so the Rust
# GDExtension gets rebuilt on save, then opens the Godot editor in the
# foreground. `rust.gdextension` has `reloadable = true`, so the editor
# hot-reloads the extension automatically once cargo watch finishes a
# rebuild — no manual `cargo build` step needed during normal development.
# Killing/closing the editor stops the watch process too.
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

      cargo watch --manifest-path rust/Cargo.toml -x build &
      watch_pid=$!
      trap 'kill "$watch_pid" 2>/dev/null || true' EXIT

      godot4 --editor godot/project.godot
    '';
  }) + "/bin/dev";
}
