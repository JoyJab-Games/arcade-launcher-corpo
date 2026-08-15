use std::error::Error;

pub fn shutdown() -> Result<(), Box<dyn Error>> {
    arcade_core::power::shutdown()?;
    Ok(())
}

pub fn reboot() -> Result<(), Box<dyn Error>> {
    arcade_core::power::reboot()?;
    Ok(())
}
