use crate::error::{Result, RiceForgeError};
use std::process::Command;

pub struct PackageManager;

impl PackageManager {
    pub fn is_available() -> bool {
        Command::new("pacman")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    pub fn is_installed(package: &str) -> bool {
        Command::new("pacman")
            .args(["-Q", package])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    pub fn missing(packages: &[String]) -> Vec<&str> {
        packages
            .iter()
            .filter(|p| !Self::is_installed(p))
            .map(|p| p.as_str())
            .collect()
    }

    pub fn install(packages: &[&str]) -> Result<()> {
        if packages.is_empty() {
            return Ok(());
        }

        let status = Command::new("sudo")
            .args(["pacman", "-S", "--needed", "--noconfirm"])
            .args(packages)
            .status()
            .map_err(|e| RiceForgeError::PackageManager(format!("sudo/pacman unavailable: {e}")))?;

        if !status.success() {
            return Err(RiceForgeError::PackageManager(format!(
                "pacman failed for: {}",
                packages.join(", ")
            )));
        }

        Ok(())
    }
}
