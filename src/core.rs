use std::error::Error;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FileSystemError {
    pub message: String,
}

impl std::fmt::Display for FileSystemError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "FileSystemError: {}", self.message)
    }
}

impl Error for FileSystemError {}

impl From<std::io::Error> for FileSystemError {
    fn from(err: std::io::Error) -> Self {
        FileSystemError {
            message: err.to_string(),
        }
    }
}

impl From<String> for FileSystemError {
    fn from(message: String) -> Self {
        FileSystemError { message }
    }
}

impl From<&str> for FileSystemError {
    fn from(message: &str) -> Self {
        FileSystemError {
            message: message.to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FileInfo {
    pub name: String,
    pub path: String,
    pub is_directory: bool,
    pub size: u64,
}

impl From<std::fs::DirEntry> for FileInfo {
    fn from(entry: std::fs::DirEntry) -> Self {
        let metadata = entry.metadata().expect("metadata");
        FileInfo {
            name: entry.file_name().to_string_lossy().into_owned(),
            path: entry.path().to_string_lossy().into_owned(),
            is_directory: metadata.is_dir(),
            size: metadata.len(),
        }
    }
}

pub type FileContent = Vec<u8>;


pub trait FileSystem {
    fn read_file(&self, path: &str) -> Result<FileContent, FileSystemError>;
    fn write_file(&self, path: &str, content: FileContent) -> Result<(), FileSystemError>;
    fn delete_file(&self, path: &str) -> Result<(), FileSystemError>;
    fn list_files(&self, directory: &str) -> Result<Vec<FileInfo>, FileSystemError>;

    fn read_file_as_string(&self, path: &str) -> Result<String, FileSystemError> {
        let content = self.read_file(path)?;
        String::from_utf8(content).map_err(|e| FileSystemError::from(e.to_string()))
    }
    fn write_file_from_string(&self, path: &str, content: &str) -> Result<(), FileSystemError> {
        let bytes = content.as_bytes().to_vec();
        self.write_file(path, bytes)
    }
}
