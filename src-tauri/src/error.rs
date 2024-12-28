use serde::{Serialize, Serializer};

pub type FluxyResult<T> = std::result::Result<T, FluxyError>;

#[derive(Debug, thiserror::Error)]
pub enum FluxyError {
    #[error(transparent)]
    Tauri(#[from] tauri::Error),
    #[cfg(desktop)]
    #[error(transparent)]
    Update(#[from] tauri_plugin_updater::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    AddrParse(#[from] std::net::AddrParseError),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    SystemTime(#[from] std::time::SystemTimeError),
}

impl Serialize for FluxyError {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}
