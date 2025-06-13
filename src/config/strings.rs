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

pub fn userdata(f: &mut String, userdata: &Option<PathBuf>) -> Result<(), String> {
    if let Some(userdata) = userdata {
        write_line_io(f, format!("userdata={}", userdata.display()))
    } else {
        Ok(())
    }
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
