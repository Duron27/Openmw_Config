use std::{
    collections::HashMap,
    fmt::{self, Display},
    fs::{OpenOptions, create_dir_all, metadata, read_to_string},
    path::{Path, PathBuf},
};

use crate::{
    ConfigError, GameSetting, bail_config,
    config::{self, util::user_config_path},
};

mod directorysetting;
use directorysetting::DirectorySetting;

mod gamesetting;
use gamesetting::GameSettingType;

mod encodingsetting;
use encodingsetting::EncodingSetting;

#[macro_use]
pub mod error;
#[macro_use]
mod singletonsetting;
mod strings;
mod util;

#[derive(Clone, Debug)]
pub enum SettingValue {
    DataDirectory(DirectorySetting),
    GameSetting(GameSettingType),
    UserData(DirectorySetting),
    DataLocal(DirectorySetting),
    Resources(DirectorySetting),
    Encoding(EncodingSetting),
    SubConfiguration(DirectorySetting),
}

impl Display for SettingValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let str = match self {
            SettingValue::Encoding(encoding_setting) => encoding_setting.to_string(),
            SettingValue::UserData(userdata_setting) => format!(
                "{}userdata={}",
                userdata_setting.meta().comment,
                userdata_setting.original()
            ),
            SettingValue::DataLocal(data_local_setting) => format!(
                "{}data-local={}",
                data_local_setting.meta().comment,
                data_local_setting.original(),
            ),
            SettingValue::Resources(resources_setting) => format!(
                "{}resources={}",
                resources_setting.meta().comment,
                resources_setting.original()
            ),
            SettingValue::GameSetting(game_setting) => game_setting.to_string(),
            SettingValue::DataDirectory(data_directory) => format!(
                "{}data={}",
                data_directory.meta().comment,
                data_directory.original()
            ),
            SettingValue::SubConfiguration(sub_config) => format!(
                "{}config={}",
                sub_config.meta().comment,
                sub_config.original()
            ),
        };

        write!(f, "{str}")
    }
}

impl From<GameSettingType> for SettingValue {
    fn from(g: GameSettingType) -> Self {
        SettingValue::GameSetting(g)
    }
}

impl From<DirectorySetting> for SettingValue {
    fn from(d: DirectorySetting) -> Self {
        SettingValue::DataDirectory(d)
    }
}

impl SettingValue {
    // pub fn value(&self) ->
}

macro_rules! insert_dir_setting {
    ($self:ident, $variant:ident, $value:expr, $config_dir:expr, $comment:expr) => {{
        let actual_dir = match $config_dir.is_dir() {
            true => $config_dir,
            false => {
                if $config_dir.is_file() {
                    $config_dir.parent().expect("")
                } else {
                    bail_config!(not_file_or_directory, Path::new($value));
                }
            }
        };

        $self
            .settings
            .push(SettingValue::$variant(DirectorySetting::new(
                $value,
                actual_dir.to_path_buf(),
                $comment,
            )));
    }};
}

macro_rules! impl_path_selector {
    ($fn_name:ident, $variant:ident) => {
        pub fn $fn_name(&self) -> impl Iterator<Item = &PathBuf> {
            self.settings.iter().filter_map(|setting| match setting {
                SettingValue::$variant(inner) => Some(inner.parsed()),
                _ => None,
            })
        }
    };
}

/// Core struct representing the composed OpenMW configuration,
/// After it has been fully resolved.
/// The overall configuration itself is immutable after its construction
#[derive(Debug, Default)]
pub struct OpenMWConfiguration {
    root_config: PathBuf,
    sub_configs: Vec<PathBuf>,
    content_files: Vec<String>,
    fallback_archives: Vec<String>,
    data_local: Option<PathBuf>,
    userdata: Option<DirectorySetting>,
    resources: Option<PathBuf>,
    /// Unrecognized or otherwise not-super-important values
    generic: HashMap<String, String>,
    /// Maps k/v pairs to all preceding comments during parsing.
    comments: HashMap<String, Vec<String>>,
    /// Orphaned or trailing comments are preserved at the end of the configuration.
    trailing_comments: HashMap<String, String>,
    game_settings: HashMap<String, GameSettingType>,
    settings: Vec<SettingValue>,
}

