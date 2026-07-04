use std::path::PathBuf;
use tauri::{command, State, Manager, Emitter};
use std::sync::Mutex;
use serde::{Deserialize, Serialize};
use image::ImageReader;

#[derive(Default)]
struct AppState {
    current_images: Mutex<Vec<PathBuf>>,
    current_index: Mutex<usize>,
    pending_paths: Mutex<Vec<PathBuf>>,
    thumbnail_cache: Mutex<std::collections::HashMap<PathBuf, PathBuf>>,
}

#[derive(Serialize, Deserialize)]
struct ImageInfo {
    width: u32,
    height: u32,
    size: u64,
    images: Vec<String>,
    index: usize,
}

#[command]
fn load_image(path: String, state: State<AppState>) -> Result<ImageInfo, String> {
    let path = PathBuf::from(&path);

    // Get image dimensions and file size
    let img = ImageReader::open(&path)
        .map_err(|e| e.to_string())?
        .decode()
        .map_err(|e| e.to_string())?;

    let width = img.width();
    let height = img.height();

    let size = std::fs::metadata(&path)
        .map_err(|e| e.to_string())?
        .len();

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
                .map(|ext| matches!(ext.to_lowercase().as_str(), "jpg" | "jpeg" | "png" | "gif" | "bmp" | "webp" | "tiff"))
                .unwrap_or(false)
        })
        .collect();

    images.sort();

    // Find current image index
    let index = images.iter().position(|p| p == &path).unwrap_or(0);

    let images_str: Vec<String> = images.iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect();

    *state.current_images.lock().unwrap() = images;
    *state.current_index.lock().unwrap() = index;

    Ok(ImageInfo {
        width,
        height,
        size,
        images: images_str,
        index,
    })
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
fn first_image(state: State<AppState>) -> Result<Option<String>, String> {
    let images = state.current_images.lock().unwrap();
    let mut index = state.current_index.lock().unwrap();

    if images.is_empty() {
        return Ok(None);
    }

    *index = 0;
    Ok(Some(images[*index].to_string_lossy().to_string()))
}

#[command]
fn last_image(state: State<AppState>) -> Result<Option<String>, String> {
    let images = state.current_images.lock().unwrap();
    let mut index = state.current_index.lock().unwrap();

    if images.is_empty() {
        return Ok(None);
    }

    *index = images.len() - 1;
    Ok(Some(images[*index].to_string_lossy().to_string()))
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

    let size = std::fs::metadata(&path)
        .map_err(|e| e.to_string())?
        .len();

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
        .plugin(tauri_plugin_process::init())
        .manage(AppState::default())
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

            // 画像ファイルを探す
            let image_files: Vec<PathBuf> = args.iter().skip(1)
                .filter(|arg| {
                    let lower = arg.to_lowercase();
                    lower.ends_with(".jpg") ||
                    lower.ends_with(".jpeg") ||
                    lower.ends_with(".png") ||
                    lower.ends_with(".gif") ||
                    lower.ends_with(".bmp") ||
                    lower.ends_with(".webp") ||
                    lower.ends_with(".tiff")
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
