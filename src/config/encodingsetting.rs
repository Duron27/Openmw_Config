use std::fmt;

use crate::{ConfigError, GameSetting, GameSettingMeta, bail_config};

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum EncodingType {
    WIN1250,
    WIN1251,
    WIN1252,
}

impl std::fmt::Display for EncodingType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = match self {
            EncodingType::WIN1250 => "win1250",
            EncodingType::WIN1251 => "win1251",
            EncodingType::WIN1252 => "win1252",
        };

        writeln!(f, "{value}")
    }
}

#[derive(Debug, Clone)]
pub struct EncodingSetting {
    meta: GameSettingMeta,
    encoding: EncodingType,
}

impl PartialEq for EncodingSetting {
    fn eq(&self, other: &Self) -> bool {
        self.encoding == other.encoding
    }
}

impl GameSetting for EncodingSetting {
    fn meta(&self) -> &GameSettingMeta {
        &self.meta
    }
}

impl fmt::Display for EncodingSetting {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", format!("encoding={}", self.encoding))
    }
}

impl<P: AsRef<std::path::Path>> TryFrom<(String, P)> for EncodingSetting {
    type Error = ConfigError;

    fn try_from((value, config_path): (String, P)) -> Result<Self, Self::Error> {
        let config_path = config_path.as_ref().to_path_buf();

        let encoding = match value.as_str() {
            "win1250" => EncodingType::WIN1250,
            "win1251" => EncodingType::WIN1251,
            "win1252" => EncodingType::WIN1252,
            _ => bail_config!(bad_encoding, value, config_path),
        };

        let meta = GameSettingMeta {
            source_config: config_path,
        };

        Ok(EncodingSetting { encoding, meta })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn dummy_path() -> PathBuf {
        PathBuf::from("/tmp/test.cfg")
    }

    #[test]
    fn test_valid_encodings() {
        let encodings = vec![
            ("win1250", EncodingType::WIN1250),
            ("win1251", EncodingType::WIN1251),
            ("win1252", EncodingType::WIN1252),
        ];

        for (input, expected_variant) in encodings {
            let setting = EncodingSetting::try_from((input.to_string(), dummy_path())).unwrap();
            assert_eq!(setting.encoding, expected_variant);
        }
    }

    #[test]
    fn test_invalid_encoding() {
        let err = EncodingSetting::try_from(("utf8".to_string(), dummy_path()));
        assert!(matches!(err, Err(ConfigError::BadEncoding { .. })));
    }

    #[test]
    fn test_empty_encoding_string() {
        let err = EncodingSetting::try_from(("".to_string(), dummy_path()));
        assert!(matches!(err, Err(ConfigError::BadEncoding { .. })));
    }

    #[test]
    fn test_source_path_preservation() {
        let path = PathBuf::from("/some/path/to/config.cfg");
        let setting = EncodingSetting::try_from(("win1251".to_string(), path.as_path())).unwrap();
        assert_eq!(setting.meta.source_config, path);
    }

    #[test]
    fn test_display_trait_output() {
        let setting = EncodingSetting::try_from(("win1250".to_string(), dummy_path().as_path())).unwrap();
        let rendered = setting.to_string();
        assert_eq!(rendered.trim(), "encoding=win1250");
    }
}

