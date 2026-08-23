//! Downloads/caches Valve's own Proton builds via `steamcmd` — each Proton
//! version is itself a Steam "tool" app with its own AppID (the same one
//! the Steam client installs when you tick a compatibility tool in its
//! settings), so this reuses the exact same `SteamCmdSession::run`
//! `app_update` pipeline `SteamSource::provision` uses for games, just
//! pointed at a shared, version-keyed cache instead of a game's own folder.
//!
//! Deliberately Valve's official Proton only, not GE-Proton (GloriousEggroll's
//! fork — real compat patches, but GitHub-only, not a Steam app, so it can't
//! be fetched this way). Given the choice, reusing 100% of the already-working
//! steamcmd/login/session machinery here beat adding a second, unrelated
//! download pipeline just for Proton.
use std::io;
use std::path::{Path, PathBuf};

use super::steamcmd_session::SteamCmdSession;

/// Used when a game's `proton_version` isn't set to something more specific
/// (see `GameConfig.proton_version`'s doc comment on why that's resolved
/// once at install/update time and stored, not re-resolved at launch).
/// Experimental rather than a numbered release because it tracks Valve's
/// latest compat fixes — the version itself only moves when an admin
/// explicitly runs `arcade install`/`update`, never on its own.
pub const DEFAULT_VERSION: &str = "Proton Experimental";

/// Every Proton version `arcade` knows how to fetch. Extend this list as
/// new Proton releases matter — Valve doesn't expose a "list all Proton
/// AppIDs" API, so there's no way to resolve this dynamically.
const KNOWN_VERSIONS: &[(&str, &str)] = &[
    ("Proton Experimental", "1493710"),
    ("Proton 9.0", "2805730"),
    ("Proton 8.0", "2348590"),
    ("Proton 7.0", "1887720"),
];

fn appid_for(version: &str) -> Option<&'static str> {
    KNOWN_VERSIONS.iter().find(|(name, _)| *name == version).map(|(_, id)| *id)
}

/// Where `version`'s build lives under `root` (pass `arcade_core::proton_dir()`
/// in real use, a tempdir in tests) — `None` if `version` isn't a known
/// Proton version at all, distinct from "known but not downloaded yet"
/// (`is_provisioned` returning false), since the two need different error
/// messages at launch time.
pub fn dir(root: &Path, version: &str) -> Option<PathBuf> {
    appid_for(version).map(|appid| root.join(appid))
}

/// Whether `version`'s build has already been fetched into `root`.
pub fn is_provisioned(root: &Path, version: &str) -> bool {
    dir(root, version).is_some_and(|d| d.exists())
}

/// Downloads (or, run again later, updates) `version`'s build into `root` —
/// same shape as `SteamSource::provision`, just an anonymous login (Proton
/// tools are free/auto-owned by every account, same as the anonymous
/// `+app_info_print` metadata fetch) and a fixed AppID instead of an
/// admin-chosen one. Called from `install`/`update` only — never at launch,
/// so a cabinet sitting idle between admin visits never talks to Steam.
pub fn provision(root: &Path, version: &str) -> io::Result<()> {
    let appid = appid_for(version)
        .ok_or_else(|| io::Error::other(format!("unknown Proton version '{version}'")))?;
    let dest = root.join(appid);

    let args = [
        "+force_install_dir".to_string(),
        dest.to_string_lossy().into_owned(),
        "+login".to_string(),
        "anonymous".to_string(),
        "+app_update".to_string(),
        appid.to_string(),
        "validate".to_string(),
        "+quit".to_string(),
    ];
    let session = SteamCmdSession::open(crate::steamcmd_session_dir());
    let arg_refs: Vec<&str> = args.iter().map(String::as_str).collect();
    let status = session.run(&arg_refs)?;
    if !status.success() {
        return Err(io::Error::other(format!(
            "steamcmd exited with status {status} while fetching Proton '{version}'"
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dir_is_keyed_by_appid_under_root() {
        let root = Path::new("/tmp/proton-root");
        assert_eq!(dir(root, "Proton Experimental"), Some(root.join("1493710")));
    }

    #[test]
    fn dir_is_none_for_an_unknown_version() {
        let root = Path::new("/tmp/proton-root");
        assert_eq!(dir(root, "Some Made Up Version"), None);
    }

    #[test]
    fn is_provisioned_is_false_until_the_folder_exists() {
        let root = tempfile::tempdir().unwrap();
        assert!(!is_provisioned(root.path(), "Proton Experimental"));
        std::fs::create_dir_all(root.path().join("1493710")).unwrap();
        assert!(is_provisioned(root.path(), "Proton Experimental"));
    }

    #[test]
    fn is_provisioned_is_false_for_an_unknown_version() {
        let root = tempfile::tempdir().unwrap();
        assert!(!is_provisioned(root.path(), "Some Made Up Version"));
    }

    #[test]
    fn provision_errors_on_an_unknown_version_without_touching_steamcmd() {
        let root = tempfile::tempdir().unwrap();
        let err = provision(root.path(), "Some Made Up Version").unwrap_err();
        assert!(err.to_string().contains("unknown Proton version"));
    }
}
