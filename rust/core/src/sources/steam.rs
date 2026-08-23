use std::io::{self, Write};
use std::path::Path;

use serde_json::Value;

use super::steamcmd_session::SteamCmdSession;
use super::{GameMetadata, GameSource, ResolvedGame};
use crate::game_config::GameConfig;

pub struct SteamSource;

impl GameSource for SteamSource {
    fn id(&self) -> &'static str {
        "steam"
    }

    fn resolve_install_target(&self) -> io::Result<ResolvedGame> {
        let appid = prompt("Steam AppID: ")?;
        let branch = prompt("Branch (blank for default): ")?;
        Ok(ResolvedGame {
            suggested_name: appid.clone(),
            source_ref: appid,
            branch: if branch.is_empty() { None } else { Some(branch) },
        })
    }

    /// Real `steamcmd` download: login (username prompted here, password
    /// and any Steam Guard code prompted by steamcmd itself, straight on
    /// the calling terminal), then `app_update` into `dest`, on the branch
    /// `resolve_install_target` picked if any. See SteamCmdSession for how
    /// the login itself is cached/expired across calls.
    fn provision(&self, game: &ResolvedGame, dest: &Path) -> io::Result<()> {
        let username = prompt("Steam username: ")?;

        // steamcmd requires +force_install_dir before +login in the script
        // sequence — putting it after is silently wrong (it warns "Please
        // use force_install_dir before logon!" and ignores it).
        let mut args = vec![
            "+force_install_dir".to_string(),
            dest.to_string_lossy().into_owned(),
            "+login".to_string(),
            username,
            "+app_update".to_string(),
            game.source_ref.clone(),
        ];
        if let Some(branch) = &game.branch {
            args.push("-beta".to_string());
            args.push(branch.clone());
        }
        args.push("validate".to_string());
        args.push("+quit".to_string());

        let session = SteamCmdSession::open(crate::steamcmd_session_dir());
        let arg_refs: Vec<&str> = args.iter().map(String::as_str).collect();
        let status = session.run(&arg_refs)?;
        if !status.success() {
            return Err(io::Error::other(format!("steamcmd exited with status {status}")));
        }
        Ok(())
    }

    // TODO(Phase 4): real update check — compare the installed buildid
    // (steamcmd `app_info_print`/appdetails) against the current one on
    // `game`'s branch.
    fn check_update(&self, game: &GameConfig) -> io::Result<bool> {
        println!("(stub) update check for '{}' not implemented yet", game.name);
        Ok(false)
    }

    /// Best-effort, no HTML scraping: name/description/genres from Steam's
    /// public `appdetails` JSON endpoint (genres aren't the same as the
    /// store page's crowd-sourced tags — there's no official API for those
    /// — but real data with no scraping involved), an image (preferring the
    /// Library Header over the plain store header, see
    /// find_asset_hash/find_linux_executable below), and the Linux
    /// executable path. Any field can come back empty (bad AppID, no
    /// genres listed, no image at all, no Linux build); that's not an
    /// error, the CLI's manual-entry fallback covers tags/image, and
    /// `arcade_core::launch` reports clearly if exec is unset.
    fn fetch_metadata(&self, game: &ResolvedGame) -> io::Result<GameMetadata> {
        let mut metadata = GameMetadata::default();

        if let Ok(body) = crate::http::get_string(&format!(
            "https://store.steampowered.com/api/appdetails?appids={}",
            game.source_ref
        )) {
            if let Ok(root) = serde_json::from_str::<Value>(&body) {
                if let Some(data) = root.get(&game.source_ref).and_then(|entry| entry.get("data")) {
                    metadata.name = data.get("name").and_then(Value::as_str).map(str::to_string);
                    metadata.description = data
                        .get("short_description")
                        .and_then(Value::as_str)
                        .filter(|s| !s.is_empty())
                        .map(str::to_string);
                    if let Some(genres) = data.get("genres").and_then(Value::as_array) {
                        metadata.tags = genres
                            .iter()
                            .filter_map(|g| g.get("description").and_then(Value::as_str))
                            .map(str::to_string)
                            .collect();
                    }
                    // The store header (460x215) — not 16:9, but an
                    // official, always-present asset for any published
                    // app, so it's a solid safety net under the fetch
                    // below.
                    metadata.image_url =
                        data.get("header_image").and_then(Value::as_str).map(str::to_string);
                }
            }
        }

        // Both the Library Header (closer to a real 16:9 box-art asset than
        // header_image) and the Linux executable path live only in Steam's
        // internal "appinfo" data (PICS) — the same source SteamDB itself
        // is built on — accessed here through `steamcmd +app_info_print`, a
        // real Steam CLI command, not scraping a third party. One fetch
        // covers both. Best-effort: silently keeps the header_image
        // fallback above / leaves exec unset if this finds nothing (schema
        // drift, no steamcmd on PATH, anonymous login failing, etc.).
        if let Some(app_info) = fetch_app_info_text(&game.source_ref) {
            if let Some(hash) = find_asset_hash(&app_info, "library_header") {
                metadata.image_url = Some(format!(
                    "https://shared.akamai.steamstatic.com/store_item_assets/steam/apps/{}/{hash}/library_header.jpg",
                    game.source_ref
                ));
            }
            metadata.exec = find_linux_executable(&app_info);
        }

        Ok(metadata)
    }
}

