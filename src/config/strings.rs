use std::{fmt::Write, path::PathBuf};

// Private helper for error handling when writing to String
fn write_line_io(f: &mut String, line: String) -> Result<(), String> {
    writeln!(f, "{line}").map_err(|_| format!("Failed writing config line: {line:?}"))
}

pub fn resources(f: &mut String, resources: &Option<PathBuf>) -> Result<(), String> {
    if let Some(resources) = resources {
        return write_line_io(f, format!("resources={}", resources.display()));
    } else {
        Ok(())
    }
}

pub fn userdata(f: &mut String, userdata: &String) -> Result<(), String> {
    write_line_io(f, format!("userdata={}", userdata))
}

pub fn data_local(f: &mut String, data_local: &Option<PathBuf>) -> Result<(), String> {
    if let Some(data_local) = data_local {
        write_line_io(f, format!("data-local={}", data_local.display()))
    } else {
        Ok(())
    }
}

pub fn fallback_archive(f: &mut String, fallback_archive: &String) -> Result<(), String> {
    write_line_io(f, format!("fallback-archive={}", fallback_archive))
}

pub fn data_directory(f: &mut String, data_directory: &PathBuf) -> Result<(), String> {
    write_line_io(f, format!("data={}", data_directory.display()))
}

pub fn content_file(f: &mut String, content_file: &String) -> Result<(), String> {
    write_line_io(f, format!("content={}", content_file))
}

pub fn fallback_entry(f: &mut String, key: &String, value: &String) -> Result<(), String> {
    write_line_io(f, format!("fallback={},{}", key, value))
}

pub fn write_comments(comments: Option<Vec<String>>, config_string: &mut String) {
    if let Some(comments) = comments {
        for comment in comments {
            config_string.push_str(&comment);
            config_string.push('\n');
        }
    }
}

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
