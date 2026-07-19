use tauri::menu::{Menu, MenuItem, PredefinedMenuItem, Submenu};
use tauri::{AppHandle, Emitter, Manager, Runtime};

/// Ids emitted to the frontend (via the `menu-command` event) when a menu
/// item that mirrors an existing keyboard shortcut is clicked.
pub const ACTION_NEXT_IMAGE: &str = "next-image";
pub const ACTION_PREVIOUS_IMAGE: &str = "previous-image";
pub const ACTION_FIRST_IMAGE: &str = "first-image";
pub const ACTION_LAST_IMAGE: &str = "last-image";
pub const ACTION_ZOOM_IN: &str = "zoom-in";
pub const ACTION_ZOOM_OUT: &str = "zoom-out";
pub const ACTION_ACTUAL_SIZE: &str = "actual-size";
pub const ACTION_FIT_TO_WINDOW: &str = "fit-to-window";
pub const ACTION_TOGGLE_THUMBNAILS: &str = "toggle-thumbnails";
pub const ACTION_OPEN_FILE: &str = "open-file-dialog";
pub const ACTION_OPEN_FOLDER: &str = "open-folder-dialog";

/// All ids that should be forwarded to the frontend as a `menu-command` event.
const FORWARDED_ACTIONS: &[&str] = &[
    ACTION_NEXT_IMAGE,
    ACTION_PREVIOUS_IMAGE,
    ACTION_FIRST_IMAGE,
    ACTION_LAST_IMAGE,
    ACTION_ZOOM_IN,
    ACTION_ZOOM_OUT,
    ACTION_ACTUAL_SIZE,
    ACTION_FIT_TO_WINDOW,
    ACTION_TOGGLE_THUMBNAILS,
    ACTION_OPEN_FILE,
    ACTION_OPEN_FOLDER,
];

/// Builds the application menu bar. Every command that already has a
/// keyboard shortcut in the frontend (see `src/main.js`) gets a matching
/// menu item so it is discoverable and clickable, not just triggerable via
/// the keyboard. Accelerators are only assigned to keys that are unambiguous
/// across both the image view and the thumbnail view, so the native
/// accelerator can never shadow a context-dependent shortcut (e.g. the
/// arrow keys, which move the thumbnail grid selection while in thumbnail
/// view).
pub fn build_menu<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<Menu<R>> {
    let go_menu = Submenu::with_items(
        app,
        "Go",
        true,
        &[
            &MenuItem::with_id(app, ACTION_NEXT_IMAGE, "Next Image", true, Some("N"))?,
            &MenuItem::with_id(
                app,
                ACTION_PREVIOUS_IMAGE,
                "Previous Image",
                true,
                Some("P"),
            )?,
            &PredefinedMenuItem::separator(app)?,
            &MenuItem::with_id(app, ACTION_FIRST_IMAGE, "First Image", true, Some("Home"))?,
            &MenuItem::with_id(app, ACTION_LAST_IMAGE, "Last Image", true, Some("End"))?,
        ],
    )?;

    let view_menu = Submenu::with_items(
        app,
        "View",
        true,
        &[
            &MenuItem::with_id(app, ACTION_ZOOM_IN, "Zoom In", true, Some("="))?,
            &MenuItem::with_id(app, ACTION_ZOOM_OUT, "Zoom Out", true, Some("-"))?,
            &MenuItem::with_id(app, ACTION_ACTUAL_SIZE, "Actual Size", true, Some("0"))?,
            &MenuItem::with_id(app, ACTION_FIT_TO_WINDOW, "Fit to Window", true, Some("F"))?,
            &PredefinedMenuItem::separator(app)?,
            &MenuItem::with_id(
                app,
                ACTION_TOGGLE_THUMBNAILS,
                "Show Thumbnails",
                true,
                Some("T"),
            )?,
            #[cfg(target_os = "macos")]
            &PredefinedMenuItem::separator(app)?,
            #[cfg(target_os = "macos")]
            &PredefinedMenuItem::fullscreen(app, None)?,
        ],
    )?;

    let file_menu = Submenu::with_items(
        app,
        "File",
        true,
        &[
            &MenuItem::with_id(
                app,
                ACTION_OPEN_FILE,
                "Open File...",
                true,
                Some("CmdOrCtrl+O"),
            )?,
            &MenuItem::with_id(
                app,
                ACTION_OPEN_FOLDER,
                "Open Folder...",
                true,
                Some("CmdOrCtrl+Shift+O"),
            )?,
            &PredefinedMenuItem::separator(app)?,
            &PredefinedMenuItem::close_window(app, None)?,
            #[cfg(not(target_os = "macos"))]
            &PredefinedMenuItem::quit(app, None)?,
        ],
    )?;

    let window_menu = Submenu::with_items(
        app,
        "Window",
        true,
        &[
            &PredefinedMenuItem::minimize(app, None)?,
            &PredefinedMenuItem::maximize(app, None)?,
            #[cfg(target_os = "macos")]
            &PredefinedMenuItem::separator(app)?,
            &PredefinedMenuItem::close_window(app, None)?,
        ],
    )?;

    let help_menu = Submenu::with_items(
        app,
        "Help",
        true,
        &[
            #[cfg(not(target_os = "macos"))]
            &PredefinedMenuItem::about(app, None, None)?,
        ],
    )?;

    #[cfg(target_os = "macos")]
    {
        let pkg_info = app.package_info();
        let app_menu = Submenu::with_items(
            app,
            pkg_info.name.clone(),
            true,
            &[
                &PredefinedMenuItem::about(app, None, None)?,
                &PredefinedMenuItem::separator(app)?,
                &PredefinedMenuItem::services(app, None)?,
                &PredefinedMenuItem::separator(app)?,
                &PredefinedMenuItem::hide(app, None)?,
                &PredefinedMenuItem::hide_others(app, None)?,
                &PredefinedMenuItem::separator(app)?,
                &PredefinedMenuItem::quit(app, None)?,
            ],
        )?;

        Menu::with_items(
            app,
            &[
                &app_menu,
                &file_menu,
                &go_menu,
                &view_menu,
                &window_menu,
                &help_menu,
            ],
        )
    }

    #[cfg(not(target_os = "macos"))]
    {
        Menu::with_items(
            app,
            &[&file_menu, &go_menu, &view_menu, &window_menu, &help_menu],
        )
    }
}

/// Forwards clicks on our custom menu items to the frontend as a
/// `menu-command` event carrying the action id, so the same JS handlers
/// used by the keyboard shortcuts (see `src/main.js`) run either way.
pub fn handle_menu_event<R: Runtime>(app: &AppHandle<R>, event: tauri::menu::MenuEvent) {
    let action = event.id().as_ref();
    if !FORWARDED_ACTIONS.contains(&action) {
        return;
    }

    if let Some(window) = app.get_webview_window("main") {
        let _ = window.emit("menu-command", action);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn forwarded_actions_have_no_duplicates() {
        let mut sorted = FORWARDED_ACTIONS.to_vec();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(sorted.len(), FORWARDED_ACTIONS.len());
    }

    #[test]
    fn forwarded_actions_are_kebab_case() {
        for action in FORWARDED_ACTIONS {
            assert!(
                action.chars().all(|c| c.is_ascii_lowercase() || c == '-'),
                "action id `{action}` should be kebab-case"
            );
        }
    }
}
