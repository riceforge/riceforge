use crate::{
    config::Paths,
    error::{Result, RiceForgeError},
    models::BackupEntry,
};
use chrono::Utc;
use std::{fs, path::PathBuf};
use walkdir::WalkDir;

pub struct BackupManager;

impl BackupManager {
    pub fn create(rice_id: Option<&str>, files: &[PathBuf]) -> Result<BackupEntry> {
        let id = Utc::now().format("%Y%m%d_%H%M%S").to_string();
        let backup_dir = Paths::backups_dir().join(&id);
        fs::create_dir_all(&backup_dir)?;

        let home = dirs::home_dir()
            .ok_or_else(|| RiceForgeError::Backup("home directory not found".into()))?;
        let config_base = home.join(".config");

        let mut backed_up = Vec::new();
        for file in files {
            if file.is_symlink() || !file.exists() {
                continue;
            }
            let relative = file.strip_prefix(&config_base).map_err(|_| {
                RiceForgeError::Backup(format!("file is outside ~/.config: {}", file.display()))
            })?;
            let dest = backup_dir.join(relative);
            if let Some(parent) = dest.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(file, &dest)?;
            backed_up.push(relative.display().to_string());
        }

        let entry = BackupEntry {
            id: id.clone(),
            rice_id: rice_id.map(str::to_string),
            created_at: Utc::now(),
            files: backed_up,
        };
        fs::write(
            backup_dir.join("meta.json"),
            serde_json::to_string_pretty(&entry)?,
        )?;

        Ok(entry)
    }

    pub fn list() -> Result<Vec<BackupEntry>> {
        let dir = Paths::backups_dir();
        if !dir.exists() {
            return Ok(vec![]);
        }

        let mut entries = Vec::new();
        for entry in fs::read_dir(&dir)? {
            let path = entry?.path();
            if !path.is_dir() {
                continue;
            }
            let meta = path.join("meta.json");
            if meta.exists() {
                let data = fs::read_to_string(&meta)?;
                entries.push(serde_json::from_str::<BackupEntry>(&data)?);
            }
        }
        entries.sort_by_key(|b| std::cmp::Reverse(b.created_at));
        Ok(entries)
    }

    pub fn restore(id: &str) -> Result<()> {
        let backup_dir = Paths::backups_dir().join(id);
        if !backup_dir.exists() {
            return Err(RiceForgeError::NoBackup(id.to_string()));
        }

        let home = dirs::home_dir()
            .ok_or_else(|| RiceForgeError::Backup("home directory not found".into()))?;
        let config_base = home.join(".config");

        for entry in WalkDir::new(&backup_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            let path = entry.path();
            if path.file_name().is_some_and(|n| n == "meta.json") {
                continue;
            }
            let relative = path
                .strip_prefix(&backup_dir)
                .map_err(|_| RiceForgeError::Backup("path strip failed".into()))?;
            let dest = config_base.join(relative);
            if let Some(parent) = dest.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(path, &dest)?;
        }

        Ok(())
    }

    pub fn clean(keep: usize) -> Result<Vec<String>> {
        let mut entries = Self::list()?;
        let mut removed = Vec::new();

        for entry in entries.drain(keep..) {
            let path = Paths::backups_dir().join(&entry.id);
            fs::remove_dir_all(&path)?;
            removed.push(entry.id);
        }
        Ok(removed)
    }
}
