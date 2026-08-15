use std::io;
use std::process::Command;

pub fn shutdown() -> io::Result<()> {
    run_systemctl("poweroff")
}

pub fn reboot() -> io::Result<()> {
    run_systemctl("reboot")
}

fn run_systemctl(action: &str) -> io::Result<()> {
    let status = Command::new("systemctl").arg(action).status()?;
    if status.success() {
        Ok(())
    } else {
        Err(io::Error::other(format!(
            "systemctl {action} exited with {status}"
        )))
    }
}
