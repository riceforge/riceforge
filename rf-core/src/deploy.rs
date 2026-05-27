use crate::{
    config::Paths,
    error::{Result, RiceForgeError},
    models::{DeployPlan, Rice},
};
#[cfg(unix)]
use std::fs;
use std::path::PathBuf;
use walkdir::WalkDir;

const EXCLUDED_AT_ROOT: &[&str] = &[
    "rice.toml",
    "pipeline.toml",
    "README.md",
    "README.rst",
    "README",
    "LICENSE",
    "LICENSE.md",
    "LICENSE.txt",
    ".git",
    ".gitignore",
    ".gitmodules",
    "screenshots",
    "preview.png",
    "preview.jpg",
    "preview.webp",
    "INSTALL.md",
    "install.sh",
];

pub struct DeployManager;

impl DeployManager {
    pub fn plan(rice: &Rice) -> Result<DeployPlan> {
        let rice_dir = Paths::rices_dir().join(&rice.id);
        if !rice_dir.exists() {
            return Err(RiceForgeError::Deploy(format!(
                "rice '{}' has not been cloned",
                rice.id
            )));
        }

        let home = dirs::home_dir()
            .ok_or_else(|| RiceForgeError::Deploy("home directory not found".into()))?;

        let mut links = Vec::new();
        let mut to_backup = Vec::new();

        for entry in WalkDir::new(&rice_dir)
            .min_depth(1)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let src = entry.path().to_path_buf();
            let relative = src
                .strip_prefix(&rice_dir)
                .map_err(|_| RiceForgeError::Deploy("path strip failed".into()))?;

            let first = relative
                .components()
                .next()
                .map(|c| c.as_os_str().to_string_lossy().to_string())
                .unwrap_or_default();

            if EXCLUDED_AT_ROOT.contains(&first.as_str()) {
                continue;
            }
            if entry.file_type().is_dir() {
                continue;
            }

            let dest = home.join(relative);
            if dest.exists() && !dest.is_symlink() {
                to_backup.push(dest.clone());
            }
            links.push((src, dest));
        }

        Ok(DeployPlan { links, to_backup })
    }

    #[cfg(unix)]
    pub fn apply(plan: &DeployPlan) -> Result<()> {
        use std::os::unix::fs::symlink;

        for (src, dest) in &plan.links {
            if let Some(parent) = dest.parent() {
                fs::create_dir_all(parent)?;
            }
            if fs::symlink_metadata(dest).is_ok() {
                fs::remove_file(dest)?;
            }
            symlink(src, dest).map_err(|e| {
                RiceForgeError::Deploy(format!(
                    "symlink {} -> {}: {e}",
                    src.display(),
                    dest.display()
                ))
            })?;
        }
        Ok(())
    }

    #[cfg(not(unix))]
    pub fn apply(_plan: &DeployPlan) -> Result<()> {
        Err(RiceForgeError::UnsupportedPlatform)
    }

    #[cfg(unix)]
    pub fn remove(rice: &Rice) -> Result<Vec<PathBuf>> {
        let plan = Self::plan(rice)?;
        let mut removed = Vec::new();

        for (_, dest) in &plan.links {
            if dest.is_symlink() {
                fs::remove_file(dest)?;
                removed.push(dest.clone());
            }
        }
        Ok(removed)
    }

    #[cfg(not(unix))]
    pub fn remove(_rice: &Rice) -> Result<Vec<PathBuf>> {
        Err(RiceForgeError::UnsupportedPlatform)
    }
}
