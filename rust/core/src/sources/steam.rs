use std::io::{self, Write};
use std::path::Path;

use super::{GameSource, ResolvedGame};
use crate::game_config::GameConfig;

pub struct SteamSource;

impl GameSource for SteamSource {
    fn id(&self) -> &'static str {
        "steam"
    }

    fn resolve_install_target(&self) -> io::Result<ResolvedGame> {
        print!("Steam AppID: ");
        io::stdout().flush()?;
        let mut appid = String::new();
        io::stdin().read_line(&mut appid)?;
        let appid = appid.trim().to_string();
        Ok(ResolvedGame {
            suggested_name: appid.clone(),
            source_ref: appid,
        })
    }

    // TODO(Phase 4): actual DepotDownloader subprocess (login/Steam Guard,
    // progress parsing, abort handling). For now this just creates the
    // destination directory, so the rest of the install flow (GameStore.add,
    // dest path handling) is already testable end-to-end.
    fn provision(&self, game: &ResolvedGame, dest: &Path) -> io::Result<()> {
        println!(
            "(stub) would download Steam AppID {} into {}",
            game.source_ref,
            dest.display()
        );
        std::fs::create_dir_all(dest)
    }

    // TODO(Phase 4): real update check via DepotDownloader/Steam API.
    fn check_update(&self, game: &GameConfig) -> io::Result<bool> {
        println!("(stub) update check for '{}' not implemented yet", game.name);
        Ok(false)
    }
}
