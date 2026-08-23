use std::io;
use std::path::Path;
use std::process::{Child, Command};

use crate::game_config::GameConfig;

/// Starts `game`'s process, detached — returns as soon as it's spawned,
/// does not wait for it to exit.
///
/// TODO(Phase 1): Proton/`umu-run` wrapping for `game.proton` games (stubbed
/// as an error below, not silently guessed at), process monitoring + crash
/// auto-restart, and the actual on-cabinet focus handover to the running
/// game — that last one is blocked on the still-open compositor choice
/// (cage vs gamescope, see arcade-launcher open questions).
pub fn launch(game: &GameConfig) -> io::Result<Child> {
    spawn_in(game, &crate::game_data_dir(&game.name))
}

fn spawn_in(game: &GameConfig, dir: &Path) -> io::Result<Child> {
    let exec = game
        .exec
        .as_ref()
        .ok_or_else(|| io::Error::other(format!("'{}' has no executable set", game.name)))?;

    if game.proton {
        return Err(io::Error::other(
            "launching a Proton/Windows game isn't implemented yet (needs umu-run wrapping)",
        ));
    }

    let path = dir.join(exec);

    // steamcmd-downloaded Linux binaries don't reliably come out with the
    // executable bit set - make sure it's there rather than failing with a
    // confusing "Permission denied" on every fresh install.
    #[cfg(unix)]
    make_executable(&path)?;

    Command::new(&path).current_dir(dir).spawn()
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
        let err = spawn_in(&sample("half-life"), dir.path()).unwrap_err();
        assert!(err.to_string().contains("no executable set"));
    }

    #[test]
    fn errors_on_a_proton_game_instead_of_guessing() {
        let dir = tempfile::tempdir().unwrap();
        let mut game = sample("half-life");
        game.exec = Some("hl.exe".to_string());
        game.proton = true;
        let err = spawn_in(&game, dir.path()).unwrap_err();
        assert!(err.to_string().contains("umu-run"));
    }
}