impl OpenMWConfiguration {
    pub fn new(path: Option<PathBuf>) -> Result<Self, ConfigError> {
        let mut config = OpenMWConfiguration::default();
        let root_config = match path {
            Some(path) => util::input_config_path(&path)?,
            None => crate::default_config_path().join("openmw.cfg"),
        };

        match config.load(&root_config) {
            Err(error) => Err(error),
            Ok(_) => {
                config.root_config = root_config.clone();

                if let Some(dir) = &config.data_local {
                    let data_local_setting: SettingValue = DirectorySetting::new(
                        dir.to_string_lossy().to_string(),
                        root_config.clone(),
                        &mut String::default(),
                    )
                    .into();

                    config.settings.push(data_local_setting);

                    let dir_meta = metadata(dir);
                    if !dir_meta.is_ok() {
                        if let Err(error) = create_dir_all(dir) {
                            eprintln!(
                                "WARNING: Attempted to crete a data-local directory at {dir:?}, but failed: {error}"
                            )
                        };
                    }
                }

                if let Some(dir) = &config.resources {
                    let morrowind_vfs: SettingValue = DirectorySetting::new(
                        dir.join("vfs-mw").to_string_lossy().to_string(),
                        root_config.clone(),
                        &mut String::default(),
                    )
                    .into();

                    let engine_vfs: SettingValue = DirectorySetting::new(
                        dir.join("vfs").to_string_lossy().to_string(),
                        root_config,
                        &mut String::default(),
                    )
                    .into();

                    config.settings.insert(0, morrowind_vfs);
                    config.settings.insert(0, engine_vfs);
                }

                util::debug_log(format!("{:#?}\n{:#?}", config.settings, config.sub_configs));

                config
                    .settings
                    .iter()
                    .for_each(|setting_value| println!("{setting_value}"));

                Ok(config)
            }
        }
    }

    /// Path to the configuration file which is the root of the configuration chain
    /// Typically, this will be whatever is defined in the `Paths` documentation for the appropriate platform:
    /// https://openmw.readthedocs.io/en/latest/reference/modding/paths.html#configuration-files-and-log-files
    pub fn root_config_file(&self) -> &PathBuf {
        &self.root_config
    }

    /// Same as root_config_file, but returns the directory it's in.
    /// Useful for reading other configuration files, or if assuming openmw.cfg
    /// Is always *called* openmw.cfg (which it should be)
    pub fn root_config_dir(&self) -> PathBuf {
        self.root_config.parent().expect("").to_path_buf()
    }

    /// In order of priority, the list of all openmw.cfg files which were loaded by the configuration chain after the root.
    /// If the root openmw.cfg is different than the user one, this list will contain the user openmw.cfg as its last element.
    /// If the root and user openmw.cfg are the *same*, then this list will be empty and the root config should be considered the user config.
    /// Otherwise, if one wishes to get the contents of the user configuration specifically, construct a new OpenMWConfiguration from the last sub_config.
    ///
    /// Openmw.cfg files are added in order of the sequence in which they are defined by one openmw.cfg, and then each of *those* openmw.cfg files
    /// is then processed in their entirety, sequentially, after the first one has resolved.
    /// The highest-priority openmw.cfg loaded (the last one!) is considered the user openmw.cfg,
    /// and will be the one which is modifiable by OpenMW-Launcher and OpenMW proper.
    ///
    /// See https://openmw.readthedocs.io/en/latest/reference/modding/paths.html#configuration-sources for examples and further explanation of multiple config sources.

    /// Path to the highest-level configuration *directory*
    pub fn user_config_path(&self) -> PathBuf {
        util::user_config_path(&self.sub_configs().collect(), &self.root_config_dir())
    }

    impl_path_selector!(sub_configs, SubConfiguration);
    impl_path_selector!(data_directories, DataDirectory);

    impl_singleton_setting! {
        UserData => {
            get: userdata,
            set: set_userdata,
            in_type: DirectorySetting
        },
        Resources => {
            get: resources,
            set: set_resources,
            in_type: DirectorySetting
        },
        DataLocal => {
            get: data_local,
            set: set_data_local,
            in_type: DirectorySetting
        },
        Encoding => {
            get: encoding,
            set: set_encoding,
            in_type: EncodingSetting
        }
    }

    /// Content files are the actual *mods* or plugins which are created by either OpenCS or Bethesda's construction set
    /// These entries only refer to the names and ordering of content files.
    /// vfstool-lib should be used to derive paths
    pub fn content_files(&self) -> &Vec<String> {
        &self.content_files
    }

