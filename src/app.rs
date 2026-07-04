use iced::widget::{column, container, image, row, text, Image};
use iced::window::Mode;
use iced::{window, Alignment, Color, Element, Length, Task};
use std::path::PathBuf;

use crate::file_manager::FileManager;
use crate::image_cache::ImageCache;
use crate::image_loader::ImageData;
use crate::ui::theme::Theme;
use crate::ui::thumbnail::{NavigationDirection, ThumbnailGrid, ThumbnailMessage};
use crate::zoom::ZoomCalculator;

#[derive(Debug, Clone)]
pub enum ZoomLevel {
    Fit,
    Percentage(f32),
}

pub struct ImageViewer {
    file_manager: FileManager,
    image_cache: ImageCache,
    current_image: Option<ImageData>,
    zoom_level: ZoomLevel,
    window_width: f32,
    window_height: f32,
    error_message: Option<String>,
    show_thumbnails: bool,
    thumbnail_grid: ThumbnailGrid,
    is_fullscreen: bool,
    is_loading: bool,
}

#[derive(Debug, Clone)]
pub enum Message {
    FileOpened(PathBuf),
    ImageLoaded(Result<ImageData, String>),
    NextImage,
    PreviousImage,
    FirstImage,
    LastImage,
    ZoomIn,
    ZoomOut,
    ZoomFit,
    ZoomActualSize,
    WindowResized(f32, f32),
    ToggleThumbnails,
    ThumbnailAction(ThumbnailMessage),
    ThumbnailNavigate(NavigationDirection),
    SelectThumbnail,
    ToggleFullscreen,
    Quit,
    FileOpenedFromMac(std::path::PathBuf), 
}

impl ImageViewer {
    pub fn new() -> (Self, Task<Message>) {
        let args: Vec<String> = std::env::args().collect();

        let has_file_arg = args.len() > 1;

        let viewer = Self {
            file_manager: FileManager::new(),
            image_cache: ImageCache::new(15),
            current_image: None,
            zoom_level: ZoomLevel::Fit,
            window_width: 1200.0,
            window_height: 800.0,
            error_message: None,
            show_thumbnails: false,
            thumbnail_grid: ThumbnailGrid::new(),
            is_fullscreen: false,
            is_loading: has_file_arg,
        };

        if has_file_arg {
            let path = PathBuf::from(&args[1]);
            (viewer, Task::done(Message::FileOpened(path)))
        } else {
            (viewer, Task::none())
        }
    }

