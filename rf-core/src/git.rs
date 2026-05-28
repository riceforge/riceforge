use crate::{
    config::Paths,
    error::{Result, RiceForgeError},
    models::Rice,
};
use std::{fs, path::Path, process::Command, time::{Duration, Instant}};

const CLONE_TIMEOUT_SECS: u64 = 300; // 5 min for large repos (HyDE ~500MB)
const PULL_TIMEOUT_SECS:  u64 = 120;

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
        let mut child = Command::new("git")
            .args(["clone", "--depth=1", "--progress", url])
            .arg(dest)
            .env("GIT_TERMINAL_PROMPT", "0")
            .spawn()
            .map_err(|e| RiceForgeError::Git(format!("git not found: {e}")))?;

        Self::wait_with_timeout(&mut child, CLONE_TIMEOUT_SECS)
            .and_then(|ok| {
                if ok {
                    Ok(())
                } else {
                    Err(RiceForgeError::Git(format!("git clone failed for '{id}'")))
                }
            })?;

        Self::head_hash(dest)
    }

    fn pull(dir: &Path) -> Result<String> {
        let mut child = Command::new("git")
            .current_dir(dir)
            .args(["pull", "--ff-only", "--quiet"])
            .env("GIT_TERMINAL_PROMPT", "0")
            .spawn()
            .map_err(|e| RiceForgeError::Git(format!("git not found: {e}")))?;

        Self::wait_with_timeout(&mut child, PULL_TIMEOUT_SECS)
            .and_then(|ok| {
                if ok {
                    Ok(())
                } else {
                    Err(RiceForgeError::Git("git pull failed — manual resolution needed".into()))
                }
            })?;

        Self::head_hash(dir)
    }

    /// Wait for a child process with a timeout. Returns `Ok(true)` on success,
    /// `Ok(false)` on non-zero exit, `Err` on timeout or I/O error.
    fn wait_with_timeout(child: &mut std::process::Child, secs: u64) -> Result<bool> {
        let deadline = Duration::from_secs(secs);
        let start = Instant::now();

        loop {
            match child.try_wait().map_err(|e| RiceForgeError::Git(e.to_string()))? {
                Some(status) => return Ok(status.success()),
                None => {
                    if start.elapsed() >= deadline {
                        let _ = child.kill();
                        return Err(RiceForgeError::Git(format!(
                            "git operation timed out after {secs}s"
                        )));
                    }
                    std::thread::sleep(Duration::from_millis(200));
                }
            }
        }
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
