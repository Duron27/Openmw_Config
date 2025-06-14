use std::{
    collections::HashMap,
    fmt::{self, write},
    fs::{File, OpenOptions, create_dir_all, metadata, read_to_string, remove_file},
    path::{Path, PathBuf},
};

mod gamesetting;
mod strings;
mod util;

/// Core struct representing the composed OpenMW configuration,
/// After it has been fully resolved.
/// The overall configuration itself is immutable after its construction
#[derive(Debug, Default)]
pub struct OpenMWConfiguration {
    root_config: PathBuf,
    sub_configs: Vec<PathBuf>,
    data_directories: Vec<PathBuf>,
    content_files: Vec<String>,
    fallback_entries: HashMap<String, String>,
    fallback_archives: Vec<String>,
    data_local: Option<PathBuf>,
    userdata: Option<PathBuf>,
    resources: Option<PathBuf>,
    /// Unrecognized or otherwise not-super-important values
    generic: HashMap<String, String>,
    /// Maps k/v pairs to all preceding comments during parsing.
    comments: HashMap<String, Vec<String>>,
    /// Orphaned or trailing comments are preserved at the end of the configuration.
    trailing_comments: HashMap<String, Vec<String>>,
    game_settings: HashMap<String, gamesetting::GameSettingType>,
}

impl OpenMWConfiguration {
    pub fn new(path: Option<PathBuf>) -> Result<Self, String> {
        let mut config = OpenMWConfiguration::default();
        let root_config = match path {
            Some(path) => {
                if path.is_file() {
                    path
                } else if path.is_dir() {
                    path.join("openmw.cfg")
                } else {
                    return Err(format!(
                        "[CRITICAL FAILURE]: Provided openmw.cfg was neither a directory nor a file. PLEASE report this issue in the OpenMW Discord!"
                    ));
                }
            }
            None => crate::default_config_path().join("openmw.cfg"),
        };

        match config.load(&root_config) {
            Err(error) => Err(error),
            Ok(_) => {
                config.root_config = root_config;

                if let Some(dir) = &config.data_local {
                    config.data_directories.push(dir.clone());

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
                    config.data_directories.insert(0, dir.join("vfs-mw"));
                    config.data_directories.insert(0, dir.join("vfs"))
                }

                util::debug_log(format!("{:#?}", config.game_settings));

                Ok(config)
            }
        }
    }