    pub fn title(&self) -> String {
        if let Some(img) = &self.current_image {
            format!("Image Viewer - {}", img.file_name())
        } else {
            "Image Viewer".to_string()
        }
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::FileOpened(path) | Message::FileOpenedFromMac(path) => {
                self.is_loading = true;
                if let Err(e) = self.file_manager.load_directory(&path) {
                    self.error_message = Some(format!("Failed to load directory: {}", e));
                    self.is_loading = false;
                    return Task::none();
                }

                if let Some(current_path) = self.file_manager.get_current() {
                    let path = current_path.clone();
                    return self.load_image(path);
                }

                self.is_loading = false;
                Task::none()
            }
            Message::ImageLoaded(result) => {
                self.is_loading = false;
                match result {
                    Ok(image_data) => {
                        self.image_cache
                            .put(image_data.path.clone(), image_data.clone());
                        self.current_image = Some(image_data);
                        self.error_message = None;
                        self.prefetch_adjacent_images();
                    }
                    Err(e) => {
                        self.error_message = Some(format!("Failed to load image: {}", e));
                    }
                }
                Task::none()
            }
            Message::NextImage => {
                if let Some(path) = self.file_manager.next() {
                    self.is_loading = true;
                    let path = path.clone();
                    self.load_image(path)
                } else {
                    Task::none()
                }
            }
            Message::PreviousImage => {
                if let Some(path) = self.file_manager.previous() {
                    self.is_loading = true;
                    let path = path.clone();
                    self.load_image(path)
                } else {
                    Task::none()
                }
            }
            Message::FirstImage => {
                if let Some(path) = self.file_manager.first() {
                    self.is_loading = true;
                    let path = path.clone();
                    self.load_image(path)
                } else {
                    Task::none()
                }
            }
            Message::LastImage => {
                if let Some(path) = self.file_manager.last() {
                    self.is_loading = true;
                    let path = path.clone();
                    self.load_image(path)
                } else {
                    Task::none()
                }
            }
            Message::ZoomIn => {
                if let ZoomLevel::Percentage(level) = self.zoom_level {
                    self.zoom_level = ZoomLevel::Percentage(ZoomCalculator::zoom_in(level));
                }
                Task::none()
            }
            Message::ZoomOut => {
                if let ZoomLevel::Percentage(level) = self.zoom_level {
                    self.zoom_level = ZoomLevel::Percentage(ZoomCalculator::zoom_out(level));
                }
                Task::none()
            }
            Message::ZoomFit => {
                self.zoom_level = ZoomLevel::Fit;
                Task::none()
            }
            Message::ZoomActualSize => {
                self.zoom_level = ZoomLevel::Percentage(1.0);
                Task::none()
            }
            Message::WindowResized(width, height) => {
                self.window_width = width;
                self.window_height = height;
                Task::none()
            }
            Message::ToggleThumbnails => {
                self.show_thumbnails = !self.show_thumbnails;
                if self.show_thumbnails {
                    let files = self.file_manager.get_all_files();
                    let current_index = self.file_manager.current_index();
                    self.thumbnail_grid.load_files(files, current_index);
                }
                Task::none()
            }
            Message::ThumbnailAction(thumbnail_msg) => match thumbnail_msg {
                ThumbnailMessage::SelectThumbnail(index) => {
                    if let Some(path) = self.file_manager.jump_to(index) {
                        self.is_loading = true;
                        let path = path.clone();
                        self.show_thumbnails = false;
                        self.load_image(path)
                    } else {
                        Task::none()
                    }
                }
                ThumbnailMessage::Close => {
                    self.show_thumbnails = false;
                    Task::none()
                }
            },
            Message::ThumbnailNavigate(direction) => {
                self.thumbnail_grid.move_selection(direction);
                Task::none()
            }
            Message::SelectThumbnail => {
                let selected_index = self.thumbnail_grid.selected_index();
                if let Some(path) = self.file_manager.jump_to(selected_index) {
                    self.is_loading = true;
                    let path = path.clone();
                    self.show_thumbnails = false;
                    self.load_image(path)
                } else {
                    Task::none()
                }
            }
            Message::ToggleFullscreen => {
                self.is_fullscreen = !self.is_fullscreen;
                let mode = if self.is_fullscreen {
                    Mode::Fullscreen
                } else {
                    Mode::Windowed
                };
                window::get_latest().and_then(move |id| window::change_mode(id, mode))
            }
            Message::Quit => iced::exit(),
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        if self.show_thumbnails {
            self.thumbnail_grid.view().map(Message::ThumbnailAction)
        } else {
            let content = if let Some(img) = &self.current_image {
                self.view_image(img)
            } else if let Some(error) = &self.error_message {
                self.view_error(error)
            } else if self.is_loading {
                self.view_loading()
            } else {
                self.view_empty()
            };

            container(content)
                .width(Length::Fill)
                .height(Length::Fill)
                .style(|_theme| container::Style {
                    background: Some(iced::Background::Color(Theme::BACKGROUND)),
                    ..Default::default()
                })
                .into()
        }
    }

    fn view_image(&self, img: &ImageData) -> Element<'_, Message> {
        let scale = self.calculate_scale(img);
        let (display_width, display_height) =
            ZoomCalculator::calculate_scaled_dimensions(img.width, img.height, scale);

        let image_widget: Image<iced::widget::image::Handle> = image(img.handle.clone())
            .width(Length::Fixed(display_width as f32))
            .height(Length::Fixed(display_height as f32));

        let image_container = container(image_widget)
            .width(Length::Fill)
            .height(Length::Fill)
            .center(Length::Fill);

