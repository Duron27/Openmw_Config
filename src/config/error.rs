use std::{fmt, path::PathBuf};

#[macro_export]
macro_rules! config_err {
    // InvalidGameSetting: value, path
    (invalid_game_setting, $value:expr, $path:expr) => {
        $crate::ConfigError::InvalidGameSetting {
            value: $value.to_string(),
            config_path: $path.to_path_buf(),
        }
    };

    (not_file_or_directory, $config_path:expr) => {
        $crate::ConfigError::NotFileOrDirectory($config_path.to_path_buf())
    };

    (cannot_find, $config_path:expr) => {
        $crate::ConfigError::CannotFind($config_path.to_path_buf())
    };

    (duplicate_content_file, $content_file:expr, $config_path:expr) => {
        $crate::ConfigError::DuplicateContentFile {
            file: $content_file,
            config_path: $config_path.to_path_buf(),
        }
    };

    (bad_encoding, $encoding:expr, $config_path:expr) => {
        $crate::ConfigError::BadEncoding {
            value: $encoding,
            config_path: $config_path,
        }
    };

    // Parse error: single string
    (parse, $msg:expr) => {
        $crate::ConfigError::Parse($msg.to_string())
    };

    // Wrap std::io::Error
    (io, $err:expr) => {
        $crate::ConfigError::Io($err)
    };
}

#[macro_export]
macro_rules! bail_config {
    ($($tt:tt)*) => {
        {
        return Err($crate::config_err!($($tt)*));
    }
};
}

#[derive(Debug)]
pub enum ConfigError {
    DuplicateContentFile { file: String, config_path: PathBuf },
    InvalidGameSetting { value: String, config_path: PathBuf },
    BadEncoding { value: String, config_path: PathBuf },
    Io(std::io::Error),
    NotFileOrDirectory(PathBuf),
    CannotFind(PathBuf),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::InvalidGameSetting { value, config_path } => {
                write!(
                    f,
                    "Invalid fallback setting '{}' in config file '{}'",
                    value,
                    config_path.display()
                )
            }
            ConfigError::Io(e) => write!(f, "IO error: {}", e),
            ConfigError::NotFileOrDirectory(config_path) => write!(
                f,
                "Unable to determine whether {config_path:?} was a file or directory, refusing to read."
            ),
            ConfigError::CannotFind(config_path) => {
                write!(f, "An openmw.cfg does not exist at: {config_path:?}")
            }
            ConfigError::DuplicateContentFile { file, config_path } => write!(
                f,
                "{file} has appeared in the content files list twice. Its second occurence was in: {config_path:?}",
            ),
            ConfigError::BadEncoding { value, config_path } => {
                write!(
                    f,
                    "Invalid encoding type: {value} in config file {config_path:?}",
                )
            }
        }
    }
}

impl std::error::Error for ConfigError {}

impl From<std::io::Error> for ConfigError {
    fn from(err: std::io::Error) -> Self {
        ConfigError::Io(err)
    }
}
