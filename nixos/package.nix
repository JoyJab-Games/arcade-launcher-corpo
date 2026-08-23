# The actual thing a cabinet runs: a headless `godot4 --export-release`
# build of godot/, with arcade-gdext's cdylib (see rust-build.nix) copied
# into godot/bin/ first — rust.gdextension reads from there rather than
# straight out of rust/target/, since only a path inside the project's own
# res:// tree survives being exported at all (see its doc comment).
{ pkgs }:
let
  workspace = import ./rust-build.nix { inherit pkgs; };
in
pkgs.stdenv.mkDerivation {
  pname = "arcade-launcher";
  version = "0.1.0";
  src = ../godot;

  nativeBuildInputs = [ pkgs.godot_4 pkgs.makeWrapper ];

  buildPhase = ''
    runHook preBuild

    # Godot only ever looks for export templates under $HOME - point it at
    # a throwaway one in the sandbox rather than a real user's cache.
    # Symlinking whatever version directory(ies) godot_4-export-templates-bin
    # actually has, rather than hardcoding the expected name ourselves:
    # Godot's own "x.y-stable" package version string doesn't literally
    # match its "x.y.stable" template-folder naming, and re-deriving that
    # transform by hand is just one more thing to get wrong/out of sync
    # whenever pkgs.godot_4 bumps versions.
    export HOME="$TMPDIR/home"
    mkdir -p "$HOME/.local/share/godot/export_templates"
    for template_dir in ${pkgs.godot_4-export-templates-bin}/share/godot/export_templates/*; do
      ln -s "$template_dir" "$HOME/.local/share/godot/export_templates/$(basename "$template_dir")"
    done

    # Both debug and release paths need to resolve even though only
    # release is ever used here, or Godot logs (non-fatal) errors trying
    # to validate the debug entry too.
    mkdir -p bin/debug bin/release
    cp ${workspace}/lib/libarcade_gdext.so bin/release/libarcade_gdext.so
    cp ${workspace}/lib/libarcade_gdext.so bin/debug/libarcade_gdext.so

    mkdir -p builds
    godot4 --headless --export-release "Linux" "$(pwd)/builds/arcade-launcher.x86_64" --path .

    runHook postBuild
  '';

  installPhase = ''
    runHook preInstall

    mkdir -p $out/share/arcade-launcher
    cp builds/arcade-launcher.x86_64 builds/libarcade_gdext.so $out/share/arcade-launcher/

    # umu-run (Proton games, see rust/core/src/launch.rs) and steamcmd
    # (still shelled out to for the appinfo-based Linux/Windows exec
    # detection at install/update time, see rust/core/src/sources/steam.rs)
    # both need to be on PATH wherever this actually runs, not just in
    # `nix develop`/`nix run .#dev`.
    makeWrapper $out/share/arcade-launcher/arcade-launcher.x86_64 $out/bin/arcade-launcher \
      --prefix PATH : ${pkgs.lib.makeBinPath [ pkgs.steamcmd pkgs.umu-launcher ]}

    runHook postInstall
  '';
}
