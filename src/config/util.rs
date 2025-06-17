pub fn debug_log(message: String) {
    if std::env::var("CFG_DEBUG").is_ok() {
        println!("[CONFIG DEBUG]: {message}")
    }
}

pub fn user_config_path(
    sub_configs: &Vec<&std::path::PathBuf>,
    fallthrough_dir: &std::path::PathBuf,
) -> std::path::PathBuf {
    sub_configs
        .into_iter()
        .last()
        .unwrap_or(&fallthrough_dir)
        .to_path_buf()
}

pub fn user_config_writable(path: &std::path::PathBuf) -> bool {
    std::fs::metadata(path)
        .map(|m| m.permissions().readonly() == false)
        .unwrap_or(false)
}

pub fn can_write_to_dir<P: AsRef<std::path::Path>>(dir: &P) -> bool {
    let test_path = dir.as_ref().join(".openmw_cfg_write_test");
    match std::fs::File::create(&test_path) {
        Ok(_) => {
            let _ = std::fs::remove_file(&test_path);
            true
        }
        Err(_) => false,
    }
}

/// Transposes an input directory or file path to an openmw.cfg path
/// Maybe could do with some additional validation
pub fn input_config_path(
    config_path: &std::path::Path,
) -> Result<std::path::PathBuf, crate::ConfigError> {
    match std::fs::symlink_metadata(config_path) {
        Ok(metadata) => {
            if metadata.is_file() {
                Ok(config_path.to_path_buf())
            } else if metadata.is_dir() {
                let maybe_config = config_path.join("openmw.cfg");
                if maybe_config.is_file() {
                    Ok(maybe_config)
                } else {
                    crate::config::bail_config!(cannot_find, config_path);
                }
            } else {
                crate::config::bail_config!(not_file_or_directory, config_path);
            }
        }
        Err(err) => {
            if err.kind() == std::io::ErrorKind::NotFound {
                crate::config::bail_config!(not_file_or_directory, config_path);
            } else {
                Err(crate::ConfigError::Io(err))
            }
        }
    }
}
