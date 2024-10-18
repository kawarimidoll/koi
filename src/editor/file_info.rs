use std::ffi::OsStr;
use std::path::{Path, PathBuf};

#[derive(Eq, PartialEq, Debug, Copy, Clone)]
pub enum FileType {
    Rust,
    Text,
    Gitignore,
    Gitcommit,
    Vim,
}

impl FileType {
    pub fn from_ext(ext: &str) -> Option<Self> {
        match ext {
            "txt" => Some(FileType::Text),
            "rs" => Some(FileType::Rust),
            _ => None,
        }
    }
    pub fn from_file_name(file_name: &str) -> Option<Self> {
        match file_name {
            ".gitignore" => Some(FileType::Gitignore),
            "COMMIT_EDITMSG" => Some(FileType::Gitcommit),
            ".vimrc" => Some(FileType::Vim),
            // TODO: add other file types
            _ => None,
        }
    }
}

#[derive(Default)]
pub struct FileInfo {
    path: Option<PathBuf>,
    file_type: Option<FileType>,
}

impl FileInfo {
    pub fn from(path: &str) -> Self {
        let path = PathBuf::from(path);
        let file_type = path
            .extension()
            .and_then(|ext| ext.to_str())
            .and_then(FileType::from_ext)
            .or_else(|| {
                path.file_name()
                    .and_then(|file_name| FileType::from_file_name(file_name.to_str()?))
            });

        Self {
            path: Some(path),
            file_type,
        }
    }
    pub fn has_path(&self) -> bool {
        self.path.is_some()
    }
    pub fn get_path(&self) -> Option<&Path> {
        self.path.as_deref()
    }
    pub fn get_file_name(&self) -> Option<String> {
        self.path
            .as_deref()
            .and_then(Path::file_name)
            .and_then(OsStr::to_str)
            .map(std::string::ToString::to_string)
    }
    pub fn get_file_type(&self) -> Option<FileType> {
        self.file_type
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from() {
        let fi = FileInfo::from("test.txt");
        assert_eq!(fi.get_path(), Some(PathBuf::from("test.txt").as_ref()));
        assert_eq!(fi.get_file_type(), Some("txt".to_string()));
        let fi = FileInfo::from("/User/home/test.rs");
        assert_eq!(
            fi.get_path(),
            Some(PathBuf::from("/User/home/test.rs").as_ref())
        );
        assert_eq!(fi.get_file_name(), Some(String::from("test.rs")));
        assert_eq!(fi.get_file_type(), Some("rs".to_string()));
        let fi = FileInfo::from(".gitignore");
        assert_eq!(fi.get_path(), Some(PathBuf::from(".gitignore").as_ref()));
        assert_eq!(fi.get_file_type(), Some("gitignore".to_string()));
        let fi = FileInfo::from(".vimrc");
        assert_eq!(fi.get_path(), Some(PathBuf::from(".vimrc").as_ref()));
        assert_eq!(fi.get_file_type(), Some("vim".to_string()));
    }
}
