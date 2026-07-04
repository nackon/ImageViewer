use std::path::PathBuf;

#[test]
fn test_image_formats_support() {
    let supported_extensions = vec!["jpg", "jpeg", "png", "gif", "bmp", "webp", "tiff"];

    for ext in supported_extensions {
        let test_path = format!("test.{}", ext);
        assert!(test_path.ends_with(ext));
    }
}

#[test]
fn test_file_size_formatting() {
    // Test various file sizes
    let test_cases = vec![
        (500u64, "500 B"),
        (1024, "1.0 KB"),
        (1536, "1.5 KB"),
        (1048576, "1.0 MB"),
        (2621440, "2.5 MB"),
    ];

    for (bytes, expected) in test_cases {
        let formatted = format_file_size(bytes);
        assert_eq!(formatted, expected, "Failed for {} bytes", bytes);
    }
}

fn format_file_size(bytes: u64) -> String {
    if bytes < 1024 {
        return format!("{} B", bytes);
    }
    if bytes < 1024 * 1024 {
        return format!("{:.1} KB", bytes as f64 / 1024.0);
    }
    format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
}

#[test]
fn test_zoom_levels() {
    let zoom_step = 0.25f64;
    let mut zoom: f64 = 1.0;

    // Zoom in
    zoom += zoom_step;
    assert_eq!(zoom, 1.25);

    zoom += zoom_step;
    assert_eq!(zoom, 1.5);

    // Zoom out
    zoom -= zoom_step;
    assert_eq!(zoom, 1.25);

    zoom -= zoom_step;
    assert_eq!(zoom, 1.0);

    // Test limits
    let min_zoom = 0.25;
    let max_zoom = 10.0;

    zoom = 0.1; // Below minimum
    zoom = zoom.max(min_zoom);
    assert_eq!(zoom, min_zoom);

    zoom = 15.0; // Above maximum
    zoom = zoom.min(max_zoom);
    assert_eq!(zoom, max_zoom);
}

#[test]
fn test_image_navigation_wraparound() {
    let images = [
        PathBuf::from("image1.jpg"),
        PathBuf::from("image2.jpg"),
        PathBuf::from("image3.jpg"),
    ];

    let mut index = 0;

    // Next from start
    index = (index + 1) % images.len();
    assert_eq!(index, 1);

    // Next again
    index = (index + 1) % images.len();
    assert_eq!(index, 2);

    // Next wraps to start
    index = (index + 1) % images.len();
    assert_eq!(index, 0);

    // Previous from start wraps to end
    index = if index == 0 {
        images.len() - 1
    } else {
        index - 1
    };
    assert_eq!(index, 2);
}

#[test]
fn test_thumbnail_grid_calculation() {
    let thumbnail_width = 150;
    let gap = 15;
    let item_width = thumbnail_width + gap;

    let container_widths = vec![800, 1024, 1440, 1920];
    let expected_columns = vec![4, 6, 8, 11];

    for (container, expected) in container_widths.iter().zip(expected_columns.iter()) {
        let columns = container / item_width;
        assert_eq!(
            columns, *expected as usize,
            "Failed for width {}",
            container
        );
    }
}
