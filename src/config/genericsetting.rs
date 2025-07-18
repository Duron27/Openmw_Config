// This file is part of Openmw_Config.
// Openmw_Config is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.
// Openmw_Config is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.
// You should have received a copy of the GNU General Public License along with Openmw_Config. If not, see <https://www.gnu.org/licenses/>.

use crate::{GameSetting, GameSettingMeta};
use std::fmt;

#[derive(Debug, Clone)]
pub struct GenericSetting {
    meta: GameSettingMeta,
    key: String,
    value: String,
}

impl GameSetting for GenericSetting {
    fn meta(&self) -> &GameSettingMeta {
        &self.meta
    }
}

impl fmt::Display for GenericSetting {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}={}", self.meta.comment, self.key, self.value)
    }
}

impl GenericSetting {
    pub fn new(
        key: &str,
        value: &str,
        source_config: &std::path::Path,
        comment: &mut String,
    ) -> Self {
        Self {
            meta: GameSettingMeta {
                source_config: source_config.to_path_buf(),
                comment: std::mem::take(comment),
            },
            key: key.to_string(),
            value: value.to_string(),
        }
    }
}