/// Runs `steamcmd +app_info_print <appid>` anonymously (no login/Guard
/// prompt needed — this is public app metadata, the same PICS data every
/// Steam client has local access to) and returns the raw KeyValues (VDF)
/// text it prints. Confirmed against real `app_info_print` output
/// (2026-08-20).
fn fetch_app_info_text(appid: &str) -> Option<String> {
    let session = SteamCmdSession::open(crate::steamcmd_session_dir());
    let output = session
        .run_capturing(&["+login", "anonymous", "+app_info_print", appid, "+quit"])
        .ok()?;
    Some(String::from_utf8_lossy(&output.stdout).into_owned())
}

/// Finds the content hash for a `library_assets_full` entry, e.g.
/// `"library_header" { "image" { "english" "<hash>/library_header.jpg" } }`.
///
/// Valve's appinfo VDF mentions each library asset name twice: once as a
/// flag in `library_assets` (`"library_header"  "en"` — a locale marker,
/// no hash), and once as the real object in `library_assets_full`. Only
/// the second, `{`-shaped occurrence has a usable hash after it, so this
/// walks past `key`-then-string occurrences and only reads a hash out of
/// the first `key`-then-`{` occurrence. Deliberately a lenient scan rather
/// than a full VDF parser — Valve's appinfo schema has shifted before, and
/// this only needs to survive that, not represent it faithfully.
fn find_asset_hash<'a>(text: &'a str, key: &str) -> Option<&'a str> {
    let quoted_key = format!("\"{key}\"");
    let mut rest = text;
    loop {
        let after_key = rest.split_once(&quoted_key)?.1;
        if after_key.trim_start().starts_with('{') {
            return after_key.split(|c: char| !c.is_ascii_hexdigit()).find(|token| token.len() == 40);
        }
        rest = after_key;
    }
}

/// Finds the Linux executable path from an appinfo VDF's `config.launch`
/// entries, e.g. `"1" { "executable" "Foo.x86_64" "config" { "oslist"
/// "linux" } }` — one entry per platform, per Valve's schema.
///
/// Same lenient-scan approach as find_asset_hash: walks `"executable"`
/// occurrences in order; for each, the text up to the *next* `"executable"`
/// occurrence is that same launch entry's own scope (Valve always emits an
/// entry's oslist before the next entry's executable), so the first
/// `"oslist"` found within that scope belongs to it. Returns the first
/// entry whose oslist mentions "linux".
fn find_linux_executable(text: &str) -> Option<String> {
    let mut rest = text;
    loop {
        let after_exec = rest.split_once("\"executable\"")?.1;
        let entry_scope = after_exec.split_once("\"executable\"").map_or(after_exec, |(scope, _)| scope);

        if let Some(oslist_pos) = entry_scope.find("\"oslist\"") {
            // Skip past the "oslist" key itself, or quoted_token_after
            // would just read the key's own closing quote back as "the
            // first quoted token" instead of its value.
            let after_oslist_key = &entry_scope[oslist_pos + "\"oslist\"".len()..];
            if let Some(oslist) = quoted_token_after(after_oslist_key) {
                if oslist.contains("linux") {
                    return quoted_token_after(after_exec).map(str::to_string);
                }
            }
        }

        rest = after_exec;
    }
}

