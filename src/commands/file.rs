use std::fs::File as FsFile;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
pub enum FileError {
    NotFound(String),
    PermissionDenied(String),
    InvalidPath(String),
    ReadError(String),
    Other(String),
}

impl std::fmt::Display for FileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FileError::NotFound(path) => write!(f, "File not found: {}", path),
            FileError::PermissionDenied(path) => write!(f, "Permission denied: {}", path),
            FileError::InvalidPath(path) => write!(f, "Invalid path: {}", path),
            FileError::ReadError(msg) => write!(f, "Error reading file: {}", msg),
            FileError::Other(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl std::error::Error for FileError {}

impl From<std::io::Error> for FileError {
    fn from(error: std::io::Error) -> Self {
        match error.kind() {
            std::io::ErrorKind::NotFound => FileError::NotFound(error.to_string()),
            std::io::ErrorKind::PermissionDenied => FileError::PermissionDenied(error.to_string()),
            _ => FileError::Other(error.to_string()),
        }
    }
}

pub struct File {
    file_path: String
}

impl File {
    //constructor
    pub fn new(file_path: String) -> Self {
        Self { file_path }
    }

    //validate path exists
    fn validate_path(&self) -> Result<(), FileError> {
        let path = Path::new(&self.file_path);

        if !path.exists() {
            return Err(FileError::NotFound(self.file_path.clone()));
        }

        if !path.is_file() {
            return Err(FileError::InvalidPath(format!("{} is not a file", self.file_path)));
        }

        Ok(())
    }

    ///public method info
    pub fn info(&self) -> Result<String, FileError> {
        self.validate_path()?;

        let file = FsFile::open(&self.file_path)
            .map_err(|e| FileError::ReadError(format!("Failed to open {}: {}", self.file_path, e)))?;

        let metadata = file.metadata()
            .map_err(|e| FileError::ReadError(format!("Failed to read metadata: {}", e)))?;

        let info = format!(
            "Path: {}\nSize: {} bytes\nReadonly: {}",
            self.file_path,
            metadata.len(),
            metadata.permissions().readonly()
        );

        Ok(info)
    }

    ///public method lines
    pub fn lines(&self) -> Result<String, FileError> {
        self.validate_path()?;

        let file = FsFile::open(&self.file_path)
            .map_err(|e| FileError::ReadError(format!("Failed to open {}: {}", self.file_path, e)))?;

        let lines = BufReader::new(file).lines().count();

        let info_line = format!("File {} has {} lines", self.file_path, lines);
        Ok(info_line)
    }

    ///public method size
    pub fn size(&self) -> Result<String, FileError> {
        self.validate_path()?;

        let file = FsFile::open(&self.file_path)
            .map_err(|e| FileError::ReadError(format!("Failed to open {}: {}", self.file_path, e)))?;

        let metadata = file.metadata()
            .map_err(|e| FileError::ReadError(format!("Failed to read metadata: {}", e)))?;

        let info_size = format!("File {} has {} bytes", self.file_path, metadata.len());
        Ok(info_size)
    }

    ///public method head
    pub fn head(&self, lines: usize) -> Result<String, FileError> {
        self.validate_path()?;

        let file = FsFile::open(&self.file_path)
            .map_err(|e| FileError::ReadError(format!("Failed to open {}: {}", self.file_path, e)))?;

        let reader = BufReader::new(file);
        let mut result = String::new();

        for line in reader.lines().take(lines) {
            let line = line.map_err(|e| FileError::ReadError(format!("Failed to read line: {}", e)))?;
            result.push_str(&line);
            result.push('\n');
        }

        Ok(result)
    }
}