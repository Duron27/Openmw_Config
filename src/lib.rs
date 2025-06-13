mod config;
pub use config::OpenMWConfiguration;

/// Path to input bindings and core configuration
/// These functions are not expected to fail and should they fail, indicate either:
/// a severe issue with the system
/// or that an unsupported system is being used.
pub fn default_config_path() -> std::path::PathBuf {
    if cfg!(windows) {
        dirs::document_dir()
            .expect("FAILURE: COULD NOT READ CONFIG DIRECTORY")
            .join("openmw")
    } else {
        dirs::preference_dir()
            .expect("FAILURE: COULD NOT READ CONFIG DIRECTORY")
            .join("openmw")
    }
}

/// Path to save storage, screenshots, navmeshdb, and data-local
/// These functions are not expected to fail and should they fail, indicate either:
/// a severe issue with the system
/// or that an unsupported system is being used.
pub fn default_userdata_path() -> std::path::PathBuf {
    if cfg!(windows) {
        default_config_path()
    } else {
        dirs::data_dir()
            .expect("FAILURE: COULD NOT READ USERDATA DIRECTORY")
            .join("openmw")
    }
}
