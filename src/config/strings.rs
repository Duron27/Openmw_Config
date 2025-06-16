/// This whole file should just be part of DirectorySetting now and the parse_data_directory function exposed in the lib.rs
/// But right now, meh
use std::path::PathBuf;

fn strip_special_components<P: AsRef<std::path::Path>>(input: P) -> PathBuf {
    let mut result = PathBuf::new();
    for component in input.as_ref().components() {
        use std::path::Component::*;
        match component {
            CurDir => {} // skip '.'
            ParentDir => {
                result.pop(); // remove last segment
            }
            Normal(part) => result.push(part),
            RootDir => result.push(component),
            Prefix(prefix) => result.push(prefix.as_os_str()), // for Windows
        }
    }
    result
}

/// Parses a data directory string according to OpenMW rules.
/// https://openmw.readthedocs.io/en/latest/reference/modding/paths.html#openmw-cfg-syntax
pub fn parse_data_directory<P: AsRef<std::path::Path>>(
    config_dir: &P,
    mut data_dir: String,
) -> PathBuf {
    // Quote handling
    if data_dir.starts_with('"') {
        let mut result = String::new();
        let mut i = 1;
        let chars: Vec<char> = data_dir.chars().collect();
        while i < chars.len() {
            if chars[i] == '&' {
                i += 1; // skip the next char (escape)
            } else if chars[i] == '"' {
                break;
            }
            if i < chars.len() {
                result.push(chars[i]);
            }
            i += 1;
        }
        data_dir = result;
    }

    // Token replacement
    if data_dir.starts_with("?userdata?") {
        let suffix = data_dir["?userdata?".len()..].trim_start_matches(&['/', '\\'][..]);

        data_dir = crate::default_userdata_path()
            .join(suffix)
            .to_string_lossy()
            .to_string();
    } else if data_dir.starts_with("?userconfig?") {
        let suffix = data_dir["?userdata?".len()..].trim_start_matches(&['/', '\\'][..]);

        data_dir = crate::default_config_path()
            .join(suffix)
            .to_string_lossy()
            .to_string();
    }

    let data_dir = data_dir.replace(['/', '\\'], &std::path::MAIN_SEPARATOR.to_string());

    let mut path = PathBuf::from(&data_dir);
    if !path.is_absolute() {
        path = config_dir.as_ref().join(path);
    }

    strip_special_components(path)
}
