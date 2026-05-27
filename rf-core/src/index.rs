use crate::{
    config::Paths,
    error::{Result, RiceForgeError},
    models::{Index, Rice, WindowManager},
};
use std::{fs, process::Command};

pub const INDEX_URL: &str =
    "https://raw.githubusercontent.com/riceforge/riceforge-index/main/index.json";

#[cfg(test)]
fn fixture_index() -> Index {
    let json = include_str!("../../fixtures/test-index.json");
    serde_json::from_str(json).expect("fixture JSON must be valid")
}

pub struct IndexManager;

impl IndexManager {
    pub fn update() -> Result<Index> {
        let output = Command::new("curl")
            .args(["-sf", "--connect-timeout", "15", INDEX_URL])
            .output()
            .map_err(|e| RiceForgeError::Http(format!("curl not found: {e}")))?;

        if !output.status.success() {
            return Err(RiceForgeError::Http(format!(
                "curl failed ({}). Check your internet connection.",
                output.status
            )));
        }

        let index: Index = serde_json::from_slice(&output.stdout)?;
        Paths::ensure_dirs()?;
        fs::write(Paths::index_cache(), serde_json::to_string_pretty(&index)?)?;
        Ok(index)
    }

    pub fn load_cached() -> Result<Index> {
        let path = Paths::index_cache();
        if !path.exists() {
            return Err(RiceForgeError::NotFound(
                "no cached index — run 'riceforge update' first".into(),
            ));
        }
        let data = fs::read_to_string(path)?;
        Ok(serde_json::from_str(&data)?)
    }

    pub fn find(index: &Index, id: &str) -> Option<Rice> {
        index.rices.iter().find(|r| r.id == id).cloned()
    }

    pub fn search<'a>(
        index: &'a Index,
        query: &str,
        wm: Option<&WindowManager>,
        theme: Option<&str>,
    ) -> Vec<&'a Rice> {
        let q = query.to_lowercase();
        index
            .rices
            .iter()
            .filter(|r| {
                let matches_query = q.is_empty()
                    || r.id.to_lowercase().contains(&q)
                    || r.name.to_lowercase().contains(&q)
                    || r.author.to_lowercase().contains(&q)
                    || r.description.to_lowercase().contains(&q)
                    || r.theme.to_lowercase().contains(&q);

                let matches_wm = wm.is_none_or(|w| &r.wm == w);
                let matches_theme =
                    theme.is_none_or(|t| r.theme.to_lowercase().contains(&t.to_lowercase()));

                matches_query && matches_wm && matches_theme
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_returns_correct_rice() {
        let index = fixture_index();
        let rice = IndexManager::find(&index, "test-rice-minimal").unwrap();
        assert_eq!(rice.id, "test-rice-minimal");
        assert_eq!(rice.author, "testuser");
    }

    #[test]
    fn find_returns_none_for_unknown_id() {
        let index = fixture_index();
        assert!(IndexManager::find(&index, "does-not-exist").is_none());
    }

    #[test]
    fn search_empty_query_returns_all() {
        let index = fixture_index();
        let results = IndexManager::search(&index, "", None, None);
        assert_eq!(results.len(), index.rices.len());
    }

    #[test]
    fn search_by_id_prefix() {
        let index = fixture_index();
        let results = IndexManager::search(&index, "test-rice-full", None, None);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "test-rice-full");
    }

    #[test]
    fn search_by_author() {
        let index = fixture_index();
        let results = IndexManager::search(&index, "testuser", None, None);
        assert_eq!(results.len(), index.rices.len());
    }

    #[test]
    fn search_by_wm_filter() {
        let index = fixture_index();
        let wm = WindowManager::Sway;
        let results = IndexManager::search(&index, "", Some(&wm), None);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "test-rice-full");
    }

    #[test]
    fn search_by_theme_filter() {
        let index = fixture_index();
        let results = IndexManager::search(&index, "", None, Some("nord"));
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].theme, "nord");
    }

    #[test]
    fn search_no_match_returns_empty() {
        let index = fixture_index();
        let results = IndexManager::search(&index, "zzznomatch", None, None);
        assert!(results.is_empty());
    }

    #[test]
    fn search_combined_wm_and_query() {
        let index = fixture_index();
        let wm = WindowManager::Hyprland;
        let results = IndexManager::search(&index, "minimal", Some(&wm), None);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "test-rice-minimal");
    }

    #[test]
    fn index_serde_roundtrip() {
        let index = fixture_index();
        let json = serde_json::to_string(&index).unwrap();
        let back: Index = serde_json::from_str(&json).unwrap();
        assert_eq!(back.rices.len(), index.rices.len());
        assert_eq!(back.version, index.version);
    }
}
