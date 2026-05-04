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
