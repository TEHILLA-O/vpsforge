use thiserror::Error;

pub type ForgeResult<T> = Result<T, ForgeError>;

#[derive(Debug, Error)]
pub enum ForgeError {
    #[error("{0}")]
    Message(String),

    #[error("unsupported platform: {0}")]
    UnsupportedPlatform(String),

    #[error("unsupported distribution family: {0}")]
    UnsupportedDistro(String),

    #[error("package manager error: {0}")]
    PackageManager(String),

    #[error("profile '{0}' not found")]
    ProfileNotFound(String),

    #[error("blueprint error: {0}")]
    Blueprint(String),

    #[error("checkpoint error: {0}")]
    Checkpoint(String),

    #[error("security: {0}")]
    Security(String),

    #[error("trust verification failed: {0}")]
    Trust(String),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("{0}")]
    Other(#[from] anyhow::Error),
}

impl ForgeError {
    pub fn msg(msg: impl Into<String>) -> Self {
        Self::Message(msg.into())
    }
}
