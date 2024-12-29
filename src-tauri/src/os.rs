use std::fmt::Display;

use os_info::Version;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum OsType {
    Linux,
    Windows,
    Macos,
    IOS,
    Android,
}

impl Display for OsType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Linux => write!(f, "linux"),
            Self::Windows => write!(f, "windows"),
            Self::Macos => write!(f, "macos"),
            Self::IOS => write!(f, "ios"),
            Self::Android => write!(f, "android"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OsInformation {
    platform: String,
    os_type: OsType,
    version: String,
}

impl OsInformation {
    pub fn new() -> Self {
        Self {
            platform: platform().to_string(),
            os_type: os_type(),
            version: version().to_string(),
        }
    }
}

fn platform() -> &'static str {
    std::env::consts::OS
}

pub fn version() -> Version {
    os_info::get().version().clone()
}

pub fn os_type() -> OsType {
    #[cfg(any(
        target_os = "linux",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd"
    ))]
    return OsType::Linux;
    #[cfg(target_os = "windows")]
    return OsType::Windows;
    #[cfg(target_os = "macos")]
    return OsType::Macos;
    #[cfg(target_os = "ios")]
    return OsType::IOS;
    #[cfg(target_os = "android")]
    return OsType::Android;
}
