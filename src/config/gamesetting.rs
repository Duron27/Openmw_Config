// This file is part of Openmw_Config.
// Openmw_Config is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.
// Openmw_Config is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.
// You should have received a copy of the GNU General Public License along with Openmw_Config. If not, see <https://www.gnu.org/licenses/>.

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
            format!("{}fallback={},{}", self.meta.comment, self.key, self.value)
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
            format!("{}fallback={},{}", self.meta.comment, self.key, self.value)
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
            format!("{}fallback={},{}", self.meta.comment, self.key, self.value)
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

impl GameSettingType {
    pub fn key(&self) -> &String {
        match &self {
            &GameSettingType::Color(setting) => &setting.key,
            &GameSettingType::String(setting) => &setting.key,
            &GameSettingType::Float(setting) => &setting.key,
            &GameSettingType::Int(setting) => &setting.key,
        }
    }

    pub fn value(&self) -> String {
        match &self {
            &GameSettingType::Color(setting) => {
                let (r, g, b) = setting.value;
                format!("{r},{g},{b}")
            }
            &GameSettingType::String(setting) => setting.value.clone(),
            &GameSettingType::Float(setting) => setting.value.to_string(),
            &GameSettingType::Int(setting) => setting.value.to_string(),
        }
    }
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

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    fn default_meta() -> GameSettingMeta {
        GameSettingMeta {
            source_config: PathBuf::default(),
            comment: String::default(),
        }
    }

    #[test]
    fn test_value_string_setting() {
        let setting = GameSettingType::String(StringGameSetting {
            meta: default_meta(),
            key: "greeting".into(),
            value: "hello world".into(),
        });

        assert_eq!(setting.value(), "hello world");
    }

    #[test]
    fn test_value_int_setting() {
        let setting = GameSettingType::Int(IntGameSetting {
            meta: default_meta(),
            key: "MaxEyesOfTodd".into(),
            value: 3,
        });

        assert_eq!(setting.value(), "3");
    }

    #[test]
    fn test_value_float_setting() {
        let setting = GameSettingType::Float(FloatGameSetting {
            meta: default_meta(),
            key: "FLightAttenuationEnfuckulation".into(),
            value: 0.75,
        });

        assert_eq!(setting.value(), "0.75");
    }

    #[test]
    fn test_value_color_setting() {
        let setting = GameSettingType::Color(ColorGameSetting {
            meta: default_meta(),
            key: "hud_color".into(),
            value: (255, 128, 64),
        });

        assert_eq!(setting.value(), "255,128,64");
    }

    #[test]
    fn test_to_string_for_string_setting() {
        let setting = GameSettingType::String(StringGameSetting {
            meta: default_meta(),
            key: "sGreeting".into(),
            value: "Hello, Nerevar.".into(),
        });

        assert_eq!(setting.to_string(), "fallback=sGreeting,Hello, Nerevar.");
    }

    #[test]
    fn test_to_string_for_int_setting() {
        let setting = GameSettingType::Int(IntGameSetting {
            meta: default_meta(),
            key: "iMaxSpeed".into(),
            value: 42,
        });

        assert_eq!(setting.to_string(), "fallback=iMaxSpeed,42");
    }

    #[test]
    fn test_to_string_for_float_setting() {
        let setting = GameSettingType::Float(FloatGameSetting {
            meta: default_meta(),
            key: "fJumpHeight".into(),
            value: 1.75,
        });

        assert_eq!(setting.to_string(), "fallback=fJumpHeight,1.75");
    }

    #[test]
    fn test_to_string_for_color_setting() {
        let setting = GameSettingType::Color(ColorGameSetting {
            meta: default_meta(),
            key: "iHUDColor".into(),
            value: (128, 64, 255),
        });

        assert_eq!(setting.to_string(), "fallback=iHUDColor,128,64,255");
    }

    #[test]
    fn test_commented_string() {
        let setting = GameSettingType::Color(ColorGameSetting {
            meta: GameSettingMeta { source_config: PathBuf::from("$HOME/.config/openmw/openmw.cfg"), comment: String::from("#Monochrome UI Settings\n#\n#\n#\n#######\n##\n##\n##\n") },
            key: "iHUDColor".into(),
            value: (128, 64, 255),
        });

        assert_eq!(setting.to_string(), "#Monochrome UI Settings\n#\n#\n#\n#######\n##\n##\n##\nfallback=iHUDColor,128,64,255");
    }
}
