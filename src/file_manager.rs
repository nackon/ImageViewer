use std::path::{Path, PathBuf};
use anyhow::Result;

pub struct FileManager {
    files: Vec<PathBuf>,
    current_index: usize,
}

impl FileManager {
    pub fn new() -> Self {
        Self {
            files: Vec::new(),
            current_index: 0,
        }
    }

    pub fn load_directory(&mut self, path: &Path) -> Result<()> {
        let dir = if path.is_dir() {
            path.to_path_buf()
        } else {
            path.parent()
                .ok_or_else(|| anyhow::anyhow!("Cannot get parent directory"))?
                .to_path_buf()
        };

        let mut files: Vec<PathBuf> = std::fs::read_dir(&dir)?
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .filter(|p| p.is_file() && Self::is_supported_image(p))
            .collect();

        self.sort_files(&mut files);

        if let Some(index) = files.iter().position(|p| p == path) {
            self.current_index = index;
        }

        self.files = files;
        Ok(())
    }

    pub fn get_current(&self) -> Option<&PathBuf> {
        self.files.get(self.current_index)
    }

    pub fn next(&mut self) -> Option<&PathBuf> {
        if self.files.is_empty() {
            return None;
        }
        self.current_index = (self.current_index + 1) % self.files.len();
        self.get_current()
    }

    pub fn previous(&mut self) -> Option<&PathBuf> {
        if self.files.is_empty() {
            return None;
        }
        if self.current_index == 0 {
            self.current_index = self.files.len() - 1;
        } else {
            self.current_index -= 1;
        }
        self.get_current()
    }

    pub fn jump_to(&mut self, index: usize) -> Option<&PathBuf> {
        if index < self.files.len() {
            self.current_index = index;
            self.get_current()
        } else {
            None
        }
    }

    pub fn first(&mut self) -> Option<&PathBuf> {
        self.jump_to(0)
    }

    pub fn last(&mut self) -> Option<&PathBuf> {
        if self.files.is_empty() {
            None
        } else {
            self.jump_to(self.files.len() - 1)
        }
    }

    pub fn current_index(&self) -> usize {
        self.current_index
    }

    pub fn total_count(&self) -> usize {
        self.files.len()
    }

    pub fn get_all_files(&self) -> &[PathBuf] {
        &self.files
    }

    pub fn peek_next(&self, offset: usize) -> Option<&PathBuf> {
        if self.files.is_empty() {
            return None;
        }
        let index = (self.current_index + offset) % self.files.len();
        self.files.get(index)
    }

    pub fn peek_previous(&self, offset: usize) -> Option<&PathBuf> {
        if self.files.is_empty() {
            return None;
        }
        let len = self.files.len();
        let index = (self.current_index + len - (offset % len)) % len;
        self.files.get(index)
    }

    fn is_supported_image(path: &Path) -> bool {
        if let Some(ext) = path.extension() {
            let ext = ext.to_string_lossy().to_lowercase();
            matches!(
                ext.as_str(),
                "jpg" | "jpeg" | "png" | "gif" | "bmp" | "webp" | "tiff" | "tif"
            )
        } else {
            false
        }
    }

