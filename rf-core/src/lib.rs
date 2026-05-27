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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Index {
    pub version: String,
    pub updated_at: String,
    pub rices: Vec<Rice>,
}

#[derive(Debug, thiserror::Error)]
pub enum RiceForgeError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
    #[error("toml: {0}")]
    Toml(#[from] toml::de::Error),
    #[error("rice not found: {id}")]
    NotFound { id: String },
}

pub type Result<T> = std::result::Result<T, RiceForgeError>;
