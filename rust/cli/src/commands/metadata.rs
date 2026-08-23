use std::io::{self, Write};
use std::path::Path;

/// Returns `fetched` if the source found any tags itself, otherwise prompts
/// the admin to type some in — the manual fallback every source (not just
/// Steam) goes through when it has nothing to offer. Never fails the
/// install over this; a blank answer just means no tags.
pub fn resolve_tags(fetched: Vec<String>) -> io::Result<Vec<String>> {
    if !fetched.is_empty() {
        return Ok(fetched);
    }
    let typed = prompt("Tags (comma-separated, blank to skip): ")?;
    Ok(typed
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect())
}

/// Downloads `fetched_url` into `dest_dir` if present and reachable;
/// otherwise (missing, 404, or any fetch error) asks the admin to paste one
/// manually. Returns the image's filename (relative to `dest_dir`) if
/// anything was saved — never fails the install over this, a preview
/// image is cosmetic.
pub fn resolve_image(fetched_url: Option<String>, dest_dir: &Path) -> io::Result<Option<String>> {
    if let Some(url) = fetched_url {
        match try_download(&url, dest_dir) {
            Ok(Some(name)) => return Ok(Some(name)),
            Ok(None) => println!("No preview image found automatically ({url} doesn't exist)."),
            Err(e) => println!("Couldn't fetch a preview image automatically ({e})."),
        }
    } else {
        println!("This source has no preview image to fetch automatically.");
    }

    let typed = prompt("Preview image URL, if you have one (optional — blank leaves it unset, you can add one later with `arcade update`): ")?;
    if typed.is_empty() {
        return Ok(None);
    }
    match try_download(&typed, dest_dir) {
        Ok(Some(name)) => Ok(Some(name)),
        Ok(None) => {
            println!("Couldn't fetch that URL — continuing without a preview image.");
            Ok(None)
        }
        Err(e) => {
            println!("Couldn't fetch that URL either ({e}) — continuing without a preview image.");
            Ok(None)
        }
    }
}

fn try_download(url: &str, dest_dir: &Path) -> io::Result<Option<String>> {
    let filename = format!("capsule.{}", extension_of(url));
    let dest = dest_dir.join(&filename);
    if arcade_core::http::download_to_file(url, &dest)? {
        Ok(Some(filename))
    } else {
        Ok(None)
    }
}

fn extension_of(url: &str) -> &str {
    url.rsplit('/')
        .next()
        .and_then(|last| last.rsplit_once('.'))
        .map(|(_, ext)| ext)
        .filter(|ext| ext.len() <= 4 && !ext.is_empty() && ext.chars().all(|c| c.is_ascii_alphanumeric()))
        .unwrap_or("jpg")
}

/// Yes/no prompt, defaulting to "no" on a blank/unrecognized answer — for
/// the "you're about to redo a slow step, are you sure" cases (e.g.
/// re-fetching an already-downloaded Proton build) where staying quiet is
/// the safer default.
pub fn confirm(label: &str) -> io::Result<bool> {
    let typed = prompt(&format!("{label} [y/N]: "))?;
    Ok(matches!(typed.to_lowercase().as_str(), "y" | "yes"))
}

fn prompt(label: &str) -> io::Result<String> {
    print!("{label}");
    io::stdout().flush()?;
    let mut line = String::new();
    io::stdin().read_line(&mut line)?;
    Ok(line.trim().to_string())
}
