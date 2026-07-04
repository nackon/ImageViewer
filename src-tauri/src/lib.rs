use std::path::PathBuf;
use tauri::{command, State};
use std::sync::Mutex;

#[derive(Default)]
struct AppState {
    current_images: Mutex<Vec<PathBuf>>,
    current_index: Mutex<usize>,
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![load_image, next_image, previous_image])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
