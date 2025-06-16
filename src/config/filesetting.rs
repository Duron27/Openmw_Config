use crate::{GameSetting, GameSettingMeta};
use std::fmt;

#[derive(Debug, Clone)]
pub struct FileSetting {
    meta: GameSettingMeta,
    value: String,
}

impl PartialEq for FileSetting {
    fn eq(&self, other: &Self) -> bool {
        &self.value == other.value()
    }
}

impl PartialEq<&str> for FileSetting {
    fn eq(&self, other: &&str) -> bool {
        self.value == *other
    }
}

impl PartialEq<str> for FileSetting {
    fn eq(&self, other: &str) -> bool {
        self.value == other
    }
}

impl PartialEq<&String> for FileSetting {
    fn eq(&self, other: &&String) -> bool {
        &self.value == *other
    }
}

impl GameSetting for FileSetting {
    fn meta(&self) -> &GameSettingMeta {
        &self.meta
    }
}

impl fmt::Display for FileSetting {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl FileSetting {
    pub fn new(value: &str, source_config: &std::path::Path, comment: &mut String) -> Self {
        Self {
            meta: GameSettingMeta {
                source_config: source_config.to_path_buf(),
                comment: std::mem::take(comment),
            },
            value: value.to_string(),
        }
    }

    pub fn value(&self) -> &String {
        &self.value
    }
}
