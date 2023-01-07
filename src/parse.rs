use std::{fs, path::PathBuf};

pub struct FileData {
    pub name: String,
    pub contents: String,
}

pub fn parse_file(path: PathBuf) -> FileData {
    FileData {
        name: path.file_name().unwrap().to_str().unwrap().into(),
        contents: fs::read_to_string(path).unwrap(),
    }
}