        if self.is_fullscreen {
            image_container.into()
        } else {
            let header = self.create_header(img);
            let footer = self.create_footer();

            column![header, image_container, footer]
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
        }
    }

    fn view_loading(&self) -> Element<'static, Message> {
        container(text("Loading...").size(24).color(Theme::TEXT))
            .width(Length::Fill)
            .height(Length::Fill)
            .center(Length::Fill)
            .into()
    }

    fn view_empty(&self) -> Element<'static, Message> {
        container(
            text("No image loaded. Please open an image file.")
                .size(24)
                .color(Theme::TEXT),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center(Length::Fill)
        .into()
    }

    fn view_error<'a>(&self, error: &'a str) -> Element<'a, Message> {
        container(
            column![
                text("Error").size(32).color(Color::from_rgb(1.0, 0.3, 0.3)),
                text(error).size(16).color(Theme::TEXT),
            ]
            .spacing(10)
            .align_x(Alignment::Center),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center(Length::Fill)
        .into()
    }

    fn create_header(&self, img: &ImageData) -> Element<'_, Message> {
        let info = format!(
            "{} - {}x{} - {}",
            img.file_name(),
            img.width,
            img.height,
            img.file_size_string()
        );

        container(text(info).size(14).color(Theme::TEXT))
            .padding(10)
            .width(Length::Fill)
            .style(|_theme| container::Style {
                background: Some(iced::Background::Color(Theme::HEADER_BG)),
                ..Default::default()
            })
            .into()
    }

    fn create_footer(&self) -> Element<'static, Message> {
        let position = format!(
            "{}/{}",
            self.file_manager.current_index() + 1,
            self.file_manager.total_count()
        );

        let zoom_text = match &self.zoom_level {
            ZoomLevel::Fit => "Fit".to_string(),
            ZoomLevel::Percentage(level) => format!("{:.0}%", level * 100.0),
        };

        let info = row![
            text(position).size(12).color(Theme::TEXT),
            text(" | Zoom: ").size(12).color(Theme::TEXT),
            text(zoom_text).size(12).color(Theme::ACCENT),
            text(" | [T] Thumbnails | [F] Fullscreen")
                .size(12)
                .color(Theme::TEXT),
        ]
        .spacing(5);

        container(info)
            .padding(10)
            .width(Length::Fill)
            .style(|_theme| container::Style {
                background: Some(iced::Background::Color(Theme::HEADER_BG)),
                ..Default::default()
            })
            .into()
    }

    fn calculate_scale(&self, img: &ImageData) -> f32 {
        match &self.zoom_level {
            ZoomLevel::Fit => {
                let available_height = if self.is_fullscreen {
                    self.window_height
                } else {
                    self.window_height - 60.0
                };
                ZoomCalculator::calculate_fit_scale(
                    img.width,
                    img.height,
                    self.window_width as u32,
                    available_height as u32,
                )
            }
            ZoomLevel::Percentage(level) => *level,
        }
    }

    fn load_image(&self, path: PathBuf) -> Task<Message> {
        if let Some(cached_image) = self.image_cache.get(&path) {
            return Task::done(Message::ImageLoaded(Ok(cached_image)));
        }

        Task::perform(
            async move {
                tokio::task::spawn_blocking(move || {
                    let _ = thread_priority::set_current_thread_priority(
                        thread_priority::ThreadPriority::Max,
                    );
                    ImageData::load(&path)
                })
                .await
                .unwrap_or_else(|e| Err(anyhow::anyhow!("Join error: {}", e)))
                .map_err(|e| e.to_string())
            },
            Message::ImageLoaded,
        )
    }

    fn prefetch_adjacent_images(&self) {
        let mut paths_to_prefetch = Vec::new();

        for i in 1..=5 {
            if let Some(path) = self.file_manager.peek_next(i) {
                paths_to_prefetch.push(path.clone());
            }
        }

        for i in 1..=2 {
            if let Some(path) = self.file_manager.peek_previous(i) {
                paths_to_prefetch.push(path.clone());
            }
        }

        if !paths_to_prefetch.is_empty() {
            self.image_cache.prefetch(paths_to_prefetch);
        }
    }

