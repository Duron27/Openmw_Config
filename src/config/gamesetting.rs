use std::fmt;

use crate::{GameSetting, GameSettingMeta};

#[derive(Debug, Clone)]
pub struct ColorGameSetting {
    meta: GameSettingMeta,
    value: (u8, u8, u8),
}

impl std::fmt::Display for ColorGameSetting {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (r, g, b) = self.value;
        writeln!(f, "{}", format!("{r},{g},{b}"))
    }
}
#[derive(Debug, Clone)]
pub struct StringGameSetting {
    meta: GameSettingMeta,
    value: String,
}

impl std::fmt::Display for StringGameSetting {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", format!("{}", self.value))
    }
}

#[derive(Debug, Clone)]
pub struct FloatGameSetting {
    meta: GameSettingMeta,
    value: f64,
}

impl std::fmt::Display for FloatGameSetting {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", format!("{}", self.value))
    }
}

#[derive(Debug, Clone)]
pub struct IntGameSetting {
    meta: GameSettingMeta,
    value: i64,
}

impl std::fmt::Display for IntGameSetting {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", format!("{}", self.value))
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

impl From<(String, std::path::PathBuf)> for GameSettingType {
    fn from((value, source_config): (String, std::path::PathBuf)) -> Self {
        let meta = GameSettingMeta { source_config };

        if let Some(color) = parse_color_value(&value) {
            return GameSettingType::Color(ColorGameSetting { meta, value: color });
        }

        if value.contains('.') {
            if let Ok(f) = value.parse::<f64>() {
                return GameSettingType::Float(FloatGameSetting { meta, value: f });
            }
        }

        if let Ok(i) = value.parse::<i64>() {
            return GameSettingType::Int(IntGameSetting { meta, value: i });
        }

        GameSettingType::String(StringGameSetting { meta, value })
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