    /// Path to the highest-level configuration *directory*
    pub fn user_config_path(&self) -> PathBuf {
        util::user_config_path(&self.sub_configs, &self.root_config)
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
    pub fn data_directories(&self) -> &Vec<PathBuf> {
        &self.data_directories
    }

    /// This early iteration of the crate provides no input validation for setter functions.
    pub fn set_data_directories(&mut self, dirs: Vec<PathBuf>) {
        self.data_directories = dirs
    }

    /// Fallback entries are k/v pairs baked into the value side of k/v pairs in `fallback=` entries of openmw.cfg
    /// They are used to express settings which are defined in Morrowind.ini for things such as:
    /// weather, lighting behaviors, UI Colors, and levelup messages
    pub fn fallback_entries(&self) -> &HashMap<String, String> {
        &self.fallback_entries
    }

    /// This early iteration of the crate provides no input validation for setter functions.
    pub fn set_fallback_entries(&mut self, entries: HashMap<String, String>) {
        self.fallback_entries = entries
    }

    /// List of filenames of Bethesda Archive files to use in the composed configuration
    pub fn fallback_archives(&self) -> &Vec<String> {
        &self.fallback_archives
    }

    /// This early iteration of the crate provides no input validation for setter functions.
    pub fn set_fallback_archives(&mut self, archives: Vec<String>) {
        self.fallback_archives = archives
    }

    /// Path to the openmw.cfg file which is the root of the configuration chain
    pub fn root_config(&self) -> &PathBuf {
        &self.root_config
    }

    /// Data-local is a special directory which, if defined, always has the highest priority over all data directories,
    /// thus overwriting their files
    pub fn data_local(&self) -> &Option<PathBuf> {
        &self.data_local
    }

    /// Override the data-local dir
    /// This early iteration of the crate provides no input validation for setter functions.
    pub fn set_data_local(&mut self, dir: PathBuf) {
        self.data_local = Some(dir)
    }

    /// Resources is a special directory which functions the opposite of data-local: It always has the *lowest* priority, to load necessary files
    /// but potentially be overridden by mods or other games
    pub fn resources_dir(&self) -> &Option<PathBuf> {
        &self.resources
    }

    /// Overrides the resources directory
    /// This early iteration of the crate provides no input validation for setter functions.
    pub fn set_resources_dir(&mut self, dir: PathBuf) {
        self.resources = Some(dir)
    }

    /// Userdata is a special directory in which saves, screenshots, and other user-specific miscellany go
    /// which are *not* related to configuration, such as navmesh.db
    pub fn userdata_dir(&self) -> &Option<PathBuf> {
        &self.userdata
    }

    /// Overrides the userdata directory
    /// This early iteration of the crate provides no input validation for setter functions.
    pub fn set_userdata_dir(&mut self, dir: PathBuf) {
        self.userdata = Some(dir)
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
    pub fn sub_configs(&self) -> &Vec<PathBuf> {
        &self.sub_configs
    }

    /// Transposes an input directory or file path to an openmw.cfg path
    /// Maybe could do with some additional validation
    fn input_config_path(&self, config_dir: &Path) -> Result<PathBuf, String> {
        if config_dir.is_file() {
            Ok(config_dir.to_path_buf())
        } else if config_dir.is_dir() {
            Ok(config_dir.join("openmw.cfg"))
        } else {
            Err(format!(
                "Unable to determine whether {config_dir:?} was a file or directory, refusing to read config!"
            ))
        }
    }

    fn load(&mut self, config_dir: &Path) -> Result<(), String> {
        let config_path = self.input_config_path(config_dir)?;

        util::debug_log(format!("BEGIN CONFIG PARSING: {config_path:?}"));

        if !config_path.exists() {
            return Err(format!(
                "openmw.cfg does not exist at the path {:?}",
                config_path
            ));
        }

        let mut sub_configs = Vec::new();
        let lines =
            read_to_string(&config_path).map_err(|e| format!("Failed to read config: {}", e))?;

        let mut comment_queue: Vec<String> = Vec::new();

        for line in lines.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                comment_queue.push(line.to_string());
                continue;
            }

            let tokens: Vec<&str> = trimmed.splitn(2, '=').collect();
            if tokens.len() < 2 {
                comment_queue.push(line.to_string());
                continue;
            }

            let key = tokens[0].trim();
            let value = tokens[1].trim().to_string();

            // HACK: Clone the value to we can move it into other functions. I do not care right now.
            let value_key = value.clone();

            match key {
                "data" => {
                    let dir = strings::parse_data_directory(&config_dir, value);
                    self.data_directories.push(dir);
                }
                "content" => {
                    if self.content_files.contains(&value) {
                        return Err(format!(
                            "{value} was listed as a content file by two configurations! The second one was: {config_dir:?}",
                        ));
                    }
                    self.content_files.push(value);
                }
                "fallback-archive" => {
                    self.fallback_archives.push(value);
                }
                "fallback" => {
                    let tokens: Vec<&str> = value.splitn(2, ',').collect();

                    if tokens.len() < 2 {
                        return Err(format!(
                            "ERROR: Invalid fallback= value {value} found in the openmw.cfg in: {config_dir:?}"
                        ));
                    }

                    self.game_settings.insert(
                        tokens[0].to_string(),
                        gamesetting::GameSettingType::from((
                            tokens[1].to_string(),
                            config_dir.to_owned(),
                        )),
                    );
                }
                "data-local" => {
                    self.data_local = Some(strings::parse_data_directory(&config_dir, value));
                }
                "config" => {
                    let config_dir = if config_dir.is_dir() {
                        config_dir
                    } else {
                        config_dir.parent().expect("")
                    }
                    .to_path_buf();

                    let config_path = strings::parse_data_directory(&config_dir, value.to_owned());

                    sub_configs.push(config_path);
                }
                "resources" => {
                    self.resources = Some(strings::parse_data_directory(&config_dir, value));
                }
                "userdata" => {
                    self.userdata = Some(strings::parse_data_directory(&config_dir, value));
                }
                "replace" => match value.to_lowercase().as_str() {
                    "content" => self.content_files = Vec::new(),
                    "data" => self.data_directories = Vec::new(),
                    "fallback" => self.game_settings = HashMap::new(),
                    "fallback-archives" => self.fallback_archives = Vec::new(),
                    "data-local" => self.data_local = None,
                    "resources" => self.resources = None,
                    "userdata" => self.userdata = None,
                    "config" => {
                        self.content_files = Vec::new();
                        self.data_directories = Vec::new();
                        self.game_settings = HashMap::new();
                        self.fallback_archives = Vec::new();
                        self.data_local = None;
                        self.resources = None;
                        self.userdata = None;
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

            if !comment_queue.is_empty() {
                self.comments.insert(value_key, comment_queue.clone());
                comment_queue.clear();
            }
        }

        // Store the trailing comments for a given openmw.cfg
        // By placing a copy of its absolute path in the comment map
        // Then, during reserialization, rewrite the trailing comments with their own set of
        // comments according to what configuration file they belong to
        self.trailing_comments.insert(
            config_path.to_string_lossy().to_ascii_lowercase(),
            comment_queue,
        );

        // A configuration entry doesn't necessarily *need* to have an openmw.cfg as the system is more complex than that
        // However, it should still be tracked for other purposes regardless
        for config in sub_configs {
            self.sub_configs.push(config.clone());

            if config.join("openmw.cfg").is_file() {
                // dbg!("READING NEXT CONFIG: ", config.join("openmw.cfg"), &self);
                if let Err(e) = self.load(Path::new(&config)) {
                    return Err(format!(
                        "WARNING: Sub-configuration {:?} failed to load with error: {}",
                        config_dir, e
                    ));
                }
            }
        }

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
            strings::write_comments(
                comments.remove(&userdata.display().to_string()),
                &mut config_string,
            );
            strings::userdata(&mut config_string, &self.userdata)?;
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

        for dir in &self.data_directories {
            strings::write_comments(
                comments.remove(&dir.display().to_string()),
                &mut config_string,
            );
            strings::data_directory(&mut config_string, &dir)?;
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

            for comment in comments {
                config_string.push_str(comment.as_str());
                config_string.push('\n');
            }
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
            writeln!(f, "userdata={}", userdata.display())?;
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
        for dir in &self.data_directories {
            writeln!(f, "data={}", dir.display())?;
        }

        // Content files
        for content in &self.content_files {
            writeln!(f, "content={}", content)?;
        }

        // Fallback entries
        for (key, value) in &self.fallback_entries {
            writeln!(f, "fallback={},{}", key, value)?;
        }

        Ok(())
    }
}
