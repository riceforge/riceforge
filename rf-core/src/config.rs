use crate::error::Result;
use std::path::PathBuf;

pub struct Paths;

impl Paths {
    pub fn cache_dir() -> PathBuf {
        dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join("riceforge")
    }

    pub fn data_dir() -> PathBuf {
        dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join("riceforge")
    }

    pub fn rices_dir() -> PathBuf {
        Self::data_dir().join("rices")
    }

    pub fn backups_dir() -> PathBuf {
        Self::data_dir().join("backups")
    }

    pub fn index_cache() -> PathBuf {
        Self::cache_dir().join("index.json")
    }

    pub fn installed_db() -> PathBuf {
        Self::data_dir().join("installed.json")
    }

    pub fn ensure_dirs() -> Result<()> {
        for dir in [
            Self::cache_dir(),
            Self::data_dir(),
            Self::rices_dir(),
            Self::backups_dir(),
        ] {
            std::fs::create_dir_all(dir)?;
        }
        Ok(())
    }
}