    fn sort_files(&self, files: &mut [PathBuf]) {
        files.sort_by(|a, b| {
            a.file_name()
                .unwrap_or_default()
                .cmp(b.file_name().unwrap_or_default())
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use tempfile::TempDir;

    fn create_test_images(dir: &Path, names: &[&str]) -> Vec<PathBuf> {
        names
            .iter()
            .map(|name| {
                let path = dir.join(name);
                File::create(&path).unwrap();
                path
            })
            .collect()
    }

    #[test]
    fn test_new() {
        let manager = FileManager::new();
        assert_eq!(manager.total_count(), 0);
        assert_eq!(manager.current_index(), 0);
    }

    #[test]
    fn test_load_directory() {
        let temp_dir = TempDir::new().unwrap();
        let image_names = vec!["a.jpg", "b.png", "c.gif", "d.txt", "e.bmp"];
        create_test_images(temp_dir.path(), &image_names);

        let mut manager = FileManager::new();
        let first_image = temp_dir.path().join("a.jpg");
        manager.load_directory(&first_image).unwrap();

        assert_eq!(manager.total_count(), 4); // txt is excluded
        assert_eq!(manager.current_index(), 0);
    }

    #[test]
    fn test_navigation_next() {
        let temp_dir = TempDir::new().unwrap();
        let image_names = vec!["1.jpg", "2.jpg", "3.jpg"];
        create_test_images(temp_dir.path(), &image_names);

        let mut manager = FileManager::new();
        manager.load_directory(&temp_dir.path().join("1.jpg")).unwrap();

        assert_eq!(manager.current_index(), 0);

        manager.next();
        assert_eq!(manager.current_index(), 1);

        manager.next();
        assert_eq!(manager.current_index(), 2);

        // Wrap around
        manager.next();
        assert_eq!(manager.current_index(), 0);
    }

    #[test]
    fn test_navigation_previous() {
        let temp_dir = TempDir::new().unwrap();
        let image_names = vec!["1.jpg", "2.jpg", "3.jpg"];
        create_test_images(temp_dir.path(), &image_names);

        let mut manager = FileManager::new();
        manager.load_directory(&temp_dir.path().join("2.jpg")).unwrap();
        assert_eq!(manager.current_index(), 1);

        manager.previous();
        assert_eq!(manager.current_index(), 0);

        // Wrap around to end
        manager.previous();
        assert_eq!(manager.current_index(), 2);
    }

    #[test]
    fn test_jump_to() {
        let temp_dir = TempDir::new().unwrap();
        let image_names = vec!["1.jpg", "2.jpg", "3.jpg"];
        create_test_images(temp_dir.path(), &image_names);

        let mut manager = FileManager::new();
        manager.load_directory(&temp_dir.path().join("1.jpg")).unwrap();

        let result = manager.jump_to(2);
        assert!(result.is_some());
        assert_eq!(manager.current_index(), 2);

        let result = manager.jump_to(10);
        assert!(result.is_none());
        assert_eq!(manager.current_index(), 2); // Index unchanged
    }

    #[test]
    fn test_first_last() {
        let temp_dir = TempDir::new().unwrap();
        let image_names = vec!["1.jpg", "2.jpg", "3.jpg"];
        create_test_images(temp_dir.path(), &image_names);

        let mut manager = FileManager::new();
        manager.load_directory(&temp_dir.path().join("2.jpg")).unwrap();

        manager.last();
        assert_eq!(manager.current_index(), 2);

        manager.first();
        assert_eq!(manager.current_index(), 0);
    }

    #[test]
    fn test_is_supported_image() {
        assert!(FileManager::is_supported_image(Path::new("test.jpg")));
        assert!(FileManager::is_supported_image(Path::new("test.JPG")));
        assert!(FileManager::is_supported_image(Path::new("test.jpeg")));
        assert!(FileManager::is_supported_image(Path::new("test.png")));
        assert!(FileManager::is_supported_image(Path::new("test.gif")));
        assert!(FileManager::is_supported_image(Path::new("test.bmp")));
        assert!(FileManager::is_supported_image(Path::new("test.webp")));
        assert!(FileManager::is_supported_image(Path::new("test.tiff")));
        assert!(FileManager::is_supported_image(Path::new("test.tif")));

        assert!(!FileManager::is_supported_image(Path::new("test.txt")));
        assert!(!FileManager::is_supported_image(Path::new("test.mp4")));
        assert!(!FileManager::is_supported_image(Path::new("test")));
    }

    #[test]
    fn test_sort_files() {
        let temp_dir = TempDir::new().unwrap();
        let image_names = vec!["z.jpg", "a.jpg", "m.jpg", "b.jpg"];
        create_test_images(temp_dir.path(), &image_names);

        let mut manager = FileManager::new();
        manager.load_directory(&temp_dir.path().join("z.jpg")).unwrap();

        let files = manager.get_all_files();
        let names: Vec<&str> = files
            .iter()
            .map(|p| p.file_name().unwrap().to_str().unwrap())
            .collect();

        assert_eq!(names, vec!["a.jpg", "b.jpg", "m.jpg", "z.jpg"]);
    }

    #[test]
    fn test_empty_directory() {
        let temp_dir = TempDir::new().unwrap();

        let mut manager = FileManager::new();
        let result = manager.load_directory(temp_dir.path());

        assert!(result.is_ok());
        assert_eq!(manager.total_count(), 0);
        assert!(manager.get_current().is_none());
        assert!(manager.next().is_none());
        assert!(manager.previous().is_none());
    }
}
