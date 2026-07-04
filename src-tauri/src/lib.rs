use std::path::PathBuf;
use tauri::{command, State, Manager, Emitter};
use std::sync::Mutex;

#[derive(Default)]
struct AppState {
    current_images: Mutex<Vec<PathBuf>>,
    current_index: Mutex<usize>,
    pending_paths: Mutex<Vec<PathBuf>>, // 起動時に受け取ったファイルパスを保存
}

#[command]
fn load_image(path: String, state: State<AppState>) -> Result<String, String> {
    let path = PathBuf::from(&path);

    // Get parent directory
    let parent = path.parent().ok_or("No parent directory")?;

    // Find all image files in the directory
    let mut images: Vec<PathBuf> = std::fs::read_dir(parent)
        .map_err(|e| e.to_string())?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|p| {
            p.extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| matches!(ext.to_lowercase().as_str(), "jpg" | "jpeg" | "png" | "gif" | "bmp" | "webp"))
                .unwrap_or(false)
        })
        .collect();

    images.sort();

    // Find current image index
    let index = images.iter().position(|p| p == &path).unwrap_or(0);

    *state.current_images.lock().unwrap() = images;
    *state.current_index.lock().unwrap() = index;

    Ok(path.to_string_lossy().to_string())
}

#[command]
fn next_image(state: State<AppState>) -> Result<Option<String>, String> {
    let images = state.current_images.lock().unwrap();
    let mut index = state.current_index.lock().unwrap();

    if images.is_empty() {
        return Ok(None);
    }

    *index = (*index + 1) % images.len();
    Ok(Some(images[*index].to_string_lossy().to_string()))
}

#[command]
fn previous_image(state: State<AppState>) -> Result<Option<String>, String> {
    let images = state.current_images.lock().unwrap();
    let mut index = state.current_index.lock().unwrap();

    if images.is_empty() {
        return Ok(None);
    }

    *index = if *index == 0 {
        images.len() - 1
    } else {
        *index - 1
    };

    Ok(Some(images[*index].to_string_lossy().to_string()))
}

#[command]
fn frontend_ready(_app: tauri::AppHandle, state: State<AppState>) -> Result<Vec<String>, String> {
    println!("[Rust] frontend_ready command called");
    let mut paths = state.pending_paths.lock().unwrap();
    let result: Vec<String> = paths.iter().map(|p| p.to_string_lossy().to_string()).collect();

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
        .invoke_handler(tauri::generate_handler![load_image, next_image, previous_image, frontend_ready])
        .setup(|app| {
            // コマンドライン引数をチェック（アプリ起動時のダブルクリックの場合）
            let args: Vec<String> = std::env::args().collect();
            println!("[Rust] Startup args: {:?}", args);

            // 画像ファイルを探す
            let image_files: Vec<PathBuf> = args.iter().skip(1)
                .filter(|arg| {
                    let lower = arg.to_lowercase();
                    lower.ends_with(".jpg") ||
                    lower.ends_with(".jpeg") ||
                    lower.ends_with(".png") ||
                    lower.ends_with(".gif") ||
                    lower.ends_with(".bmp") ||
                    lower.ends_with(".webp")
                })
                .map(PathBuf::from)
                .collect();

            println!("[Rust] Found {} image files in args", image_files.len());

            if !image_files.is_empty() {
                let state: tauri::State<AppState> = app.state();
                let mut pending = state.pending_paths.lock().unwrap();
                for path in image_files {
                    println!("[Rust] Buffering startup file: {}", path.display());
                    pending.push(path);
                }
            }

            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    // macOS でダブルクリック（起動時・実行中問わず）されたすべてのApple Event（URL）をここでキャッチ
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_state_default() {
        let state = AppState::default();
        assert!(state.current_images.lock().unwrap().is_empty());
        assert_eq!(*state.current_index.lock().unwrap(), 0);
        assert!(state.pending_paths.lock().unwrap().is_empty());
    }

    #[test]
    fn test_next_image_logic() {
        let state = AppState::default();

        // Empty list
        {
            let images = state.current_images.lock().unwrap();
            assert!(images.is_empty());
        }

        // Test with images
        *state.current_images.lock().unwrap() = vec![
            PathBuf::from("/test/image1.jpg"),
            PathBuf::from("/test/image2.jpg"),
            PathBuf::from("/test/image3.jpg"),
        ];
        *state.current_index.lock().unwrap() = 0;

        // Next from 0 -> 1
        {
            let images = state.current_images.lock().unwrap();
            let mut index = state.current_index.lock().unwrap();
            *index = (*index + 1) % images.len();
            assert_eq!(*index, 1);
        }

        // Next from 2 -> 0 (wrap around)
        {
            *state.current_index.lock().unwrap() = 2;
            let images = state.current_images.lock().unwrap();
            let mut index = state.current_index.lock().unwrap();
            *index = (*index + 1) % images.len();
            assert_eq!(*index, 0);
        }
    }

    #[test]
    fn test_previous_image_logic() {
        let state = AppState::default();

        // Test with images
        *state.current_images.lock().unwrap() = vec![
            PathBuf::from("/test/image1.jpg"),
            PathBuf::from("/test/image2.jpg"),
            PathBuf::from("/test/image3.jpg"),
        ];
        *state.current_index.lock().unwrap() = 1;

        // Previous from 1 -> 0
        {
            let images = state.current_images.lock().unwrap();
            let mut index = state.current_index.lock().unwrap();
            *index = if *index == 0 {
                images.len() - 1
            } else {
                *index - 1
            };
            assert_eq!(*index, 0);
        }

        // Previous from 0 -> 2 (wrap around)
        {
            *state.current_index.lock().unwrap() = 0;
            let images = state.current_images.lock().unwrap();
            let mut index = state.current_index.lock().unwrap();
            *index = if *index == 0 {
                images.len() - 1
            } else {
                *index - 1
            };
            assert_eq!(*index, 2);
        }
    }

    #[test]
    fn test_image_extension_filter() {
        let valid_extensions = ["jpg", "jpeg", "png", "gif", "bmp", "webp"];
        for ext in valid_extensions {
            let path = PathBuf::from(format!("/test/image.{}", ext));
            assert!(path.extension()
                .and_then(|e| e.to_str())
                .map(|e| matches!(e.to_lowercase().as_str(), "jpg" | "jpeg" | "png" | "gif" | "bmp" | "webp"))
                .unwrap_or(false));
        }

        let invalid_extensions = ["txt", "pdf", "doc"];
        for ext in invalid_extensions {
            let path = PathBuf::from(format!("/test/file.{}", ext));
            assert!(!path.extension()
                .and_then(|e| e.to_str())
                .map(|e| matches!(e.to_lowercase().as_str(), "jpg" | "jpeg" | "png" | "gif" | "bmp" | "webp"))
                .unwrap_or(false));
        }
    }

    #[test]
    fn test_image_list_sorting() {
        let mut images = vec![
            PathBuf::from("/test/image3.jpg"),
            PathBuf::from("/test/image1.jpg"),
            PathBuf::from("/test/image2.jpg"),
        ];
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
}
