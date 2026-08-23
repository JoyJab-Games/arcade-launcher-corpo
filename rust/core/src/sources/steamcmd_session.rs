use std::fs;
use std::io;
use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;

/// Wraps steamcmd's own login/session cache so it survives repeated
/// `arcade install`/`update` calls made in quick succession, but goes stale
/// after a few minutes of inactivity and never survives a reboot.
///
/// Achieved by pointing steamcmd at a directory we own (see
/// `arcade_core::steamcmd_session_dir` — tmpfs-backed by default, so it's
/// gone at reboot with no cleanup needed from us) via `$HOME`, since
/// steamcmd resolves its own config/session cache off that. Idle expiry is
/// enforced the same way `ProcessLock` handles a stale lock: a sentinel
/// file's mtime is checked before each command, and the whole session
/// directory is wiped if it's older than IDLE_TIMEOUT — steamcmd then just
/// re-prompts for login (including any Steam Guard code) on its next
/// command, same as a first-ever login.
pub struct SteamCmdSession {
    dir: PathBuf,
}

const IDLE_TIMEOUT: Duration = Duration::from_secs(5 * 60);

impl SteamCmdSession {
    pub fn open(dir: impl Into<PathBuf>) -> Self {
        Self { dir: dir.into() }
    }

    fn sentinel(&self) -> PathBuf {
        self.dir.join(".last_used")
    }

    fn expire_if_idle(&self) -> io::Result<()> {
        let idle = match fs::metadata(self.sentinel()).and_then(|m| m.modified()) {
            Ok(modified) => modified.elapsed().unwrap_or_default() > IDLE_TIMEOUT,
            Err(_) => false, // no sentinel yet - nothing to expire
        };
        if idle {
            fs::remove_dir_all(&self.dir)?;
        }
        Ok(())
    }

    /// Runs `steamcmd` with `args`, its stdio inherited so login/Guard-code
    /// prompts go straight to the calling terminal. Touches the sentinel
    /// (resetting the idle window) only on success. For interactive
    /// commands (e.g. a real account login + download).
    pub fn run(&self, args: &[&str]) -> io::Result<std::process::ExitStatus> {
        self.expire_if_idle()?;
        fs::create_dir_all(&self.dir)?;

        let status = Command::new("steamcmd").env("HOME", &self.dir).args(args).status()?;

        if status.success() {
            fs::write(self.sentinel(), b"")?;
        }
        Ok(status)
    }

    /// Like `run`, but captures stdout/stderr instead of inheriting the
    /// terminal, for commands whose output needs parsing rather than
    /// watching live (e.g. `app_info_print`) and that don't need
    /// interactive prompts (anonymous login).
    pub fn run_capturing(&self, args: &[&str]) -> io::Result<std::process::Output> {
        self.expire_if_idle()?;
        fs::create_dir_all(&self.dir)?;

        let output = Command::new("steamcmd").env("HOME", &self.dir).args(args).output()?;

        if output.status.success() {
            fs::write(self.sentinel(), b"")?;
        }
        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::time::SystemTime;

    #[test]
    fn expire_if_idle_leaves_a_fresh_session_alone() {
        let dir = tempfile::tempdir().unwrap();
        let session_dir = dir.path().join("session");
        fs::create_dir_all(&session_dir).unwrap();
        fs::write(session_dir.join(".last_used"), b"").unwrap();
        fs::write(session_dir.join("config.vdf"), b"cached login").unwrap();

        SteamCmdSession::open(&session_dir).expire_if_idle().unwrap();

        assert!(session_dir.join("config.vdf").exists());
    }

    #[test]
    fn expire_if_idle_wipes_a_stale_session() {
        let dir = tempfile::tempdir().unwrap();
        let session_dir = dir.path().join("session");
        fs::create_dir_all(&session_dir).unwrap();
        let sentinel = File::create(session_dir.join(".last_used")).unwrap();
        sentinel
            .set_modified(SystemTime::now() - IDLE_TIMEOUT - Duration::from_secs(1))
            .unwrap();
        fs::write(session_dir.join("config.vdf"), b"cached login").unwrap();

        SteamCmdSession::open(&session_dir).expire_if_idle().unwrap();

        assert!(!session_dir.exists());
    }

    #[test]
    fn expire_if_idle_is_a_noop_with_no_prior_session() {
        let dir = tempfile::tempdir().unwrap();
        let session_dir = dir.path().join("session");
        // Directory doesn't even exist yet - first-ever run.
        SteamCmdSession::open(&session_dir).expire_if_idle().unwrap();
    }
}
