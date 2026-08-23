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
let
  rustToolchain = import ../rust-toolchain.nix { inherit pkgs; };
in
{
  type = "app";
  meta.description = "Auto-rebuild the Rust GDExtension on save (cargo watch) and open the Godot editor";
  program = toString (pkgs.writeShellApplication {
    name = "dev";
    # umu-launcher: the editor is what actually launches games while
    # developing (GameRoster.launch_game -> arcade_core::launch), so
    # `umu-run` needs to be on PATH here too, not just in `nix develop` —
    # see rust/core/src/launch.rs.
    #
    # rustToolchain: `cargo watch` (below) dispatches through whatever
    # `cargo` it finds on PATH first — without pinning one here explicitly,
    # that's silently whatever's ambient on the machine running this, which
    # can be a different/incompatible version from the `cargo-watch` build
    # nixpkgs provides. Same toolchain devshell.nix uses, so `nix develop`
    # and `nix run .#dev` can't drift onto different rustc/cargo either.
    runtimeInputs = [ rustToolchain pkgs.cargo-watch pkgs.godot_4 pkgs.umu-launcher ];
    text = ''
      repo_root="$(git rev-parse --show-toplevel)"
      cd "$repo_root"
      export ARCADE_DATA_DIR="$repo_root/.data"

      # `-s` (shell) rather than `-x build` (cargo subcommand): the editor
      # loads the extension from godot/bin/, not straight out of
      # rust/target/ - see rust.gdextension's doc comment on why - so each
      # rebuild needs the fresh .so copied into place too.
      #
      # `cargo-watch` directly, not `cargo watch`: this cargo/cargo-watch
      # pairing has a real dispatch bug where `cargo watch ...` hands
      # cargo-watch a stray leading "watch" argument it then rejects
      # ("Found argument 'watch' which wasn't expected") - calling the
      # binary directly skips cargo's subcommand dispatch entirely.
      #
      # Run from rust/, not repo_root: cargo-watch has no --manifest-path
      # of its own (confirmed against its real --help, unlike cargo itself)
      # - it finds the project by looking for a Cargo.toml in its working
      # directory, which repo_root doesn't have (only rust/ does).
      build_and_copy='cargo build -p arcade-gdext && cp target/debug/libarcade_gdext.so ../godot/bin/debug/libarcade_gdext.so'
      mkdir -p godot/bin/debug

      # First build+copy runs synchronously, before Godot ever starts - not
      # backgrounded like the watch loop below. Godot's very first startup
      # step reads this .so ("Verifying GDExtensions..."); starting it at
      # the same time as the first background build raced Godot against
      # `cp` still writing the file, and reading a partially-written shared
      # library is exactly the kind of thing that segfaults a dynamic
      # loader with no useful backtrace - which is what was happening here.
      (cd rust && eval "$build_and_copy")

      # `--postpone`: the first build above already happened - without
      # this, cargo-watch would immediately kick off a redundant second one
      # on startup.
      (cd rust && cargo-watch --postpone -s "$build_and_copy") &
      watch_pid=$!
      trap 'kill "$watch_pid" 2>/dev/null || true' EXIT

      godot4 --editor godot/project.godot
    '';
  }) + "/bin/dev";
}