    /// This early iteration of the crate provides no input validation for setter functions.
    pub fn set_content_files(&mut self, plugins: Vec<String>) {
        self.content_files = plugins
    }

    /// Data directories are the bulk of an OpenMW Configuration's contents,
    /// Composing the list of files from which a VFS is constructed.
    /// For a VFS implementation, see: https://github.com/magicaldave/vfstool/tree/main/vfstool_lib
    ///
    /// Calling this function will give the post-parsed versions of directories defined by an openmw.cfg,
    /// So the real ones may easily be iterated and loaded.
    /// There is not actually validation anywhere in the crate that DirectorySettings refer to a directory which actually exists.
    /// This is according to the openmw.cfg specification and doesn't technically break anything but should be considered when using these paths.

    /// This early iteration of the crate provides no input validation for setter functions.
    pub fn set_data_directories(&mut self, dirs: Option<Vec<PathBuf>>) {
        self.settings.retain(|setting| match setting {
            SettingValue::DataDirectory(_) => false,
            _ => true,
        });

        if let Some(dirs) = dirs {
            let config_path = self.user_config_path();
            let mut empty = String::default();

            dirs.iter().for_each(|dir| {
                self.settings
                    .push(SettingValue::DataDirectory(DirectorySetting::new(
                        dir.to_string_lossy(),
                        config_path.clone(),
                        &mut empty,
                    )))
            })
        }
    }

    /// Fallback entries are k/v pairs baked into the value side of k/v pairs in `fallback=` entries of openmw.cfg
    /// They are used to express settings which are defined in Morrowind.ini for things such as:
    /// weather, lighting behaviors, UI Colors, and levelup messages
    pub fn game_settings(&self) -> &HashMap<String, GameSettingType> {
        &self.game_settings
    }

    /// This early iteration of the crate provides no input validation for setter functions.
    pub fn set_game_settings(&mut self, entries: HashMap<String, GameSettingType>) {
        self.game_settings = entries
    }

    /// List of filenames of Bethesda Archive files to use in the composed configuration
    pub fn fallback_archives(&self) -> &Vec<String> {
        &self.fallback_archives
    }

    /// This early iteration of the crate provides no input validation for setter functions.
    pub fn set_fallback_archives(&mut self, archives: Vec<String>) {
        self.fallback_archives = archives
    }

