use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// A simple file-based lock guarding a single concurrent operation (e.g.
/// "don't let two `arcade install` calls race on the same store"). Not a
/// job/status system — for long-running downloads you want to check on
/// from another session, run the command inside tmux/screen instead.
pub struct ProcessLock {
    path: PathBuf,
}

impl ProcessLock {
    /// Acquires the lock at `path`, or returns an error naming the PID
    /// that already holds it. A lock left behind by a process that's no
    /// longer running is treated as stale and reclaimed automatically.
    pub fn acquire(path: impl Into<PathBuf>) -> io::Result<Self> {
        let path = path.into();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        if let Some(holder_pid) = Self::read_holder(&path) {
            if Self::is_running(holder_pid) {
                return Err(io::Error::other(format!(
                    "another arcade command is already running (pid {holder_pid})"
                )));
            }
            // Stale lock from a process that's gone - reclaim it.
            let _ = fs::remove_file(&path);
        }

        fs::write(&path, std::process::id().to_string())?;
        Ok(Self { path })
    }

    fn read_holder(path: &Path) -> Option<u32> {
        fs::read_to_string(path).ok()?.trim().parse().ok()
    }

    fn is_running(pid: u32) -> bool {
        Path::new(&format!("/proc/{pid}")).exists()
    }
}

impl Drop for ProcessLock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn second_acquire_fails_while_first_is_held() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("arcade.lock");
        let _first = ProcessLock::acquire(&path).unwrap();
        assert!(ProcessLock::acquire(&path).is_err());
    }

    #[test]
    fn lock_is_released_on_drop() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("arcade.lock");
        {
            let _first = ProcessLock::acquire(&path).unwrap();
        }
        assert!(ProcessLock::acquire(&path).is_ok());
    }

    #[test]
    fn stale_lock_from_a_dead_pid_is_reclaimed() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("arcade.lock");
        // PID 1 is init and (almost certainly) alive on any real system,
        // so use an implausibly high PID to simulate a dead process.
        fs::write(&path, "999999999").unwrap();
        assert!(ProcessLock::acquire(&path).is_ok());
    }
}
