use std::{fs, path::PathBuf};

use anyhow::{Result, Context};

pub struct FileData {
    pub name: String,
    pub contents: String,
}

pub fn parse_file(path: PathBuf) -> Result<FileData> {
    let name = path
        .file_name().context("No file name found")?
        .to_str().context("File name was not valid utf-8")?
        .into();

    let contents = fs::read_to_string(path)?;

    Ok(FileData { name, contents })
}
