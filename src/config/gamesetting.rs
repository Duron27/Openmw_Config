use std::fmt;

use crate::{ConfigError, GameSetting, GameSettingMeta, bail_config};

#[derive(Debug, Clone)]
pub struct ColorGameSetting {
    meta: GameSettingMeta,
    key: String,
    value: (u8, u8, u8),
}

impl std::fmt::Display for ColorGameSetting {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (r, g, b) = self.value;
        write!(
            f,
            "{}",
            format!("{}fallback={},{r},{g},{b}", self.meta.comment, self.key)
        )
    }
}

#[derive(Debug, Clone)]
pub struct StringGameSetting {
    meta: GameSettingMeta,
    key: String,
    value: String,
}

impl std::fmt::Display for StringGameSetting {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            format!(
                "{}fallback={},{}",
                self.meta.comment, self.key, self.value
            )
        )
    }
}

#[derive(Debug, Clone)]
pub struct FloatGameSetting {
    meta: GameSettingMeta,
    key: String,
    value: f64,
}

impl std::fmt::Display for FloatGameSetting {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            format!(
                "{}fallback={},{}",
                self.meta.comment, self.key, self.value
            )
        )
    }
}

#[derive(Debug, Clone)]
pub struct IntGameSetting {
    meta: GameSettingMeta,
    key: String,
    value: i64,
}

impl std::fmt::Display for IntGameSetting {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            format!(
                "{}fallback={},{}",
                self.meta.comment, self.key, self.value
            )
        )
    }
}

#[derive(Debug, Clone)]
pub enum GameSettingType {
    Color(ColorGameSetting),
    String(StringGameSetting),
    Float(FloatGameSetting),
    Int(IntGameSetting),
}

impl std::fmt::Display for GameSettingType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GameSettingType::Color(s) => write!(f, "{}", s),
            GameSettingType::Float(s) => write!(f, "{}", s),
            GameSettingType::String(s) => write!(f, "{}", s),
            GameSettingType::Int(s) => write!(f, "{}", s),
        }
    }
}

impl GameSetting for GameSettingType {
    fn meta(&self) -> &GameSettingMeta {
        match self {
            GameSettingType::Color(s) => &s.meta,
            GameSettingType::String(s) => &s.meta,
            GameSettingType::Float(s) => &s.meta,
            GameSettingType::Int(s) => &s.meta,
        }
    }
}

impl PartialEq for GameSettingType {
    fn eq(&self, other: &Self) -> bool {
        use GameSettingType::*;

        match (self, other) {
            (Color(a), Color(b)) => a.key == b.key,
            (String(a), String(b)) => a.key == b.key,
            (Float(a), Float(b)) => a.key == b.key,
            (Int(a), Int(b)) => a.key == b.key,
            // Mismatched types should never be considered equal
            _ => false,
        }
    }
}

impl PartialEq<&str> for GameSettingType {
    fn eq(&self, other: &&str) -> bool {
        use GameSettingType::*;

        match self {
            Color(a) => a.key == *other,
            String(a) => a.key == *other,
            Float(a) => a.key == *other,
            Int(a) => a.key == *other,
        }
    }
}

impl Eq for GameSettingType {}

impl TryFrom<(String, std::path::PathBuf, &mut String)> for GameSettingType {
    type Error = ConfigError;

    fn try_from(
        (original_value, source_config, queued_comment): (String, std::path::PathBuf, &mut String),
    ) -> Result<Self, ConfigError> {
        let tokens: Vec<&str> = original_value.splitn(2, ',').collect();

        if tokens.len() < 2 {
            bail_config!(invalid_game_setting, original_value, source_config);
        }

        let key = tokens[0].to_string();
        let value = tokens[1].to_string();

        let meta = GameSettingMeta {
            source_config,
            comment: queued_comment.clone(),
        };

        queued_comment.clear();

        if let Some(color) = parse_color_value(&value) {
            return Ok(GameSettingType::Color(ColorGameSetting {
                meta,
                key,
                value: color,
            }));
        }

        if value.contains('.') {
            if let Ok(f) = value.parse::<f64>() {
                return Ok(GameSettingType::Float(FloatGameSetting {
                    meta,
                    key,
                    value: f,
                }));
            }
        }

        if let Ok(i) = value.parse::<i64>() {
            return Ok(GameSettingType::Int(IntGameSetting {
                meta,
                key,
                value: i,
            }));
        }

        Ok(GameSettingType::String(StringGameSetting {
            meta,
            key,
            value,
        }))
    }
}

fn parse_color_value(value: &str) -> Option<(u8, u8, u8)> {
    let parts: Vec<_> = value
        .split(',')
        .map(str::trim)
        .filter_map(|s| s.parse::<u8>().ok())
        .collect();

    match parts.as_slice() {
        [r, g, b] => Some((*r, *g, *b)),
        _ => None,
    }
}
