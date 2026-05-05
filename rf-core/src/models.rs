use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum WindowManager {
    Hyprland,
    Sway,
    I3,
    Openbox,
    Bspwm,
    Qtile,
    Xmonad,
    #[serde(other)]
    Unknown,
}

impl std::fmt::Display for WindowManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Hyprland => "Hyprland",
            Self::Sway => "Sway",
            Self::I3 => "i3",
            Self::Openbox => "Openbox",
            Self::Bspwm => "bspwm",
            Self::Qtile => "Qtile",
            Self::Xmonad => "XMonad",
            Self::Unknown => "Unknown",
        })
    }
}

impl std::str::FromStr for WindowManager {
    type Err = ();
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Ok(match s.to_lowercase().as_str() {
            "hyprland" => Self::Hyprland,
            "sway" => Self::Sway,
            "i3" => Self::I3,
            "openbox" => Self::Openbox,
            "bspwm" => Self::Bspwm,
            "qtile" => Self::Qtile,
            "xmonad" => Self::Xmonad,
            _ => Self::Unknown,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Rice {
    pub id: String,
    pub name: String,
    pub author: String,
    pub description: String,
    pub wm: WindowManager,
    pub theme: String,
    pub fonts: Vec<String>,
    pub dependencies: Vec<String>,
    pub repo_url: String,
    pub screenshots: Vec<String>,
    pub stars: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commit_hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Index {
    pub version: String,
    pub updated_at: DateTime<Utc>,
    pub rices: Vec<Rice>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InstalledRice {
    pub rice_id: String,
    pub commit_hash: String,
    pub installed_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backup_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupEntry {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rice_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub files: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct DeployPlan {
    pub links: Vec<(std::path::PathBuf, std::path::PathBuf)>,
    pub to_backup: Vec<std::path::PathBuf>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wm_display_all_variants() {
        assert_eq!(WindowManager::Hyprland.to_string(), "Hyprland");
        assert_eq!(WindowManager::Sway.to_string(), "Sway");
        assert_eq!(WindowManager::I3.to_string(), "i3");
        assert_eq!(WindowManager::Openbox.to_string(), "Openbox");
        assert_eq!(WindowManager::Bspwm.to_string(), "bspwm");
        assert_eq!(WindowManager::Qtile.to_string(), "Qtile");
        assert_eq!(WindowManager::Xmonad.to_string(), "XMonad");
        assert_eq!(WindowManager::Unknown.to_string(), "Unknown");
    }

    #[test]
    fn wm_from_str_case_insensitive() {
        use std::str::FromStr;
        assert_eq!(WindowManager::from_str("Hyprland").unwrap(), WindowManager::Hyprland);
        assert_eq!(WindowManager::from_str("SWAY").unwrap(), WindowManager::Sway);
        assert_eq!(WindowManager::from_str("i3").unwrap(), WindowManager::I3);
        assert_eq!(WindowManager::from_str("openbox").unwrap(), WindowManager::Openbox);
        assert_eq!(WindowManager::from_str("bspwm").unwrap(), WindowManager::Bspwm);
        assert_eq!(WindowManager::from_str("qtile").unwrap(), WindowManager::Qtile);
        assert_eq!(WindowManager::from_str("xmonad").unwrap(), WindowManager::Xmonad);
        assert_eq!(WindowManager::from_str("nope").unwrap(), WindowManager::Unknown);
    }

    #[test]
    fn wm_serde_roundtrip() {
        let cases = [
            (WindowManager::Hyprland, r#""hyprland""#),
            (WindowManager::Sway, r#""sway""#),
            (WindowManager::I3, r#""i3""#),
        ];
        for (wm, json) in &cases {
            assert_eq!(&serde_json::to_string(wm).unwrap(), json);
            let parsed: WindowManager = serde_json::from_str(json).unwrap();
            assert_eq!(&parsed, wm);
        }
    }

    #[test]
    fn rice_serde_roundtrip() {
        let json = r#"{
            "id": "test-rice",
            "name": "Test Rice",
            "author": "user",
            "description": "desc",
            "wm": "hyprland",
            "theme": "dark",
            "fonts": ["Nerd Font"],
            "dependencies": ["waybar"],
            "repo_url": "https://github.com/user/test-rice",
            "screenshots": [],
            "stars": 7
        }"#;
        let rice: Rice = serde_json::from_str(json).unwrap();
        assert_eq!(rice.id, "test-rice");
        assert_eq!(rice.wm, WindowManager::Hyprland);
        assert_eq!(rice.stars, 7);
        assert!(rice.commit_hash.is_none());
        assert!(rice.updated_at.is_none());

        let back = serde_json::to_string(&rice).unwrap();
        let rice2: Rice = serde_json::from_str(&back).unwrap();
        assert_eq!(rice, rice2);
    }

    #[test]
    fn rice_with_optional_fields() {
        let json = r#"{
            "id": "r",
            "name": "R",
            "author": "a",
            "description": "d",
            "wm": "sway",
            "theme": "nord",
            "fonts": [],
            "dependencies": [],
            "repo_url": "https://example.com",
            "screenshots": [],
            "stars": 0,
            "commit_hash": "abc123",
            "updated_at": "2026-01-01T00:00:00Z"
        }"#;
        let rice: Rice = serde_json::from_str(json).unwrap();
        assert_eq!(rice.commit_hash.as_deref(), Some("abc123"));
        assert!(rice.updated_at.is_some());
    }
}
