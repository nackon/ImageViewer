mod app;
mod file_manager;
mod image_cache;
mod image_loader;
mod ui;
mod zoom;

use app::ImageViewer;

fn main() -> iced::Result {
    iced::application(ImageViewer::title, ImageViewer::update, ImageViewer::view)
        .subscription(ImageViewer::subscription)
        .window_size((1200.0, 800.0))
        .run_with(ImageViewer::new)
}
