use std::path::{Path, PathBuf};

#[derive(Default)]
pub struct FileInfo {
    path: Option<PathBuf>,
    #[allow(dead_code)]
    file_type: Option<String>,
}

impl FileInfo {
    pub fn from(path: &str) -> Self {
        let path = PathBuf::from(path);
        let file_type = Self::file_type_from_path(&path);
        Self {
            path: Some(path),
            file_type,
        }
    }
    fn file_type_from_path(path: &Path) -> Option<String> {
        if let Some(ext) = path.extension() {
            ext.to_str().map(std::string::ToString::to_string)
        } else {
            // TODO: return specific type for files without extension
            None
        }
    }
    pub fn has_path(&self) -> bool {
        self.path.is_some()
    }
    pub fn get_path(&self) -> Option<&Path> {
        self.path.as_deref()
    }
    #[allow(dead_code)]
    pub fn get_file_type(&self) -> Option<String> {
        self.file_type.clone()
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
        assert_eq!(fi.get_file_type(), Some("rs".to_string()));
        // let fi = FileInfo::from(".gitignore");
        // assert_eq!(fi.get_path(), Some(PathBuf::from(".gitignore").as_ref()));
        // assert_eq!(fi.get_file_type(), Some("gitignore".to_string()));
        // let fi = FileInfo::from(".vimrc");
        // assert_eq!(fi.get_path(), Some(PathBuf::from(".vimrc").as_ref()));
        // assert_eq!(fi.get_file_type(), Some("vim".to_string()));
    }
}
