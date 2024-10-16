use std::path::{Path, PathBuf};

#[derive(Default)]
pub struct FileInfo {
    path: Option<PathBuf>,
}

impl FileInfo {
    pub fn from(path: &str) -> Self {
        let path = PathBuf::from(path);
        Self { path: Some(path) }
    }
    pub fn has_path(&self) -> bool {
        self.path.is_some()
    }
    pub fn get_path(&self) -> Option<&Path> {
        self.path.as_deref()
    }
    #[allow(dead_code)]
    pub fn get_file_type(&self) -> Option<&str> {
        if let Some(path) = self.path.as_ref() {
            if let Some(ext) = path.extension() {
                ext.to_str()
            } else {
                // TODO: return specific type for files without extension
                None
            }
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from() {
        let fi = FileInfo::from("test.txt");
        assert_eq!(fi.get_path(), Some(PathBuf::from("test.txt").as_ref()));
        assert_eq!(fi.get_file_type(), Some("txt"));
        let fi = FileInfo::from("/User/home/test.rs");
        assert_eq!(
            fi.get_path(),
            Some(PathBuf::from("/User/home/test.rs").as_ref())
        );
        assert_eq!(fi.get_file_type(), Some("rs"));
        // let fi = FileInfo::from(".gitignore");
        // assert_eq!(fi.get_path(), Some(PathBuf::from(".gitignore").as_ref()));
        // assert_eq!(fi.get_file_type(), Some("gitignore"));
        // let fi = FileInfo::from(".vimrc");
        // assert_eq!(fi.get_path(), Some(PathBuf::from(".vimrc").as_ref()));
        // assert_eq!(fi.get_file_type(), Some("vim"));
    }
}
