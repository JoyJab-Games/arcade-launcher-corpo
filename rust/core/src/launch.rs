use std::io;
use std::path::{Path, PathBuf};
use std::process::{Child, Command};

use crate::game_config::GameConfig;
use crate::gamescope;
use crate::sources::proton;

/// Starts `game`'s process — returns as soon as it's spawned and gamescope
/// focus handover (see `gamescope::spawn_monitor_and_focus`) is handed off
/// to a background thread, not once the game exits.
///
/// TODO(Phase 1): crash auto-restart — `spawn_monitor_and_focus` already
/// knows exactly when the game's process exits (that's what lets it hand
/// focus back to the launcher), it just doesn't do anything with a crash
/// vs. a normal quit yet.
pub fn launch(game: &GameConfig) -> io::Result<()> {
    let child = spawn_in(game, &crate::game_data_dir(&game.name), &crate::proton_dir())?;
    gamescope::spawn_monitor_and_focus(child);
    Ok(())
}

fn spawn_in(game: &GameConfig, dir: &Path, proton_root: &Path) -> io::Result<Child> {
    let exec = game
        .exec
        .as_ref()
        .ok_or_else(|| io::Error::other(format!("'{}' has no executable set", game.name)))?;
    let path = dir.join(exec);

    if game.proton {
        return spawn_via_proton(game, &path, dir, proton_root);
    }

    // steamcmd-downloaded Linux binaries don't reliably come out with the
    // executable bit set - make sure it's there rather than failing with a
    // confusing "Permission denied" on every fresh install.
    #[cfg(unix)]
    make_executable(&path)?;

    Command::new(&path).current_dir(dir).spawn()
}

/// Hands a Windows executable off to `umu-run` under the Proton build
/// `game` was installed against. That build is expected to already be on
/// disk — `arcade install`/`update` predownload it (see
/// `sources::proton::provision`) precisely so this never has to fetch
/// anything itself; a missing build here means the manifest is stale/was
/// hand-edited, not a normal first-launch case.
fn spawn_via_proton(game: &GameConfig, exe_path: &Path, dir: &Path, proton_root: &Path) -> io::Result<Child> {
    let version = game.proton_version.as_deref().unwrap_or(proton::DEFAULT_VERSION);
    let proton_dir = proton::dir(proton_root, version).ok_or_else(|| {
        io::Error::other(format!("unknown Proton version '{version}' set on '{}'", game.name))
    })?;
    if !proton_dir.exists() {
        return Err(io::Error::other(format!(
            "Proton build '{version}' for '{}' isn't downloaded yet — run `arcade update {}` to fetch it",
            game.name, game.name
        )));
    }

    // Own, persistent prefix per game rather than umu's shared default, so
    // a game's save data/registry tweaks survive across launches and never
    // collide with another Proton game's prefix.
    let prefix = game
        .prefix_path
        .as_ref()
        .map(PathBuf::from)
        .unwrap_or_else(|| crate::game_dir(&game.name).join("prefix"));

    Command::new("umu-run")
        .env("PROTONPATH", &proton_dir)
        .env("GAMEID", "umu-default")
        .env("WINEPREFIX", &prefix)
        .arg(exe_path)
        .current_dir(dir)
        .spawn()
}

#[cfg(unix)]
fn make_executable(path: &Path) -> io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let mut perms = std::fs::metadata(path)?.permissions();
    perms.set_mode(perms.mode() | 0o111);
    std::fs::set_permissions(path, perms)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample(name: &str) -> GameConfig {
        GameConfig {
            name: name.to_string(),
            source: "steam".to_string(),
            source_ref: "70".to_string(),
            branch: None,
            exec: None,
            proton: false,
            proton_version: None,
            prefix_path: None,
            released_for_players: false,
            description: None,
            tags: vec![],
            image_path: None,
        }
    }

    #[test]
    fn errors_with_no_executable_set() {
        let dir = tempfile::tempdir().unwrap();
        let proton_root = tempfile::tempdir().unwrap();
        let err = spawn_in(&sample("half-life"), dir.path(), proton_root.path()).unwrap_err();
        assert!(err.to_string().contains("no executable set"));
    }

    #[test]
    fn errors_on_an_unknown_pinned_proton_version() {
        let dir = tempfile::tempdir().unwrap();
        let proton_root = tempfile::tempdir().unwrap();
        let mut game = sample("half-life");
        game.exec = Some("hl.exe".to_string());
        game.proton = true;
        game.proton_version = Some("Some Made Up Version".to_string());
        let err = spawn_in(&game, dir.path(), proton_root.path()).unwrap_err();
        assert!(err.to_string().contains("unknown Proton version"));
    }

    #[test]
    fn errors_when_the_pinned_proton_build_isnt_downloaded_yet() {
        let dir = tempfile::tempdir().unwrap();
        let proton_root = tempfile::tempdir().unwrap(); // empty - nothing provisioned
        let mut game = sample("half-life");
        game.exec = Some("hl.exe".to_string());
        game.proton = true; // proton_version left None -> defaults to DEFAULT_VERSION
        let err = spawn_in(&game, dir.path(), proton_root.path()).unwrap_err();
        assert!(err.to_string().contains("isn't downloaded yet"));
    }

    #[test]
    fn hands_off_to_umu_run_once_the_proton_build_is_present() {
        let dir = tempfile::tempdir().unwrap();
        let proton_root = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(proton_root.path().join("1493710")).unwrap(); // Proton Experimental's AppID
        let mut game = sample("half-life");
        game.exec = Some("hl.exe".to_string());
        game.proton = true;

        // umu-run isn't installed in this environment, so spawn() itself
        // fails - but that's a real "umu-run is missing" error from the
        // OS, not our own "isn't downloaded yet" guard, which confirms we
        // got past our own checks and actually tried to hand off to it.
        let err = spawn_in(&game, dir.path(), proton_root.path()).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::NotFound);
    }
}
