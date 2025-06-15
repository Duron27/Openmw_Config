use crate::config::strings;
use std::{fmt, path::PathBuf};

#[derive(Debug, Clone)]
pub struct DirectorySetting {
    pub meta: crate::GameSettingMeta,
    original: String,
    parsed: PathBuf,
}

/// This is tricky.
/// The trait implementation for GameSetting necessitates that all settings have a Display method.
/// However, DirectorySetting is reused interchangeably amongst variants that use a different key. So really the key should just be skipped here,
/// And handled by the upper SettingValue implementation?
/// But that, also, is fucked off, because then we wouldn't be able to handle comments.
/// So the hope I guess is that the SettingValue itself can have an implementation to account for this. 
/// That seems fair?
/// And then we just assume data= is the default in here.
impl std::fmt::Display for DirectorySetting {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", self.original)
    }
}

impl crate::GameSetting for DirectorySetting {
    fn meta(&self) -> &crate::GameSettingMeta {
        &self.meta
    }
}

impl DirectorySetting {
    pub fn new<S: Into<String>>(value: S, source_config: PathBuf, comment: &mut String) -> Self {
        let original = value.into();
        let parsed = strings::parse_data_directory(
            &source_config,
            original.clone(),
        );

        let meta = crate::GameSettingMeta {
            source_config: source_config,
            comment: comment.clone(),
        };
        comment.clear();

        Self {
            original,
            parsed,
            meta,
        }
    }

    pub fn original(&self) -> &String {
        &self.original
    }

    pub fn parsed(&self) -> &PathBuf {
        &self.parsed
    }
}
