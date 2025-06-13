# openmw_config

**openmw_config** is a lightweight Rust crate that provides a simple, idiomatic API for reading, composing, and writing [OpenMW](https://openmw.org/) configuration files. It closely matches OpenMW's own configuration parser, supporting advanced features such as configuration chains, directory tokens, and value replacement semantics. For comprehensive coverage of OpenMW's configuration and filesystem, combine this crate with [vfstool_lib](https://crates.io/crates/vfstool_lib/0.1.0).

## Features

- **Accurate Parsing:** Mirrors OpenMW's config resolution, including support for `config=`, `replace=`, and directory tokens like `?userdata?`.
- **Multi-file Support:** Handles configuration chains, where multiple `openmw.cfg` files are merged according to OpenMW's rules.
- **Simple API:** Access content files, data directories, fallback entries, and more with ergonomic Rust methods.
- **Serialization:** Easily write out a configuration in valid `openmw.cfg` format.
- **No heavy dependencies:** Only depends on the Rust standard library and the [`dirs`](https://crates.io/crates/dirs) crate.

## Why use openmw-config?

Modern OpenMW installations often use multiple configuration files, with complex rules for merging and overriding settings. This crate makes it easy to:

- Inspect and modify the effective configuration as seen by OpenMW.
- Build tools for mod management, diagnostics, or launcher frontends.
- Programmatically generate or edit `openmw.cfg` files.

## Quick Start

Add to your `Cargo.toml`:

```toml
openmw-config = "0.1"
```

## Basic Usage

### Load the active OpenMW configuration

```rust
use openmw_cfg::OpenMWConfiguration;

fn main() -> Result<(), String> {
    // Load the default OpenMW configuration chain
    let config = OpenMWConfiguration::new(None)?;

    println!("Root config: {:?}", config.root_config());
    println!("User config: {:?}", config.user_config_path());

    // List all content files (plugins)
    for plugin in config.content_files() {
        println!("Plugin: {plugin}");
    }

    // List all data directories
    for dir in config.data_directories() {
        println!("Data dir: {}", dir.display());
    }

    Ok(())
}
```

### Load a specific config file

> **Note:**  
> The argument to `OpenMWConfiguration::new(Some(path))` must be the **directory containing** `openmw.cfg`, **not** the path to the file itself.  
> For example, if your config is at `/home/user/.config/openmw/openmw.cfg`, you should pass `/home/user/.config/openmw`.

```rust
use std::path::PathBuf;
use openmw_cfg::OpenMWConfiguration;

fn main() -> Result<(), String> {
    // Correct: pass the directory, not the file!
    let config_dir = PathBuf::from("/path/to/your/config_dir");
    let config = OpenMWConfiguration::new(Some(config_dir))?;

    // ... use config as above ...
    Ok(())
}
```

### Accessing and Modifying Configuration

```rust
use std::collections::HashMap;
use std::path::PathBuf;
use openmw_cfg::OpenMWConfiguration;

// Assume you have a mutable config instance:
let mut config = OpenMWConfiguration::new(None)?;

// Set content files
config.set_content_files(vec!["MyMod.esp".to_string(), "Another.esp".to_string()]);

// Set data directories
config.set_data_directories(vec![
    PathBuf::from("/path/to/DataFiles"),
    PathBuf::from("/path/to/OtherData"),
]);

// Set fallback entries
let mut fallbacks = HashMap::new();
fallbacks.insert("Some_Fallback".to_string(), "SomeValue".to_string());
config.set_fallback_entries(fallbacks);

// Set fallback archives
config.set_fallback_archives(vec!["Archive1.bsa".to_string(), "Archive2.bsa".to_string()]);

// Set data-local directory
config.set_data_local(PathBuf::from("/path/to/data-local"));

// Set resources directory
config.set_resources_dir(PathBuf::from("/path/to/resources"));

// Set userdata directory
config.set_userdata_dir(PathBuf::from("/path/to/userdata"));
```

### Manually Serializing and Saving to a File

```rust
use std::fs::File;
use std::io::Write;

// Serialize the config to a string in openmw.cfg format
let config_string = config.display();

// Write to a file of your choice
let mut file = File::create("/path/to/your/openmw.cfg")?;
file.write_all(config_string.as_bytes())?;
```

### Serialize to String (openmw.cfg format)

```rust
println!("{}", config); // Uses the Display trait to print in openmw.cfg format
```

## Advanced Features

- **Config Chains:**  
  OpenMW can load multiple config files in a chain. Use `config.sub_configs()` to inspect the chain and get granular access to the config.
- **Replace Semantics:**  
  Handles `replace=content`, `replace=data`, etc., as in OpenMW.
- **Token Expansion:**  
  Supports tokens like `?userdata?` and `?userconfig?` in directory paths.

## API Overview

- `OpenMWConfiguration::new(path: Option<PathBuf>) -> Result<Self, String>`  
  Load a configuration, optionally from a specific directory.
- `content_files() -> &Vec<String>`  
  List of plugin files.
- `data_directories() -> &Vec<PathBuf>`  
  List of data directories.
- `fallback_archives() -> &Vec<String>`
  List of Bethesda Archives defined by the current config
- `fallback_entries() -> &HashMap<String, String>`  
  Fallback key-value pairs.
- `save(dir: Option<PathBuf>) -> Result<(), String>`  
  Save the configuration to a directory.
- `Display` trait  
  Serialize the configuration to a valid `openmw.cfg` string.

## Reference

See [OpenMW documentation](https://openmw.readthedocs.io/en/latest/reference/modding/paths.html#configuration-sources) for details on configuration file semantics.

---

**openmw-config** is not affiliated with the OpenMW project, but aims to be a faithful and practical Rust implementation of its configuration logic.

PRs and issues welcome!
