use thiserror::Error;

#[derive(Debug, Error)]
pub enum RiceForgeError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
    #[error("toml: {0}")]
    Toml(#[from] toml::de::Error),
    #[error("git: {0}")]
    Git(String),
    #[error("http: {0}")]
    Http(String),
    #[error("not found: {0}")]
    NotFound(String),
    #[error("already installed: {0}")]
    AlreadyInstalled(String),
    #[error("not installed: {0}")]
    NotInstalled(String),
    #[error("no backup found: {0}")]
    NoBackup(String),
    #[error("unsupported platform — Linux required")]
    UnsupportedPlatform,
    #[error("package manager: {0}")]
    PackageManager(String),
    #[error("backup: {0}")]
    Backup(String),
    #[error("deploy: {0}")]
    Deploy(String),
    #[error("pipeline: {0}")]
    Pipeline(String),
}

pub type Result<T> = std::result::Result<T, RiceForgeError>;