pub fn subscription(&self) -> iced::Subscription<Message> {
    use iced::keyboard;
    use iced::keyboard::key::Named;

    let sub = if self.show_thumbnails {
        iced::event::listen_with(|event, _status, _id| {
            if let iced::Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(named_key),
                ..
            }) = event
            {
                match named_key {
                    Named::ArrowUp => Some(Message::ThumbnailNavigate(NavigationDirection::Up)),
                    Named::ArrowDown => {
                        Some(Message::ThumbnailNavigate(NavigationDirection::Down))
                    }
                    Named::ArrowLeft => {
                        Some(Message::ThumbnailNavigate(NavigationDirection::Left))
                    }
                    Named::ArrowRight => {
                        Some(Message::ThumbnailNavigate(NavigationDirection::Right))
                    }
                    Named::Enter => Some(Message::SelectThumbnail),
                    Named::Escape => Some(Message::ThumbnailAction(ThumbnailMessage::Close)),
                    _ => None,
                }
            } else if let iced::Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Character(c),
                ..
            }) = event
            {
                match c.as_str() {
                    "t" | "T" => Some(Message::ToggleThumbnails),
                    _ => None,
                }
            } else if let iced::Event::Window(iced::window::Event::Resized(size)) = event {
                Some(Message::WindowResized(size.width, size.height))
            } else {
                None
            }
        })
    } else {
        iced::event::listen_with(|event, _status, _id| {
            if let iced::Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(named_key),
                ..
            }) = event
            {
                match named_key {
                    Named::ArrowRight | Named::Space => Some(Message::NextImage),
                    Named::ArrowLeft | Named::Backspace => Some(Message::PreviousImage),
                    Named::Home => Some(Message::FirstImage),
                    Named::End => Some(Message::LastImage),
                    Named::Escape => Some(Message::Quit),
                    _ => None,
                }
            } else if let iced::Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Character(c),
                ..
            }) = event
            {
                match c.as_str() {
                    "n" | "N" => Some(Message::NextImage),
                    "p" | "P" => Some(Message::PreviousImage),
                    "q" | "Q" => Some(Message::Quit),
                    "+" | "=" => Some(Message::ZoomIn),
                    "-" => Some(Message::ZoomOut),
                    "0" => Some(Message::ZoomActualSize),
                    "w" | "W" => Some(Message::ZoomFit),
                    "f" | "F" => Some(Message::ToggleFullscreen),
                    "t" | "T" => Some(Message::ToggleThumbnails),
                    _ => None,
                }
            } else if let iced::Event::Window(iced::window::Event::Resized(size)) = event {
                Some(Message::WindowResized(size.width, size.height))
            } else {
                None
            }
        })
    };

    // ★ iced 0.13 に適合する結合方式（Subscription::batch）に修正
    #[cfg(target_os = "macos")]
    {
        iced::Subscription::batch(vec![
            sub,
            iced::Subscription::run(crate::macos_integration::macos_integration::listen_open_file_events)
                .map(Message::FileOpenedFromMac),
        ])
    }
    #[cfg(not(target_os = "macos"))]
    {
        sub
    }
}

}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use tempfile::TempDir;

    fn create_test_image(dir: &std::path::Path, name: &str) -> PathBuf {
        let path = dir.join(name);
        File::create(&path).unwrap();
        path
    }

    #[test]
    fn test_file_opened_from_mac_loads_image() {
        let temp_dir = TempDir::new().unwrap();
        let image_path = create_test_image(temp_dir.path(), "test.jpg");

        let mut viewer = ImageViewer {
            file_manager: FileManager::new(),
            image_cache: ImageCache::new(15),
            current_image: None,
            zoom_level: ZoomLevel::Fit,
            window_width: 1200.0,
            window_height: 800.0,
            error_message: None,
            show_thumbnails: false,
            thumbnail_grid: ThumbnailGrid::new(),
            is_fullscreen: false,
            is_loading: false,
        };

        // FileOpenedFromMacメッセージを送信
        let _ = viewer.update(Message::FileOpenedFromMac(image_path.clone()));

        // ファイルマネージャーにファイルがロードされたことを確認
        assert!(viewer.is_loading);
        assert_eq!(viewer.file_manager.total_count(), 1);
        assert!(viewer.error_message.is_none());
    }

    #[test]
    fn test_file_opened_and_file_opened_from_mac_same_behavior() {
        let temp_dir = TempDir::new().unwrap();
        let image_path = create_test_image(temp_dir.path(), "test.jpg");

        // FileOpenedメッセージでのテスト
        let mut viewer1 = ImageViewer {
            file_manager: FileManager::new(),
            image_cache: ImageCache::new(15),
            current_image: None,
            zoom_level: ZoomLevel::Fit,
            window_width: 1200.0,
            window_height: 800.0,
            error_message: None,
            show_thumbnails: false,
            thumbnail_grid: ThumbnailGrid::new(),
            is_fullscreen: false,
            is_loading: false,
        };
        let _ = viewer1.update(Message::FileOpened(image_path.clone()));

        // FileOpenedFromMacメッセージでのテスト
        let mut viewer2 = ImageViewer {
            file_manager: FileManager::new(),
            image_cache: ImageCache::new(15),
            current_image: None,
            zoom_level: ZoomLevel::Fit,
            window_width: 1200.0,
            window_height: 800.0,
            error_message: None,
            show_thumbnails: false,
            thumbnail_grid: ThumbnailGrid::new(),
            is_fullscreen: false,
            is_loading: false,
        };
        let _ = viewer2.update(Message::FileOpenedFromMac(image_path.clone()));

        // 両方のビューアが同じ状態になることを確認
        assert_eq!(viewer1.is_loading, viewer2.is_loading);
        assert_eq!(viewer1.file_manager.total_count(), viewer2.file_manager.total_count());
        assert_eq!(viewer1.error_message.is_some(), viewer2.error_message.is_some());
    }
}
