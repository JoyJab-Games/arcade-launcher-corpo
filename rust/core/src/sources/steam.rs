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
        ];

        // steamcmd otherwise auto-detects which platform's depot to grab
        // off the host it's running on (always Linux here) - fine for a
        // game with a native Linux build, but fatal ("ERROR! Failed to
        // install app '<id>' (Invalid platform)") for a Windows-only game,
        // which has no Linux depot for it to fall back to. fetch_metadata
        // already detects exactly this (find_linux_executable vs
        // find_windows_executable, see its doc comment) to decide
        // GameMetadata.proton, but that result never reaches provision(),
        // so re-derive it here rather than forcing windows unconditionally
        // (which would wrongly skip a native Linux depot when one exists).
        if let Some(app_info) = fetch_app_info_text(&game.source_ref) {
            if let Some(platform) = platform_override(&app_info) {
                args.push("+@sSteamCmdForcePlatformType".to_string());
                args.push(platform.to_string());
            }
        }

        args.push("+app_update".to_string());
        args.push(game.source_ref.clone());
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
    /// find_asset_hash/find_linux_executable below), and the executable
    /// path — Linux if there's a native build, else Windows (which also
    /// sets `proton`, see GameMetadata's doc comment). Any field can come
    /// back empty (bad AppID, no genres listed, no image at all, no build
    /// for either platform); that's not an error, the CLI's manual-entry
    /// fallback covers tags/image, and `arcade_core::launch` reports
    /// clearly if exec is unset.
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
            if let Some(exec) = find_linux_executable(&app_info) {
                metadata.exec = Some(exec);
                metadata.proton = Some(false);
            } else if let Some(exec) = find_windows_executable(&app_info) {
                metadata.exec = Some(exec);
                metadata.proton = Some(true);
            }
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

/// Which platform, if any, `provision()` needs to force steamcmd onto via
/// `@sSteamCmdForcePlatformType` for `text` (`app_info_print` output) to
/// pick a depot at all.
///
/// Deliberately reads `common.oslist` (a comma-separated list like
/// `"windows,macos,linux"`, Valve's own top-level "which platforms does
/// this app support at all" field) rather than a launch entry's own oslist
/// (config.launch.N.config, see find_linux_executable/find_windows_executable
/// above) or a depot's own oslist (depots.<id>.config) - the launch entry
/// one only matters for choosing which *executable* to run after install
/// and isn't reliably present at all for a single-depot title (confirmed
/// against a real single-depot Windows-only demo, AppID 4191940,
/// 2026-08-27 - it has zero per-launch-entry or per-depot oslist tags,
/// only common.oslist), and a title's depot(s) aren't all individually
/// tagged either unless it actually ships more than one (see AppID
/// 3900090's `depots.3900091.config.oslist`/`depots.3900092.config.oslist`
/// for a title that does). common.oslist is the one field confirmed
/// present in both shapes.
///
/// Scoped to before the first `"config"` key (common.oslist always comes
/// before config/depots in Valve's own emission order) so a differently-
/// scoped oslist mention further down the file - a launch entry's or a
/// depot's own - can't be mistaken for common's.
fn platform_override(text: &str) -> Option<&'static str> {
    let common_section = text.split_once("\"config\"").map_or(text, |(before, _)| before);
    let after_key = common_section.split_once("\"oslist\"")?.1;
    let oslist = quoted_token_after(after_key)?;
    if oslist.split(',').any(|platform| platform == "linux") {
        None
    } else {
        Some("windows")
    }
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

/// Finds the executable path for whichever `config.launch` entry's `oslist`
/// mentions `platform`, e.g. `"1" { "executable" "Foo.x86_64" "config" {
/// "oslist" "linux" } }` — one entry per platform, per Valve's schema.
///
/// Same lenient-scan approach as find_asset_hash: walks `"executable"`
/// occurrences in order; for each, the text up to the *next* `"executable"`
/// occurrence is that same launch entry's own scope (Valve always emits an
/// entry's oslist before the next entry's executable), so the first
/// `"oslist"` found within that scope belongs to it. Returns the first
/// entry whose oslist mentions `platform`.
fn find_executable_for_platform(text: &str, platform: &str) -> Option<String> {
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
                if oslist.contains(platform) {
                    return quoted_token_after(after_exec).map(str::to_string);
                }
            }
        }

        rest = after_exec;
    }
}

