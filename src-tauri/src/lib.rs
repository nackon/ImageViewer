use image::ImageReader;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use tauri::{command, Manager, State, WebviewWindow};

#[cfg(target_os = "macos")]
use tauri::Emitter;

mod menu;

// ウィンドウごとの状態
#[derive(Clone, Default)]
struct WindowState {
    current_images: Vec<PathBuf>,
    current_index: usize,
}

// アプリ全体の状態（ウィンドウごとの状態を管理）
#[derive(Default)]
struct AppState {
    windows: Mutex<HashMap<String, WindowState>>,
    pending_paths: Mutex<Vec<PathBuf>>,
    thumbnail_cache: Mutex<HashMap<PathBuf, PathBuf>>,
}

#[derive(Serialize, Deserialize)]
struct ImageInfo {
    width: u32,
    height: u32,
    size: u64,
    images: Vec<String>,
    index: usize,
}

fn is_image_file(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| {
            matches!(
                ext.to_lowercase().as_str(),
                "jpg" | "jpeg" | "png" | "gif" | "bmp" | "webp"
            )
        })
        .unwrap_or(false)
}

fn load_images_from_directory(dir: &Path) -> Result<Vec<PathBuf>, String> {
    let mut images: Vec<PathBuf> = std::fs::read_dir(dir)
        .map_err(|e| e.to_string())?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|p| is_image_file(p))
        .collect();

    images.sort();
    Ok(images)
}

#[command]
fn load_image(
    window: WebviewWindow,
    path: String,
    state: State<AppState>,
) -> Result<ImageInfo, String> {
    let path = PathBuf::from(&path);

    // Get image dimensions and file size
    let img = ImageReader::open(&path)
        .map_err(|e| e.to_string())?
        .decode()
        .map_err(|e| e.to_string())?;

    let width = img.width();
    let height = img.height();

    let size = std::fs::metadata(&path).map_err(|e| e.to_string())?.len();

    // Get parent directory
    let parent = path.parent().ok_or("No parent directory")?;

    // Find all image files in the directory
    let images = load_images_from_directory(parent)?;

    // Find current image index
    let index = images.iter().position(|p| p == &path).unwrap_or(0);

    let images_str: Vec<String> = images
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect();

    // ウィンドウごとの状態を更新
    let window_label = window.label().to_string();
    let mut windows = state.windows.lock().unwrap();
    windows.insert(
        window_label,
        WindowState {
            current_images: images,
            current_index: index,
        },
    );

    Ok(ImageInfo {
        width,
        height,
        size,
        images: images_str,
        index,
    })
}

#[command]
fn next_image(window: WebviewWindow, state: State<AppState>) -> Result<Option<String>, String> {
    let window_label = window.label().to_string();
    let mut windows = state.windows.lock().unwrap();

    let window_state = windows
        .get_mut(&window_label)
        .ok_or("Window state not found")?;

    if window_state.current_images.is_empty() {
        return Ok(None);
    }

    window_state.current_index =
        (window_state.current_index + 1) % window_state.current_images.len();
    Ok(Some(
        window_state.current_images[window_state.current_index]
            .to_string_lossy()
            .to_string(),
    ))
}

#[command]
fn previous_image(window: WebviewWindow, state: State<AppState>) -> Result<Option<String>, String> {
    let window_label = window.label().to_string();
    let mut windows = state.windows.lock().unwrap();

    let window_state = windows
        .get_mut(&window_label)
        .ok_or("Window state not found")?;

    if window_state.current_images.is_empty() {
        return Ok(None);
    }

    window_state.current_index = if window_state.current_index == 0 {
        window_state.current_images.len() - 1
    } else {
        window_state.current_index - 1
    };

    Ok(Some(
        window_state.current_images[window_state.current_index]
            .to_string_lossy()
            .to_string(),
    ))
}

#[command]
fn first_image(window: WebviewWindow, state: State<AppState>) -> Result<Option<String>, String> {
    let window_label = window.label().to_string();
    let mut windows = state.windows.lock().unwrap();

    let window_state = windows
        .get_mut(&window_label)
        .ok_or("Window state not found")?;

    if window_state.current_images.is_empty() {
        return Ok(None);
    }

    window_state.current_index = 0;
    Ok(Some(
        window_state.current_images[window_state.current_index]
            .to_string_lossy()
            .to_string(),
    ))
}

