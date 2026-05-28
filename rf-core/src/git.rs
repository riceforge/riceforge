use crate::{
    config::Paths,
    error::{Result, RiceForgeError},
    models::Rice,
};
use std::{fs, path::Path, process::Command};

pub struct GitManager;

impl GitManager {
    pub fn clone_or_pull(rice: &Rice) -> Result<String> {
        let dest = Paths::rices_dir().join(&rice.id);
        if dest.exists() {
            Self::pull(&dest)
        } else {
            Self::clone(&rice.repo_url, &dest, &rice.id)
        }
    }

    fn clone(url: &str, dest: &Path, id: &str) -> Result<String> {
        let status = Command::new("git")
            .args(["clone", "--depth=1", "--progress", url])
            .arg(dest)
            .status()
            .map_err(|e| RiceForgeError::Git(format!("git not found: {e}")))?;

        if !status.success() {
            return Err(RiceForgeError::Git(format!("git clone failed for '{id}'")));
        }

        Self::head_hash(dest)
    }

    fn pull(dir: &Path) -> Result<String> {
        let status = Command::new("git")
            .current_dir(dir)
            .args(["pull", "--ff-only", "--quiet"])
            .status()
            .map_err(|e| RiceForgeError::Git(format!("git not found: {e}")))?;

        if !status.success() {
            return Err(RiceForgeError::Git(
                "git pull failed — manual resolution needed".into(),
            ));
        }

        Self::head_hash(dir)
    }

    pub fn head_hash(dir: &Path) -> Result<String> {
        let output = Command::new("git")
            .current_dir(dir)
            .args(["rev-parse", "HEAD"])
            .output()
            .map_err(|e| RiceForgeError::Git(e.to_string()))?;

        if !output.status.success() {
            return Err(RiceForgeError::Git("could not read HEAD hash".into()));
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    pub fn remove(rice_id: &str) -> Result<()> {
        let dest = Paths::rices_dir().join(rice_id);
        if dest.exists() {
            fs::remove_dir_all(dest)?;
        }
        Ok(())
    }
}
