use std::{fs::metadata, path::PathBuf};

pub fn debug_log(message: String) {
    if std::env::var("CFG_DEBUG").is_ok() {
        println!("[CONFIG DEBUG]: {message}")
    }
}

pub fn user_config_path(sub_configs: &Vec<PathBuf>, fallthrough_dir: &PathBuf) -> PathBuf {
    // dbg!(&self.sub_configs);
    sub_configs
        .iter()
        .last()
        .unwrap_or(fallthrough_dir)
        .to_owned()
}

pub fn user_config_writable(path: &PathBuf) -> bool {
    metadata(path)
        .map(|m| m.permissions().readonly() == false)
        .unwrap_or(false)
}