#[command]
fn last_image(window: WebviewWindow, state: State<AppState>) -> Result<Option<String>, String> {
    let window_label = window.label().to_string();
    let mut windows = state.windows.lock().unwrap();

    let window_state = windows
        .get_mut(&window_label)
        .ok_or("Window state not found")?;

    if window_state.current_images.is_empty() {
        return Ok(None);
    }

    window_state.current_index = window_state.current_images.len() - 1;
    Ok(Some(
        window_state.current_images[window_state.current_index]
            .to_string_lossy()
            .to_string(),
    ))
}

#[command]
fn get_image_info(path: String) -> Result<ImageInfo, String> {
    let path = PathBuf::from(&path);

    // Get image dimensions and file size
    let img = ImageReader::open(&path)
        .map_err(|e| e.to_string())?
        .decode()
        .map_err(|e| e.to_string())?;

    let width = img.width();
    let height = img.height();

    let size = std::fs::metadata(&path).map_err(|e| e.to_string())?.len();

    Ok(ImageInfo {
        width,
        height,
        size,
        images: vec![],
        index: 0,
    })
}

#[command]
fn get_thumbnail(path: String, state: State<AppState>) -> Result<String, String> {
    let path = PathBuf::from(&path);

    // Check cache first
    {
        let cache = state.thumbnail_cache.lock().unwrap();
        if let Some(cached_path) = cache.get(&path) {
            if cached_path.exists() {
                return Ok(cached_path.to_string_lossy().to_string());
            }
        }
    }

    // Generate thumbnail
    let img = ImageReader::open(&path)
        .map_err(|e| e.to_string())?
        .decode()
        .map_err(|e| e.to_string())?;

    // Resize to 150x150 maintaining aspect ratio
    let thumbnail = img.thumbnail(150, 150);

    // Save thumbnail to temp directory
    let temp_dir = std::env::temp_dir().join("imageviewer_thumbnails");
    std::fs::create_dir_all(&temp_dir).map_err(|e| e.to_string())?;

    let file_name = path.file_name().ok_or("Invalid file name")?;
    let thumbnail_path = temp_dir.join(file_name);

    thumbnail.save(&thumbnail_path).map_err(|e| e.to_string())?;

    // Cache the thumbnail path
    {
        let mut cache = state.thumbnail_cache.lock().unwrap();
        cache.insert(path, thumbnail_path.clone());
    }

    Ok(thumbnail_path.to_string_lossy().to_string())
}

