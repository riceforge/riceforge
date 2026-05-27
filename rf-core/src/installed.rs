use std::fs;
use chrono::Utc;
use crate::{
    config::Paths,
    error::{Result, RiceForgeError},
    models::InstalledRice,
};

pub struct InstalledManager;

impl InstalledManager {
    fn load() -> Result<Vec<InstalledRice>> {
        let path = Paths::installed_db();
        if !path.exists() {
            return Ok(vec![]);
        }
        let data = fs::read_to_string(path)?;
        Ok(serde_json::from_str(&data)?)
    }

    fn save(entries: &[InstalledRice]) -> Result<()> {
        Paths::ensure_dirs()?;
        fs::write(Paths::installed_db(), serde_json::to_string_pretty(entries)?)?;
        Ok(())
    }

    pub fn list() -> Result<Vec<InstalledRice>> {
        Self::load()
    }

    pub fn is_installed(rice_id: &str) -> Result<bool> {
        Ok(Self::load()?.iter().any(|e| e.rice_id == rice_id))
    }

    pub fn get(rice_id: &str) -> Result<InstalledRice> {
        Self::load()?
            .into_iter()
            .find(|e| e.rice_id == rice_id)
            .ok_or_else(|| RiceForgeError::NotInstalled(rice_id.to_string()))
    }

    pub fn add(rice_id: &str, commit_hash: &str, backup_id: Option<String>) -> Result<()> {
        let mut entries = Self::load()?;
        entries.retain(|e| e.rice_id != rice_id);
        entries.push(InstalledRice {
            rice_id: rice_id.to_string(),
            commit_hash: commit_hash.to_string(),
            installed_at: Utc::now(),
            backup_id,
        });
        Self::save(&entries)
    }

    pub fn remove(rice_id: &str) -> Result<Option<InstalledRice>> {
        let mut entries = Self::load()?;
        let pos = entries.iter().position(|e| e.rice_id == rice_id);
        if let Some(i) = pos {
            let entry = entries.remove(i);
            Self::save(&entries)?;
            Ok(Some(entry))
        } else {
            Ok(None)
        }
    }
}
