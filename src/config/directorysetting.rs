use crate::config::strings;
use std::{fmt, path::PathBuf};

#[derive(Debug, Clone)]
pub struct DirectorySetting {
    pub meta: crate::GameSettingMeta,
    original: String,
    parsed: PathBuf,
}

impl std::fmt::Display for DirectorySetting {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "data={}", self.original)
    }
}

impl crate::GameSetting for DirectorySetting {
    fn meta(&self) -> &crate::GameSettingMeta {
        &self.meta
    }
}

impl From<(String, PathBuf)> for DirectorySetting {
    fn from((data_dir, source_config): (String, PathBuf)) -> Self {
        let original = data_dir.clone();
        let parsed = strings::parse_data_directory(&source_config, data_dir);
        let meta = crate::GameSettingMeta { source_config };

        DirectorySetting {
            original,
            parsed,
            meta,
        }
    }
}

impl DirectorySetting {
    pub fn new<S: Into<String>>(value: S, source_config: Option<PathBuf>) -> Self {
        let original = value.into();
        let parsed = strings::parse_data_directory(
            source_config
                .as_ref()
                .unwrap_or(&PathBuf::from("<internal>")),
            original.clone(),
        );

        let meta = crate::GameSettingMeta {
            source_config: source_config.unwrap_or_else(|| PathBuf::from("<internal>")),
        };

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