#[command]
fn frontend_ready(window: WebviewWindow, state: State<AppState>) -> Result<Vec<String>, String> {
    println!(
        "[Rust] frontend_ready command called for window: {}",
        window.label()
    );
    let mut paths = state.pending_paths.lock().unwrap();
    let result: Vec<String> = paths
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect();

    if !result.is_empty() {
        println!("[Rust] Returning {} buffered paths", result.len());
        paths.clear();
    }

    Ok(result)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(AppState::default())
        .menu(menu::build_menu)
        .on_menu_event(menu::handle_menu_event)
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { .. } = event {
                window.app_handle().exit(0);
            }
        })
        .invoke_handler(tauri::generate_handler![
            load_image,
            next_image,
            previous_image,
            first_image,
            last_image,
            get_image_info,
            get_thumbnail,
            frontend_ready
        ])
        .setup(|app| {
            // コマンドライン引数をチェック（アプリ起動時のダブルクリックの場合）
            let args: Vec<String> = std::env::args().collect();
            println!("[Rust] Startup args: {:?}", args);

            let state: tauri::State<AppState> = app.state();
            let mut pending = state.pending_paths.lock().unwrap();

            for arg in args.iter().skip(1) {
                let path = PathBuf::from(arg);

                if path.is_file() && is_image_file(&path) {
                    // 画像ファイルの場合
                    println!("[Rust] Buffering startup file: {}", path.display());
                    pending.push(path);
                } else if path.is_dir() {
                    // フォルダの場合、中の画像ファイルを全て追加
                    println!("[Rust] Found directory argument: {}", path.display());
                    if let Ok(images) = load_images_from_directory(&path) {
                        println!("[Rust] Found {} images in directory", images.len());
                        for img in images {
                            println!("[Rust] Buffering image from dir: {}", img.display());
                            pending.push(img);
                        }
                    }
                }
            }

            println!("[Rust] Total buffered paths: {}", pending.len());

            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    // macOS でダブルクリック（起動時・実行中問わず）されたすべてのApple Event（URL）をここでキャッチ
    #[cfg(target_os = "macos")]
    app.run(|app_handle, event| {
        if let tauri::RunEvent::Opened { urls } = event {
            println!("[Rust] RunEvent::Opened received, {} URLs", urls.len());
            for url in urls {
                println!("[Rust] URL: {}", url);
                if let Ok(path) = url.to_file_path() {
                    println!("[Rust] File path: {}", path.display());

                    // AppStateを取得
                    let state: tauri::State<AppState> = app_handle.state();

                    // ① すでにウィンドウが存在し、フロントエンドが準備完了している場合
                    if let Some(window) = app_handle.get_webview_window("main") {
                        let path_str = path.to_string_lossy().to_string();
                        println!("[Rust] Window exists, emitting immediately: {}", path_str);
                        let _ = window.emit("open-file-from-os", path_str);
                    } else {
                        // ② アプリ起動プロセスの途中の場合（未起動からのコールドスタート）
                        // フロントエンドの準備ができるまで pending_paths に一旦退避させる
                        println!("[Rust] Window not ready, buffering: {}", path.display());
                        state.pending_paths.lock().unwrap().push(path);
                    }
                }
            }
        }
    });

    #[cfg(not(target_os = "macos"))]
    app.run(|_app_handle, _event| {});
}

#[cfg(test)]
mod tests {
    use super::*;

    // Regression test for issue #18: pressing Q (or Escape outside fullscreen)
    // calls `getCurrentWindow().close()` from the frontend, which requires
    // this permission. Without it the call silently fails (unhandled promise
    // rejection) and the window never closes.
    #[test]
    fn capabilities_grant_window_close_permission() {
        let capabilities: serde_json::Value =
            serde_json::from_str(include_str!("../capabilities/default.json"))
                .expect("capabilities/default.json should be valid JSON");
        let permissions = capabilities["permissions"]
            .as_array()
            .expect("permissions should be an array");
        assert!(
            permissions.iter().any(|p| p == "core:window:allow-close"),
            "capabilities/default.json must grant core:window:allow-close so the \
             frontend can actually close the window (see issue #18)"
        );
    }

    #[test]
    fn test_app_state_default() {
        let state = AppState::default();
        assert!(state.windows.lock().unwrap().is_empty());
        assert!(state.pending_paths.lock().unwrap().is_empty());
        assert!(state.thumbnail_cache.lock().unwrap().is_empty());
    }

    #[test]
    fn test_window_state_default() {
        let window_state = WindowState::default();
        assert!(window_state.current_images.is_empty());
        assert_eq!(window_state.current_index, 0);
    }

    #[test]
    fn test_multi_window_state_isolation() {
        let state = AppState::default();
        let mut windows = state.windows.lock().unwrap();

        // Window 1の状態
        windows.insert(
            "window-1".to_string(),
            WindowState {
                current_images: vec![
                    PathBuf::from("/test/window1/image1.jpg"),
                    PathBuf::from("/test/window1/image2.jpg"),
                ],
                current_index: 0,
            },
        );

        // Window 2の状態
        windows.insert(
            "window-2".to_string(),
            WindowState {
                current_images: vec![
                    PathBuf::from("/test/window2/imageA.jpg"),
                    PathBuf::from("/test/window2/imageB.jpg"),
                    PathBuf::from("/test/window2/imageC.jpg"),
                ],
                current_index: 1,
            },
        );

        // 各ウィンドウの状態が独立していることを確認
        let window1 = windows.get("window-1").unwrap();
        assert_eq!(window1.current_images.len(), 2);
        assert_eq!(window1.current_index, 0);

        let window2 = windows.get("window-2").unwrap();
        assert_eq!(window2.current_images.len(), 3);
        assert_eq!(window2.current_index, 1);
    }

    #[test]
    fn test_next_image_logic() {
        let images = [
            PathBuf::from("/test/image1.jpg"),
            PathBuf::from("/test/image2.jpg"),
            PathBuf::from("/test/image3.jpg"),
        ];

        // Next from 0 -> 1
        {
            let index = 0;
            let next_index = (index + 1) % images.len();
            assert_eq!(next_index, 1);
        }

        // Next from 2 -> 0 (wrap around)
        {
            let index = 2;
            let next_index = (index + 1) % images.len();
            assert_eq!(next_index, 0);
        }
    }

    #[test]
    fn test_previous_image_logic() {
        let images = [
            PathBuf::from("/test/image1.jpg"),
            PathBuf::from("/test/image2.jpg"),
            PathBuf::from("/test/image3.jpg"),
        ];

        // Previous from 1 -> 0
        {
            let index = 1;
            let prev_index = if index == 0 {
                images.len() - 1
            } else {
                index - 1
            };
            assert_eq!(prev_index, 0);
        }

        // Previous from 0 -> 2 (wrap around)
        {
            let index = 0;
            let prev_index = if index == 0 {
                images.len() - 1
            } else {
                index - 1
            };
            assert_eq!(prev_index, 2);
        }
    }

    #[test]
    fn test_image_extension_filter() {
        let valid_extensions = ["jpg", "jpeg", "png", "gif", "bmp", "webp"];
        for ext in valid_extensions {
            let path = PathBuf::from(format!("/test/image.{}", ext));
            assert!(is_image_file(&path));
        }

        let invalid_extensions = ["txt", "pdf", "doc"];
        for ext in invalid_extensions {
            let path = PathBuf::from(format!("/test/file.{}", ext));
            assert!(!is_image_file(&path));
        }
    }

    #[test]
    fn test_image_list_sorting() {
        let mut images = [
            PathBuf::from("/test/image3.jpg"),
            PathBuf::from("/test/image1.jpg"),
            PathBuf::from("/test/image2.jpg"),
        ]
        .to_vec();
        images.sort();

        assert_eq!(images[0], PathBuf::from("/test/image1.jpg"));
        assert_eq!(images[1], PathBuf::from("/test/image2.jpg"));
        assert_eq!(images[2], PathBuf::from("/test/image3.jpg"));
    }

    #[test]
    fn test_pending_paths_buffer() {
        let state = AppState::default();
        let paths = vec![
            PathBuf::from("/test/image1.jpg"),
            PathBuf::from("/test/image2.jpg"),
        ];

        // Add paths to buffer
        {
            let mut pending = state.pending_paths.lock().unwrap();
            for path in paths.clone() {
                pending.push(path);
            }
        }

        // Verify buffer contains paths
        {
            let pending = state.pending_paths.lock().unwrap();
            assert_eq!(pending.len(), 2);
            assert_eq!(pending[0], paths[0]);
            assert_eq!(pending[1], paths[1]);
        }

        // Clear buffer
        {
            let mut pending = state.pending_paths.lock().unwrap();
            pending.clear();
            assert!(pending.is_empty());
        }
    }

    #[test]
    fn test_is_image_file() {
        // Valid image extensions
        assert!(is_image_file(&PathBuf::from("/test/image.jpg")));
        assert!(is_image_file(&PathBuf::from("/test/image.JPG")));
        assert!(is_image_file(&PathBuf::from("/test/image.jpeg")));
        assert!(is_image_file(&PathBuf::from("/test/image.png")));
        assert!(is_image_file(&PathBuf::from("/test/image.gif")));
        assert!(is_image_file(&PathBuf::from("/test/image.bmp")));
        assert!(is_image_file(&PathBuf::from("/test/image.webp")));

        // Invalid extensions
        assert!(!is_image_file(&PathBuf::from("/test/file.txt")));
        assert!(!is_image_file(&PathBuf::from("/test/file.pdf")));
        assert!(!is_image_file(&PathBuf::from("/test/file.doc")));
        assert!(!is_image_file(&PathBuf::from("/test/file")));
    }

    #[test]
    fn test_load_images_from_directory_with_temp_dir() {
        use std::fs;

        // Create a temporary directory
        let temp_dir =
            std::env::temp_dir().join(format!("image_viewer_test_{}", std::process::id()));
        fs::create_dir_all(&temp_dir).unwrap();

        // Create test files
        fs::write(temp_dir.join("image1.jpg"), b"fake jpg").unwrap();
        fs::write(temp_dir.join("image2.png"), b"fake png").unwrap();
        fs::write(temp_dir.join("not_image.txt"), b"text file").unwrap();
        fs::write(temp_dir.join("image3.gif"), b"fake gif").unwrap();

        // Test loading images
        let images = load_images_from_directory(&temp_dir).unwrap();

        // Should find 3 images (jpg, png, gif) sorted alphabetically
        assert_eq!(images.len(), 3);
        assert!(images[0]
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .contains("image1.jpg"));
        assert!(images[1]
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .contains("image2.png"));
        assert!(images[2]
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .contains("image3.gif"));

        // Clean up
        fs::remove_dir_all(&temp_dir).unwrap();
    }

    #[test]
    fn test_load_images_from_empty_directory() {
        use std::fs;

        let temp_dir =
            std::env::temp_dir().join(format!("image_viewer_test_empty_{}", std::process::id()));
        fs::create_dir_all(&temp_dir).unwrap();

        // No image files
        fs::write(temp_dir.join("readme.txt"), b"text file").unwrap();

        let images = load_images_from_directory(&temp_dir).unwrap();
        assert_eq!(images.len(), 0);

        fs::remove_dir_all(&temp_dir).unwrap();
    }
}