    fn load(&mut self, config_dir: &Path) -> Result<(), ConfigError> {
        util::debug_log(format!("BEGIN CONFIG PARSING: {config_dir:?}"));

        if !config_dir.exists() {
            bail_config!(cannot_find, config_dir);
        }

        let cfg_file_path = match config_dir.is_dir() {
            true => config_dir.join("openmw.cfg"),
            false => config_dir.to_path_buf(),
        };

        let lines = read_to_string(&cfg_file_path)?;

        let mut queued_comment = String::new();
        let mut sub_configs: Vec<(String, String)> = Vec::new();

        for line in lines.lines() {
            let trimmed = line.trim();

            if trimmed.is_empty() {
                continue;
            } else if trimmed.starts_with('#') {
                queued_comment.push_str(line);
                queued_comment.push('\n');
                continue;
            }

            let tokens: Vec<&str> = trimmed.splitn(2, '=').collect();
            if tokens.len() < 2 {
                bail_config!(invalid_line, trimmed.into(), config_dir.to_path_buf());
            }

            let key = tokens[0].trim();
            let value = tokens[1].trim().to_string();

            match key {
                "content" => {
                    if self.content_files.contains(&value) {
                        bail_config!(duplicate_content_file, value, config_dir);
                    }
                    self.content_files.push(value);
                }
                "fallback-archive" => {
                    self.fallback_archives.push(value);
                }
                "fallback" => {
                    self.settings.push(
                        GameSettingType::try_from((
                            value,
                            config_dir.to_owned(),
                            &mut queued_comment,
                        ))?
                        .into(),
                    );
                }
                "encoding" => self.set_encoding(Some(EncodingSetting::try_from((
                    value,
                    config_dir,
                    &mut queued_comment,
                ))?)),
                "config" => {
                    sub_configs.push((value, queued_comment.clone()));
                    queued_comment.clear();
                }
                "data" => {
                    insert_dir_setting!(
                        self,
                        DataDirectory,
                        &value,
                        config_dir,
                        &mut queued_comment
                    )
                }
                "resources" => {
                    insert_dir_setting!(self, Resources, &value, config_dir, &mut queued_comment)
                }
                "userdata" => {
                    insert_dir_setting!(self, UserData, &value, config_dir, &mut queued_comment)
                }
                "data-local" => {
                    insert_dir_setting!(self, DataLocal, &value, config_dir, &mut queued_comment)
                }
                "replace" => match value.to_lowercase().as_str() {
                    "content" => self.content_files = Vec::new(),
                    "data" => self.set_data_directories(None),
                    "fallback" => self.game_settings = HashMap::new(),
                    "fallback-archives" => self.fallback_archives = Vec::new(),
                    "data-local" => self.set_data_local(None),
                    "resources" => self.set_resources(None),
                    "userdata" => self.set_userdata(None),
                    "config" => {
                        self.settings.clear();
                        // self.content_files = Vec::new();
                        // self.data_directories = Vec::new();
                        // self.fallback_archives = Vec::new();
                    }
                    _ => {
                        // eprintln!("Warning: Unrecognized replacement option: {value}")
                    }
                },
                _ => {
                    // eprintln!("Warning: Unrecognized configuration pair: {key}={value}");
                    self.generic.insert(key.to_string(), value);
                }
            }
        }

        // Store the trailing comments for a given openmw.cfg
        // By placing a copy of its absolute path in the comment map
        // Then, during reserialization, rewrite the trailing comments with their own set of
        // comments according to what configuration file they belong to
        self.trailing_comments.insert(
            config_dir.to_string_lossy().to_ascii_lowercase(),
            queued_comment,
        );

        // This shit with file/directory is very hard to keep track of and should be refactored post-release, but for now it isn't important
        let cfg_file_path = match config_dir.is_dir() {
            true => config_dir,
            false => config_dir
                .parent()
                .ok_or_else(|| config_err!(cannot_find, config_dir))?,
        }
        .to_path_buf();

        sub_configs.into_iter().try_for_each(
            |(subconfig_path, mut subconfig_comment): (String, String)| {
                let mut comment = std::mem::take(&mut subconfig_comment);

                let setting: DirectorySetting = DirectorySetting::new(subconfig_path.clone(), cfg_file_path.clone(), &mut comment);
                let subconfig_path = setting.parsed().join("openmw.cfg");

                if std::fs::metadata(&subconfig_path).is_ok() {
                    self.settings.push(SettingValue::SubConfiguration(setting));
                    self.load(Path::new(&subconfig_path))
                } else {
                    util::debug_log(format!(
                        "Skipping parsing of {} As this directory does not actually contain an openmw.cfg!",
                        cfg_file_path.display(),
                    ));

                    Ok(())
                }
            },
        )?;

        Ok(())
    }

    fn write_config<P: AsRef<Path> + std::fmt::Debug>(&self, path: &P) -> Result<(), String> {
        use std::io::Write;
        let mut config_string = String::new();
        let mut comments = self.comments.clone();

        if let Some(ref resources) = self.resources {
            strings::write_comments(
                comments.remove(&resources.display().to_string()),
                &mut config_string,
            );
            strings::resources(&mut config_string, &self.resources)?;
        }

        if let Some(ref userdata) = self.userdata {
            strings::write_comments(comments.remove(userdata.original()), &mut config_string);
            strings::userdata(&mut config_string, userdata.original())?;
        }

        if let Some(ref data_local) = self.data_local {
            strings::write_comments(
                comments.remove(&data_local.display().to_string()),
                &mut config_string,
            );
            strings::data_local(&mut config_string, &self.data_local)?;
        }

        for archive in &self.fallback_archives {
            strings::write_comments(comments.remove(archive.as_str()), &mut config_string);
            strings::fallback_archive(&mut config_string, &archive)?;
        }

        // Content files
        for content in &self.content_files {
            strings::write_comments(comments.remove(content.as_str()), &mut config_string);
            strings::content_file(&mut config_string, &content)?;
        }

        for (key, value) in &self.game_settings {
            let fallback_entry_comment_key = format!("{key},{value}");

            strings::write_comments(
                comments.remove(&fallback_entry_comment_key),
                &mut config_string,
            );

            strings::fallback_entry(&mut config_string, &key, &value.to_string())?;
        }

        for (config, comments) in &self.trailing_comments {
            if comments.len() == 0 {
                continue;
            };

            config_string.push_str(&format!("\n# Trailing comments defined by: {config} #\n"));
            config_string.push_str(comments);
            config_string.push('\n');
        }

        let mut file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(&path)
            .map_err(|e| format!("Failed to open {:?} for writing: {}", path, e))?;

        file.write_all(config_string.as_bytes())
            .map_err(|e| format!("Failed to write to {:?}: {}", path, e))?;

        Ok(())
    }

