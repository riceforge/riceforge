use crate::{
    config::Paths,
    error::{Result, RiceForgeError},
    models::Rice,
};
use std::{fs, io::Read, path::Path, process::Command, time::{Duration, Instant}};

const CLONE_TIMEOUT_SECS: u64 = 300;
const PULL_TIMEOUT_SECS:  u64 = 120;

pub struct GitManager;

impl GitManager {

    pub fn clone_or_pull(rice: &Rice) -> Result<String> {
        Self::clone_or_pull_with_progress(rice, |_| {})
    }

    pub fn clone_or_pull_with_progress<F>(rice: &Rice, on_line: F) -> Result<String>
    where
        F: Fn(String) + Send + 'static,
    {
        let dest = Paths::rices_dir().join(&rice.id);
        if dest.exists() {
            Self::pull_with_progress(&dest, on_line)
        } else {
            Self::clone_with_progress(&rice.repo_url, &dest, &rice.id, on_line)
        }
    }

    fn clone_with_progress<F>(url: &str, dest: &Path, id: &str, on_line: F) -> Result<String>
    where
        F: Fn(String) + Send + 'static,
    {
        let mut child = Command::new("git")
            .args(["clone", "--depth=1", "--progress", url])
            .arg(dest)
            .env("GIT_TERMINAL_PROMPT", "0")
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| RiceForgeError::Git(format!("git not found: {e}")))?;

        if let Some(stderr) = child.stderr.take() {
            std::thread::spawn(move || {
                stream_git_output(stderr, on_line);
            });
        }

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

    fn pull_with_progress<F>(dir: &Path, on_line: F) -> Result<String>
    where
        F: Fn(String) + Send + 'static,
    {
        let mut child = Command::new("git")
            .current_dir(dir)
            .args(["pull", "--ff-only", "--progress"])
            .env("GIT_TERMINAL_PROMPT", "0")
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| RiceForgeError::Git(format!("git not found: {e}")))?;

        if let Some(stderr) = child.stderr.take() {
            std::thread::spawn(move || {
                stream_git_output(stderr, on_line);
            });
        }

        Self::wait_with_timeout(&mut child, PULL_TIMEOUT_SECS)
            .and_then(|ok| {
                if ok {
                    Ok(())
                } else {
                    Err(RiceForgeError::Git(
                        "git pull failed — manual resolution needed".into(),
                    ))
                }
            })?;

        Self::head_hash(dir)
    }


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

fn stream_git_output<R, F>(mut reader: R, on_line: F)
where
    R: Read,
    F: Fn(String),
{
    let mut buf = Vec::<u8>::with_capacity(256);
    let mut byte = [0u8; 1];

    while reader.read(&mut byte).ok() == Some(1) {
        match byte[0] {
            b'\n' | b'\r' => {
                if !buf.is_empty() {
                    let line = String::from_utf8_lossy(&buf).trim().to_string();
                    if !line.is_empty() {
                        on_line(line);
                    }
                    buf.clear();
                }
            }
            b => buf.push(b),
        }
    }

    if !buf.is_empty() {
        let line = String::from_utf8_lossy(&buf).trim().to_string();
        if !line.is_empty() {
            on_line(line);
        }
    }
}