/// The first quoted string in `text` (its own quotes not included).
fn quoted_token_after(text: &str) -> Option<&str> {
    let start = text.find('"')? + 1;
    let len = text[start..].find('"')?;
    Some(&text[start..start + len])
}

fn prompt(label: &str) -> io::Result<String> {
    print!("{label}");
    io::stdout().flush()?;
    let mut line = String::new();
    io::stdin().read_line(&mut line)?;
    Ok(line.trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    // Condensed from real `steamcmd +app_info_print 3900090` output
    // (2026-08-20) — trimmed to the parts find_asset_hash cares about, but
    // keeping both "library_header" occurrences in their real order/shape,
    // since that's exactly what caught the original off-by-one bug (an
    // earlier version picked "library_capsule"'s hash instead).
    const APPINFO_FIXTURE: &str = r#"
        "common"
        {
            "library_assets"
            {
                "library_capsule"		"en"
                "library_hero"		"en"
                "library_header"		"en"
                "logo_position"
                {
                    "pinned_position"		"CenterCenter"
                }
            }
            "library_assets_full"
            {
                "library_capsule"
                {
                    "image"
                    {
                        "english"		"926dff5ad6bce3be1b7c5def9ee88102cff0e037/library_capsule.jpg"
                    }
                }
                "library_header"
                {
                    "image"
                    {
                        "english"		"6b36212f27a1173c0c00be6399be10c9afc55ef3/library_header.jpg"
                    }
                    "image2x"
                    {
                        "english"		"6b36212f27a1173c0c00be6399be10c9afc55ef3/library_header_2x.jpg"
                    }
                }
            }
        }
    "#;

    #[test]
    fn find_asset_hash_skips_the_locale_flag_and_reads_the_real_object() {
        assert_eq!(
            find_asset_hash(APPINFO_FIXTURE, "library_header"),
            Some("6b36212f27a1173c0c00be6399be10c9afc55ef3")
        );
    }

    #[test]
    fn find_asset_hash_does_not_pick_up_a_different_assets_hash() {
        // The bug this guards against: naively taking the text after the
        // *first* occurrence of the key landed inside "library_capsule"'s
        // object instead, whose hash is deliberately different here.
        let hash = find_asset_hash(APPINFO_FIXTURE, "library_header").unwrap();
        assert_ne!(hash, "926dff5ad6bce3be1b7c5def9ee88102cff0e037");
    }

    #[test]
    fn find_asset_hash_returns_none_for_a_missing_key() {
        assert_eq!(find_asset_hash(APPINFO_FIXTURE, "library_hero"), None);
    }

    // From the same real `steamcmd +app_info_print 3900090` output.
    const LAUNCH_FIXTURE: &str = r#"
        "config"
        {
            "launch"
            {
                "0"
                {
                    "executable"		"WummsenVillage.exe"
                    "config"
                    {
                        "oslist"		"windows"
                    }
                }
                "1"
                {
                    "executable"		"WummsenVillage.x86_64"
                    "config"
                    {
                        "oslist"		"linux"
                    }
                }
                "2"
                {
                    "executable"		"WummsenVillage.app/Contents/MacOS/WummsenVillage"
                    "config"
                    {
                        "oslist"		"macos"
                    }
                }
            }
        }
    "#;

    #[test]
    fn find_linux_executable_picks_the_linux_entry_not_the_first_one() {
        assert_eq!(
            find_linux_executable(LAUNCH_FIXTURE),
            Some("WummsenVillage.x86_64".to_string())
        );
    }

    #[test]
    fn find_linux_executable_returns_none_without_a_linux_entry() {
        let windows_only = r#"
            "launch"
            {
                "0"
                {
                    "executable"		"Foo.exe"
                    "config"
                    {
                        "oslist"		"windows"
                    }
                }
            }
        "#;
        assert_eq!(find_linux_executable(windows_only), None);
    }
}
