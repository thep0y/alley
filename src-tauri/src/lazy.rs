use std::{path::PathBuf, sync::LazyLock};

pub static FLUXY_CONFIG_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    let config_dir = dirs::config_dir().unwrap();

    let app_config_dir = config_dir.join("dwall");

    if !app_config_dir.exists() {
        if let Err(e) = std::fs::create_dir(&app_config_dir) {
            error!(error = ?e, "Failed to create config directory");
            panic!("Failed to create config directory: {}", e);
        } else {
            info!(path = %app_config_dir.display(), "Config directory created successfully");
        }
    } else {
        debug!(path = %app_config_dir.display(), "Config directory already exists");
    }

    app_config_dir
});
