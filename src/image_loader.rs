use anyhow::Result;
use image::GenericImageView;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct ImageData {
    pub path: PathBuf,
    pub width: u32,
    pub height: u32,
    pub file_size: u64,
    pub handle: iced::widget::image::Handle,
}

impl ImageData {
    pub fn load(path: &Path) -> Result<Self> {
        let handle = iced::widget::image::Handle::from_path(path);

        let img = image::open(path)?;
        let (width, height) = img.dimensions();

        let file_size = std::fs::metadata(path)?.len();

        Ok(Self {
            path: path.to_path_buf(),
            width,
            height,
            file_size,
            handle,
        })
    }

    pub fn file_name(&self) -> String {
        self.path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Unknown")
            .to_string()
    }

    pub fn file_size_string(&self) -> String {
        let kb = self.file_size as f64 / 1024.0;
        if kb < 1024.0 {
            format!("{:.1} KB", kb)
        } else {
            let mb = kb / 1024.0;
            format!("{:.1} MB", mb)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{ImageBuffer, Rgb};
    use std::fs::File;
    use std::io::Write;
    use tempfile::TempDir;

    fn create_test_image(dir: &Path, name: &str, width: u32, height: u32) -> PathBuf {
        let path = dir.join(name);
        let img: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::from_fn(width, height, |x, y| {
            let r = (x % 256) as u8;
            let g = (y % 256) as u8;
            let b = ((x + y) % 256) as u8;
            Rgb([r, g, b])
        });
        img.save(&path).unwrap();
        path
    }

    #[test]
    fn test_load_image() {
        let temp_dir = TempDir::new().unwrap();
        let image_path = create_test_image(temp_dir.path(), "test.png", 100, 80);

        let result = ImageData::load(&image_path);
        assert!(result.is_ok());

        let image_data = result.unwrap();
        assert_eq!(image_data.width, 100);
        assert_eq!(image_data.height, 80);
        assert!(image_data.file_size > 0);
    }

    #[test]
    fn test_load_nonexistent_image() {
        let result = ImageData::load(Path::new("/nonexistent/path/image.jpg"));
        assert!(result.is_err());
    }

    #[test]
    fn test_load_invalid_image() {
        let temp_dir = TempDir::new().unwrap();
        let invalid_path = temp_dir.path().join("invalid.jpg");
        let mut file = File::create(&invalid_path).unwrap();
        file.write_all(b"This is not a valid image").unwrap();

        let result = ImageData::load(&invalid_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_file_name() {
        let temp_dir = TempDir::new().unwrap();
        let image_path = create_test_image(temp_dir.path(), "my_photo.png", 50, 50);

        let image_data = ImageData::load(&image_path).unwrap();
        assert_eq!(image_data.file_name(), "my_photo.png");
    }

    #[test]
    fn test_file_name_unknown() {
        let temp_dir = TempDir::new().unwrap();
        let image_path = create_test_image(temp_dir.path(), "test.png", 50, 50);

        let mut image_data = ImageData::load(&image_path).unwrap();
        image_data.path = PathBuf::from("");

        assert_eq!(image_data.file_name(), "Unknown");
    }

    #[test]
    fn test_file_size_string_kb() {
        let temp_dir = TempDir::new().unwrap();
        let image_path = create_test_image(temp_dir.path(), "small.png", 10, 10);

        let image_data = ImageData::load(&image_path).unwrap();
        let size_str = image_data.file_size_string();

        assert!(size_str.ends_with(" KB"));
        assert!(size_str.contains('.'));
    }

    #[test]
    fn test_file_size_string_mb() {
        let temp_dir = TempDir::new().unwrap();
        let image_path = create_test_image(temp_dir.path(), "large.png", 1000, 1000);

        let image_data = ImageData::load(&image_path).unwrap();
        let size_str = image_data.file_size_string();

        // Large images should be in MB
        if image_data.file_size > 1024 * 1024 {
            assert!(size_str.ends_with(" MB"));
        }
    }

    #[test]
    fn test_different_formats() {
        let temp_dir = TempDir::new().unwrap();

        for (ext, width, height) in [("png", 100, 100), ("jpg", 100, 100), ("bmp", 50, 50)] {
            let filename = format!("test.{}", ext);
            let image_path = create_test_image(temp_dir.path(), &filename, width, height);

            let result = ImageData::load(&image_path);
            assert!(result.is_ok(), "Failed to load {}", ext);

            let image_data = result.unwrap();
            assert_eq!(image_data.width, width);
            assert_eq!(image_data.height, height);
        }
    }

    #[test]
    fn test_large_image() {
        let temp_dir = TempDir::new().unwrap();
        let image_path = create_test_image(temp_dir.path(), "large.png", 2000, 1500);

        let result = ImageData::load(&image_path);
        assert!(result.is_ok());

        let image_data = result.unwrap();
        assert_eq!(image_data.width, 2000);
        assert_eq!(image_data.height, 1500);
    }

    #[test]
    fn test_small_image() {
        let temp_dir = TempDir::new().unwrap();
        let image_path = create_test_image(temp_dir.path(), "tiny.png", 1, 1);

        let result = ImageData::load(&image_path);
        assert!(result.is_ok());

        let image_data = result.unwrap();
        assert_eq!(image_data.width, 1);
        assert_eq!(image_data.height, 1);
    }
}
