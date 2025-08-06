use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::io::SeekFrom;
use std::path;
use std::path::{PathBuf};
use crate::{FileContent, FileInfo, FileSystem, FileSystemError};
use crate::enc_utils::{EncKey, EncUtils};

const HEADER_SIZE: usize = 1 + 4 + 8 + 8; // Version, number of files, total size
const FILE_ENTRY_SIZE: usize = MAX_FILE_NAME_SIZE + MAX_PATH_SIZE + 8 + 8; // File name, path, size, offset
const MAX_FILE_NAME_SIZE: usize = 16; // Maximum size for file name in bytes
const MAX_PATH_SIZE: usize = 255; // Maximum size for file path in bytes

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FileEntry {
    pub name: [u8; MAX_FILE_NAME_SIZE],
    pub path: [u8; MAX_PATH_SIZE],
    pub size: u64,
    pub offset: u64,
}

impl FileEntry {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        if bytes.len() < FILE_ENTRY_SIZE {
            panic!("File entry data is too short");
        }
        let name = bytes[0..MAX_FILE_NAME_SIZE].try_into().unwrap_or([0; MAX_FILE_NAME_SIZE]);
        let path = bytes[MAX_FILE_NAME_SIZE..MAX_FILE_NAME_SIZE + MAX_PATH_SIZE].try_into().unwrap_or([0; MAX_PATH_SIZE]);
        let size = u64::from_le_bytes(bytes[MAX_FILE_NAME_SIZE + MAX_PATH_SIZE..MAX_FILE_NAME_SIZE + MAX_PATH_SIZE + 8].try_into().unwrap());
        let offset = u64::from_le_bytes(bytes[MAX_FILE_NAME_SIZE + MAX_PATH_SIZE + 8..].try_into().unwrap());
        FileEntry { name, path, size, offset }
    }

    pub fn name(&self) -> String {
        String::from_utf8_lossy(&self.name).trim_end_matches('\0').to_string()
    }

    pub fn path(&self) -> String {
        String::from_utf8_lossy(&self.path).trim_end_matches('\0').to_string()
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(FILE_ENTRY_SIZE);
        bytes.extend_from_slice(&self.name);
        bytes.extend_from_slice(&self.path);
        bytes.extend_from_slice(&self.size.to_le_bytes());
        bytes.extend_from_slice(&self.offset.to_le_bytes());
        bytes
    }

    pub fn new(name: &str, path: &str, size: u64, offset: u64) -> Self {
        let mut name_bytes = [0u8; MAX_FILE_NAME_SIZE];
        let mut path_bytes = [0u8; MAX_PATH_SIZE];
        name_bytes[..name.len()].copy_from_slice(name.as_bytes());
        path_bytes[..path.len()].copy_from_slice(path.as_bytes());
        FileEntry {
            name: name_bytes,
            path: path_bytes,
            size,
            offset,
        }
    }

    pub fn set_size(&mut self, size: u64) {
        self.size = size;
    }

    pub fn set_offset(&mut self, offset: u64) {
        self.offset = offset;
    }

    pub fn strip_prefix(&mut self, path: &PathBuf) -> Result<(), FileSystemError> {
        let full_path = PathBuf::from(self.path());
        let stripped = full_path.strip_prefix(path).unwrap_or(&full_path);
        let stripped_path = stripped.to_str().ok_or(FileSystemError::from("Invalid UTF-8 in file path"))?;
        let mut new_path_bytes = [0u8; MAX_PATH_SIZE];
        new_path_bytes[..stripped_path.len()].copy_from_slice(stripped_path.as_bytes());
        self.path.copy_from_slice(&new_path_bytes);
        Ok(())
    }
}

pub struct Header {
    pub version: u8,
    pub number_of_files: u32,
    pub size: u64,
    pub data_offset: u64,
}

impl Header {
    fn from_bytes(bytes: &[u8]) -> Self {
        if bytes.len() < HEADER_SIZE {
            panic!("Header data is too short");
        }
        let version = bytes[0];
        let number_of_files = u32::from_le_bytes([bytes[1], bytes[2], bytes[3], bytes[4]]);
        let size = u64::from_le_bytes([bytes[5], bytes[6], bytes[7], bytes[8], bytes[9], bytes[10], bytes[11], bytes[12]]);
        let data_offset = u64::from_le_bytes([bytes[13], bytes[14], bytes[15], bytes[16], bytes[17], bytes[18], bytes[19], bytes[20]]);
        Header {
            version,
            number_of_files,
            size,
            data_offset,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = vec![self.version];
        bytes.extend_from_slice(&self.number_of_files.to_le_bytes());
        bytes.extend_from_slice(&self.size.to_le_bytes());
        bytes.extend_from_slice(&self.data_offset.to_le_bytes());
        bytes
    }
}

pub struct ArchiveFileSystem {
    file_path: PathBuf,
    #[allow(dead_code)]
    header: Header,
    entries: HashMap<String, FileEntry>,
    enc_utils: EncUtils,
}


impl ArchiveFileSystem {

