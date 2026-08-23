use std::fs;
use std::io;
use std::path::Path;

/// Fetches `url` and writes its response body to `dest`, creating parent
/// directories as needed. A 404 returns `Ok(false)` (nothing written)
/// rather than an error — used for "does this asset even exist" checks
/// (e.g. not every Steam AppID has capsule art), where a missing file is
/// an expected outcome the caller falls back on, not a failure to report.
pub fn download_to_file(url: &str, dest: &Path) -> io::Result<bool> {
    let response = match ureq::get(url).call() {
        Ok(response) => response,
        Err(ureq::Error::Status(404, _)) => return Ok(false),
        Err(e) => return Err(io::Error::other(e)),
    };

    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut file = fs::File::create(dest)?;
    io::copy(&mut response.into_reader(), &mut file)?;
    Ok(true)
}

/// Fetches `url` and returns its response body as a string — used for the
/// small JSON metadata calls (e.g. Steam's `appdetails` endpoint).
pub fn get_string(url: &str) -> io::Result<String> {
    ureq::get(url)
        .call()
        .map_err(io::Error::other)?
        .into_string()
        .map_err(io::Error::other)
}