fn find_linux_executable(text: &str) -> Option<String> {
    find_executable_for_platform(text, "linux")
}

/// The Windows launch entry — a game with one of these but no Linux entry
/// needs Proton (see `fetch_metadata`, which sets `GameMetadata.proton`
/// from exactly that).
fn find_windows_executable(text: &str) -> Option<String> {
    find_executable_for_platform(text, "windows")
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
    fn find_windows_executable_picks_the_windows_entry() {
        assert_eq!(
            find_windows_executable(LAUNCH_FIXTURE),
            Some("WummsenVillage.exe".to_string())
        );
    }

    #[test]
    fn find_windows_executable_returns_none_without_a_windows_entry() {
        let linux_only = r#"
            "launch"
            {
                "0"
                {
                    "executable"		"Foo.x86_64"
                    "config"
                    {
                        "oslist"		"linux"
                    }
                }
            }
        "#;
        assert_eq!(find_windows_executable(linux_only), None);
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

    // Condensed from real `steamcmd +app_info_print 4191940` output
    // (2026-08-27) - a single-depot, Windows-only demo with *no*
    // per-launch-entry or per-depot oslist tag anywhere, only
    // common.oslist. This is the exact shape that broke the original,
    // launch-entry-scanning version of platform_override: it found no
    // oslist at all and skipped the override, so steamcmd fell through to
    // auto-detecting the host (Linux) platform and failed with "ERROR!
    // Failed to install app '4191940' (Invalid platform)".
    const WINDOWS_ONLY_APPINFO_FIXTURE: &str = r#"
        "4191940"
        {
            "common"
            {
                "name"		"Spooky Bodies Demo"
                "type"		"Demo"
                "oslist"		"windows"
                "osarch"		"64"
            }
            "config"
            {
                "installdir"		"Spooky Bodies Demo"
                "launch"
                {
                    "0"
                    {
                        "executable"		"spooky-bodies.exe"
                        "type"		"default"
                    }
                }
            }
            "depots"
            {
                "4191941"
                {
                    "manifests" { }
                }
            }
        }
    "#;

    // Condensed from real `steamcmd +app_info_print 3900090` output
    // (2026-08-27) - a 3-depot title with a per-depot oslist *and* a
    // per-launch-entry oslist for each platform, same title LAUNCH_FIXTURE
    // above is condensed from.
    const MULTI_PLATFORM_APPINFO_FIXTURE: &str = r#"
        "3900090"
        {
            "common"
            {
                "name"		"Wummsen Village Development Demo"
                "type"		"Demo"
                "oslist"		"windows,macos,linux"
            }
            "config"
            {
                "launch"
                {
                    "0" { "executable" "WummsenVillage.exe" "config" { "oslist" "windows" } }
                    "1" { "executable" "WummsenVillage.x86_64" "config" { "oslist" "linux" } }
                }
            }
            "depots"
            {
                "3900091" { "config" { "oslist" "windows" } }
                "3900092" { "config" { "oslist" "linux" } }
            }
        }
    "#;

    #[test]
    fn platform_override_is_none_when_common_oslist_includes_linux() {
        // steamcmd's own auto-detection already picks the linux depot
        // correctly for a multi-platform title, so forcing windows here
        // would wrongly skip it.
        assert_eq!(platform_override(MULTI_PLATFORM_APPINFO_FIXTURE), None);
    }

    #[test]
    fn platform_override_forces_windows_for_a_windows_only_game() {
        assert_eq!(platform_override(WINDOWS_ONLY_APPINFO_FIXTURE), Some("windows"));
    }

    #[test]
    fn platform_override_is_none_without_a_common_oslist_field() {
        assert_eq!(platform_override(""), None);
    }
}