    pub fn open(file_path: PathBuf, key: EncKey) -> Result<Self, FileSystemError> {
        let mut file = File::open(&file_path).map_err(|e| FileSystemError::from(e.to_string()))?;
        let mut header_data = [0u8; HEADER_SIZE];
        file.read_exact(&mut header_data).map_err(|e| FileSystemError::from(e.to_string()))?;
        let header = Header::from_bytes(&header_data);
        if header.version != 1 {
            return Err(FileSystemError::from("Unsupported archive version"));
        }
        if header.number_of_files == 0 {
            return Err(FileSystemError::from("Archive contains no files"));
        }
        if header.size < HEADER_SIZE as u64 + header.number_of_files as u64 * FILE_ENTRY_SIZE as u64 {
            return Err(FileSystemError::from("Invalid archive size"));
        }
        if header.data_offset < HEADER_SIZE as u64 + header.number_of_files as u64 * FILE_ENTRY_SIZE as u64 {
            return Err(FileSystemError::from("Invalid data offset in archive"));
        }
        let mut entries = HashMap::new();
        for _ in 0..header.number_of_files {
            let mut entry_data = vec![0u8; FILE_ENTRY_SIZE];
            file.read_exact(&mut entry_data).map_err(|e| FileSystemError::from(e.to_string()))?;
            let file_entry = FileEntry::from_bytes(&entry_data);
            entries.insert(file_entry.path(), file_entry);
        }
        let enc_utils = EncUtils::new(key)?;

        Ok(ArchiveFileSystem {
            file_path,
            header,
            entries,
            enc_utils,
        })
    }
}


pub struct ArchiveCreator {
    directory_path: PathBuf,
    file_path: PathBuf,
    enc_utils: EncUtils,
    file_entries: Vec<FileEntry>,
}

impl ArchiveCreator {
    pub fn new(directory_path: &str, file_path: &str, key: EncKey, overwrite: bool) -> Result<Self, FileSystemError> {
        let directory_path = PathBuf::from(directory_path);
        let file_path = PathBuf::from(file_path);
        if !directory_path.is_dir() {
            return Err(FileSystemError::from("Provided source directory path is not a directory"));
        }
        if file_path.exists() && !overwrite {
            return Err(FileSystemError::from("Archive file already exists and overwrite is not allowed"));
        }
        let enc_utils = EncUtils::new(key)?;
        Ok(ArchiveCreator {
            directory_path,
            file_path,
            enc_utils,
            file_entries: Vec::new(),
        })
    }

    fn scan_directory(&mut self, path: &PathBuf) -> Result<(), FileSystemError> {
        if !path.is_dir() {
            return Err(FileSystemError::from("Provided path is not a directory"));
        }
        for entry in std::fs::read_dir(path).map_err(|e| FileSystemError::from(e.to_string()))? {
            let entry = entry.map_err(|e| FileSystemError::from(e.to_string()))?;
            let entry_path = entry.path();
            if entry_path.is_dir() {
                self.scan_directory(&entry_path)?;
            } else if entry_path.is_file() {
                let file_name = entry.file_name().to_string_lossy().into_owned();
                let file_size = entry.metadata().map_err(|e| FileSystemError::from(e.to_string()))?.len();
                let entry = FileEntry::new(
                    &file_name,
                    entry_path.to_str().ok_or(FileSystemError::from("Invalid file path"))?,
                    file_size, // An updated file size will be set later
                    0, // Offset will be set later
                );
                self.file_entries.push(entry);
            } else {
                self.scan_directory(&entry_path)?;
            }
        }
        Ok(())
    }

