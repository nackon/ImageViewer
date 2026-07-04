mod app;
mod file_manager;
mod image_cache;
mod image_loader;
mod ui;
mod zoom;

#[cfg(target_os = "macos")]
mod macos_integration; 

use app::ImageViewer;

fn main() -> iced::Result {
    #[cfg(target_os = "macos")]
    {
        // winitの起動を先回りしてOSにDelegateを叩き込む
        macos_integration::macos_integration::pre_init_macos_listener();
    }

    iced::application(ImageViewer::title, ImageViewer::update, ImageViewer::view)
        .subscription(ImageViewer::subscription)
        .window_size((1200.0, 800.0))
        .run_with(ImageViewer::new)
}
