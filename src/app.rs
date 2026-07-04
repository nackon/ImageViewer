use iced::widget::{column, container, image, row, text, Image};
use iced::{Alignment, Color, Element, Length, Task};
use std::path::PathBuf;

use crate::file_manager::FileManager;
use crate::image_cache::ImageCache;
use crate::image_loader::ImageData;
use crate::ui::theme::Theme;
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
    Quit,
}

impl ImageViewer {
    pub fn new() -> (Self, Task<Message>) {
        let args: Vec<String> = std::env::args().collect();

        let viewer = Self {
            file_manager: FileManager::new(),
            image_cache: ImageCache::new(15),
            current_image: None,
            zoom_level: ZoomLevel::Fit,
            window_width: 1200.0,
            window_height: 800.0,
            error_message: None,
        };

        if args.len() > 1 {
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
            Message::FileOpened(path) => {
                if let Err(e) = self.file_manager.load_directory(&path) {
                    self.error_message = Some(format!("Failed to load directory: {}", e));
                    return Task::none();
                }

                if let Some(current_path) = self.file_manager.get_current() {
                    let path = current_path.clone();
                    return self.load_image(path);
                }

                Task::none()
            }
            Message::ImageLoaded(result) => {
                match result {
                    Ok(image_data) => {
                        self.image_cache.put(image_data.path.clone(), image_data.clone());
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
                    let path = path.clone();
                    self.load_image(path)
                } else {
                    Task::none()
                }
            }
            Message::PreviousImage => {
                if let Some(path) = self.file_manager.previous() {
                    let path = path.clone();
                    self.load_image(path)
                } else {
                    Task::none()
                }
            }
            Message::FirstImage => {
                if let Some(path) = self.file_manager.first() {
                    let path = path.clone();
                    self.load_image(path)
                } else {
                    Task::none()
                }
            }
            Message::LastImage => {
                if let Some(path) = self.file_manager.last() {
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
            Message::Quit => iced::exit(),
        }
    }

    pub fn view(&self) -> Element<Message> {
        let content = if let Some(img) = &self.current_image {
            self.view_image(img)
        } else if let Some(error) = &self.error_message {
            self.view_error(error)
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

    fn view_image(&self, img: &ImageData) -> Element<Message> {
        let header = self.create_header(img);
        let footer = self.create_footer();

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

        column![header, image_container, footer]
            .width(Length::Fill)
            .height(Length::Fill)
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

    fn create_header(&self, img: &ImageData) -> Element<Message> {
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
            text(" | [T] Thumbnails").size(12).color(Theme::TEXT),
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
                let available_height = self.window_height - 60.0;
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
                        thread_priority::ThreadPriority::Max
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
                    "f" | "F" => Some(Message::ZoomFit),
                    _ => None,
                }
            } else if let iced::Event::Window(iced::window::Event::Resized(size)) =
                event
            {
                Some(Message::WindowResized(size.width, size.height))
            } else {
                None
            }
        })
    }
}