    pub fn create(&mut self) -> Result<(), FileSystemError> {
        let directory_path = self.directory_path.clone();
        self.scan_directory(&directory_path)?;
        if self.file_entries.is_empty() {
            return Err(FileSystemError::from("No files found to archive"));
        }
        let mut file = File::create(&self.file_path).map_err(|e| FileSystemError::from(e.to_string()))?;
        let mut header = Header {
            version: 1,
            number_of_files: self.file_entries.len() as u32,
            size: 0, // Will be updated later
            data_offset: HEADER_SIZE as u64 + self.file_entries.len() as u64 * FILE_ENTRY_SIZE as u64,
        };
        file.write_all(&header.to_bytes()).map_err(|e| FileSystemError::from(e.to_string()))?;
        let mut new_entries: Vec<FileEntry> = Vec::new();
        for entry in &self.file_entries {
            let full_path = path::PathBuf::from(entry.path());
            if !full_path.exists() || !full_path.is_file() {
                return Err(FileSystemError::from(format!("File does not exist: {}", full_path.display())));
            }
            let content = std::fs::read(full_path).map_err(|e| FileSystemError::from(e.to_string()))?;
            let encrypted_content = self.enc_utils.encrypt(content).map_err(|e| FileSystemError::from(e.to_string()))?;
            let offset = file.stream_position().map_err(|e| FileSystemError::from(e.to_string()))?;
            file.write_all(&encrypted_content).map_err(|e| FileSystemError::from(e.to_string()))?;
            let size = encrypted_content.len() as u64;
            let mut new_entry = entry.clone();
            new_entry.set_size(size);
            new_entry.set_offset(offset);
            new_entry.strip_prefix(&self.directory_path)?;
            new_entries.push(new_entry);
        }
        // Write file entries
        file.seek(SeekFrom::Start(HEADER_SIZE as u64)).map_err(|e| FileSystemError::from(e.to_string()))?;
        for entry in new_entries {
            file.write_all(&entry.to_bytes()).map_err(|e| FileSystemError::from(e.to_string()))?;
        }
        header.size = file.stream_position().map_err(|e| FileSystemError::from(e.to_string()))?;
        file.seek(SeekFrom::Start(0)).map_err(|e| FileSystemError::from(e.to_string()))?;
        file.write_all(&header.to_bytes()).map_err(|e| FileSystemError::from(e.to_string()))?;
        Ok(())
    }
}


impl From<&FileEntry> for FileInfo {
    fn from(entry: &FileEntry) -> Self {
        FileInfo {
            name: entry.name(),
            path: entry.path(),
            size: entry.size,
            is_directory: false, // Archive entries are not directories
        }
    }
}

impl FileSystem for ArchiveFileSystem {
    fn read_file(&self, path: &str) -> Result<FileContent, FileSystemError> {
        let entry = self.entries.get(path).ok_or(FileSystemError::from("File not found in archive"))?;
        let mut file = File::open(&self.file_path).map_err(|e| FileSystemError::from(e.to_string()))?;
        file.seek(SeekFrom::Start(entry.offset)).map_err(|e| FileSystemError::from(e.to_string()))?;
        let mut content = vec![0u8; entry.size as usize];
        file.read_exact(&mut content).map_err(|e| FileSystemError::from(e.to_string()))?;
        self.enc_utils.decrypt(content).map_err(|e| FileSystemError::from(e.to_string()))
    }

    fn write_file(&self, _path: &str, _content: FileContent) -> Result<(), FileSystemError> {
        Err(FileSystemError::from("Archive is read-only, cannot write files"))
    }

    fn delete_file(&self, _path: &str) -> Result<(), FileSystemError> {
        Err(FileSystemError::from("Archive is read-only, cannot delete files"))
    }

    fn list_files(&self, directory: &str) -> Result<Vec<FileInfo>, FileSystemError> {
        let mut file_infos: Vec<FileInfo> = Vec::new();
        let keys = self.entries.keys().filter(|k| k.starts_with(directory)).cloned().collect::<Vec<_>>();
        for key in keys {
            if let Some(entry) = self.entries.get(&key) {
                let file_info = FileInfo::from(entry);
                file_infos.push(file_info);
            }
        }
        let directories: Vec<String> = self.entries.keys()
            .filter(|k| k.starts_with(directory) && k != &directory)
            .map(|k| k.split('/').next().unwrap_or("").to_string())
            .collect();
        for dir in directories {
            if !file_infos.iter().any(|f| f.path == dir) {
                file_infos.push(FileInfo {
                    path: dir.clone(),
                    name: dir.split("/").last().unwrap_or("").to_string(),
                    size: 0,
                    is_directory: true,
                });
            }
        }
        Ok(file_infos)
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    use crate::enc_utils::EncUtils;

    #[test]
    fn test_archive() {
        let key = EncUtils::generate_random_key();
        let mut creator = ArchiveCreator::new("test_directory", "test_archive.arc", key.clone(), true).expect("Failed to create ArchiveCreator");
        creator.create().expect("Failed to create archive");
        let archive_fs = ArchiveFileSystem::open(PathBuf::from("test_archive.arc"), key).expect("Failed to open archive");
        assert!(!archive_fs.entries.is_empty(), "Archive should contain files");
        let files = archive_fs.list_files("").expect("Failed to list files in archive");
        assert!(!files.is_empty(), "Archive should list files");
        for file in files {
            println!("{}", file.path);
        }
    }
}