    /// Saves the current composite configuration to whatever the designated user config path is.
    /// Note that this function does not validate that your saved configuration makes *sense*, or is what you intended for it to be.
    /// Calling this method on an openmw.cfg which, itself calls other openmw.cfg files, can have unintended effects, so it should *always*
    /// be ran against the user openmw.cfg.
    ///
    /// This is a rather unavoidable side effect of the nature of openmw.cfg's flattened data structure.
    /// Should you wish to run the `save` method, it is your responsibility to ensure you use it on the right configuration.
    ///
    /// It is recommended that, if used, this function should be called upon the most narrowly-scoped
    /// openmw.cfg which can reasonably be used.
    pub fn save(&self) -> Result<(), String> {
        let target_dir = self.user_config_path();

        // Check if target_dir is a writable directory
        if !target_dir.is_dir() {
            return Err(format!("Target path {:?} is not a directory.", target_dir));
        }

        // Try to open a file for writing to check writability
        if !util::can_write_to_dir(&target_dir) {
            return Err(format!("Directory {:?} is not writable!", target_dir));
        };

        // Write the config to openmw.cfg in the target directory
        let cfg_path = target_dir.join("openmw.cfg");
        self.write_config(&cfg_path)?;

        Ok(())
    }

    /// Save the openmw.cfg to an arbitrary path, instead of the (safe) user configuration.
    /// This doesn't prevent bad usages of the configuration such as overriding an existing one with the original root configuration,
    /// So you should exercise caution when writing an openmw.cfg and be very sure you know it is going where you think it is.
    pub fn save_path(&self, path: PathBuf) -> Result<(), String> {
        let target_dir = path.parent().expect(&format!(
            "Could not get parent directory of the path: {path:?} to write openmw.cfg!"
        ));

        // Check if target_dir is a writable directory
        if !target_dir.is_dir() {
            return Err(format!("Target path {:?} is not a directory.", target_dir));
        }

        // Try to open a file for writing to check writability
        if !util::can_write_to_dir(&target_dir) {
            return Err(format!("Directory {:?} is not writable!", target_dir));
        };

        // Write the config to openmw.cfg in the target directory
        self.write_config(&path)?;

        Ok(())
    }
}

/// Keep in mind this is *not* meant to be used as a mechanism to write the openmw.cfg contents.
/// Since the openmw.cfg is a merged entity, it is impossible to distinguish the origin of one particular data directory
/// Or content file once it has been applied - this is doubly true for entries which may only exist once in openmw.cfg.
/// Thus, what this method provides is the composite configuration.
///
/// It may be safely used to write an openmw.cfg as all directories will be absolutized upon loading the config.
///
/// Token information is also lost when a config file is processed.
/// It is not necessarily recommended to write a configuration file which loads other ones or uses tokens for this reason.
///
/// Comments are also preserved.
impl fmt::Display for OpenMWConfiguration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Resources
        if let Some(ref resources) = self.resources {
            writeln!(f, "resources={}", resources.display())?;
        }

        // Userdata (not typically in openmw.cfg, but included for completeness)
        if let Some(ref userdata) = self.userdata {
            writeln!(f, "userdata={}", userdata)?;
        }

        // Data-local
        if let Some(ref data_local) = self.data_local {
            writeln!(f, "data-local={}", data_local.display())?;
        }

        // Fallback archives
        for archive in &self.fallback_archives {
            writeln!(f, "fallback-archive={}", archive)?;
        }

        // Data directories
        for dir in self.data_directories() {
            writeln!(f, "data={}", dir.display())?;
        }

        // Content files
        for content in &self.content_files {
            writeln!(f, "content={}", content)?;
        }

        // Fallback entries
        for (_, value) in &self.game_settings {
            writeln!(f, "{value}")?;
        }

        Ok(())
    }
}
