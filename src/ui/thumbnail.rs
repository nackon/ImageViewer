use iced::widget::{button, column, container, image, row, scrollable, text, Image};
use iced::{Alignment, Element, Length};
use std::collections::HashMap;
use std::path::PathBuf;

use crate::ui::theme::Theme;

pub struct ThumbnailGrid {
    thumbnails: Vec<ThumbnailItem>,
    selected_index: usize,
    thumbnail_cache: HashMap<PathBuf, iced::widget::image::Handle>,
}

#[derive(Clone)]
pub struct ThumbnailItem {
    pub path: PathBuf,
    pub index: usize,
    pub file_name: String,
}

#[derive(Debug, Clone)]
pub enum ThumbnailMessage {
    SelectThumbnail(usize),
    Close,
}

impl ThumbnailGrid {
    pub fn new() -> Self {
        Self {
            thumbnails: Vec::new(),
            selected_index: 0,
            thumbnail_cache: HashMap::new(),
        }
    }

    pub fn load_files(&mut self, files: &[PathBuf], current_index: usize) {
        self.thumbnails = files
            .iter()
            .enumerate()
            .map(|(index, path)| ThumbnailItem {
                path: path.clone(),
                index,
                file_name: path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("Unknown")
                    .to_string(),
            })
            .collect();
        self.selected_index = current_index;
    }

    pub fn view(&self) -> Element<ThumbnailMessage> {
        let header = self.create_header();

        let grid = self.create_grid();

        let footer = self.create_footer();

        let content = column![header, grid, footer]
            .width(Length::Fill)
            .height(Length::Fill);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|_theme| container::Style {
                background: Some(iced::Background::Color(Theme::BACKGROUND)),
                ..Default::default()
            })
            .into()
    }

    fn create_header(&self) -> Element<ThumbnailMessage> {
        let title = format!("Thumbnails - {} images", self.thumbnails.len());

        container(text(title).size(16).color(Theme::TEXT))
            .padding(10)
            .width(Length::Fill)
            .style(|_theme| container::Style {
                background: Some(iced::Background::Color(Theme::HEADER_BG)),
                ..Default::default()
            })
            .into()
    }

    fn create_grid(&self) -> Element<ThumbnailMessage> {
        const THUMBNAIL_SIZE: f32 = 150.0;
        const SPACING: f32 = 10.0;
        const COLUMNS: usize = 5;

        let rows_data: Vec<Vec<&ThumbnailItem>> = self
            .thumbnails
            .chunks(COLUMNS)
            .map(|chunk| chunk.iter().collect())
            .collect();

        let mut grid_rows = Vec::new();

        for row_items in rows_data {
            let mut row_elements = Vec::new();

            for item in row_items {
                let is_selected = item.index == self.selected_index;

                let thumbnail_content = if let Some(handle) = self.thumbnail_cache.get(&item.path)
                {
                    let img: Image<iced::widget::image::Handle> = image(handle.clone())
                        .width(Length::Fixed(THUMBNAIL_SIZE))
                        .height(Length::Fixed(THUMBNAIL_SIZE));

                    container(img)
                        .width(Length::Fixed(THUMBNAIL_SIZE))
                        .height(Length::Fixed(THUMBNAIL_SIZE))
                        .center(THUMBNAIL_SIZE)
                } else {
                    let img: Image<iced::widget::image::Handle> =
                        image(iced::widget::image::Handle::from_path(&item.path))
                            .width(Length::Fixed(THUMBNAIL_SIZE))
                            .height(Length::Fixed(THUMBNAIL_SIZE));

                    container(img)
                        .width(Length::Fixed(THUMBNAIL_SIZE))
                        .height(Length::Fixed(THUMBNAIL_SIZE))
                        .center(THUMBNAIL_SIZE)
                };

                let label = text(&item.file_name)
                    .size(10)
                    .color(Theme::TEXT)
                    .width(Length::Fixed(THUMBNAIL_SIZE));

                let thumbnail_column = column![thumbnail_content, label]
                    .align_x(Alignment::Center)
                    .spacing(5);

                let border_width = if is_selected { 3.0 } else { 1.0 };
                let border_color = if is_selected {
                    Theme::ACCENT
                } else {
                    iced::Color::from_rgb(0.3, 0.3, 0.3)
                };

                let btn = button(thumbnail_column)
                    .on_press(ThumbnailMessage::SelectThumbnail(item.index))
                    .padding(5)
                    .style(move |_theme, status| {
                        let bg_color = match status {
                            button::Status::Hovered => {
                                iced::Color::from_rgb(0.25, 0.25, 0.25)
                            }
                            _ => iced::Color::from_rgb(0.23, 0.23, 0.23),
                        };

                        button::Style {
                            background: Some(iced::Background::Color(bg_color)),
                            border: iced::Border {
                                color: border_color,
                                width: border_width,
                                radius: 4.0.into(),
                            },
                            ..Default::default()
                        }
                    });

                row_elements.push(btn.into());
            }

            let row_widget = row(row_elements)
                .spacing(SPACING)
                .align_y(Alignment::Start);

            grid_rows.push(row_widget.into());
        }

        let grid_column = column(grid_rows).spacing(SPACING).padding(SPACING);

        scrollable(grid_column)
            .height(Length::Fill)
            .width(Length::Fill)
            .into()
    }

    fn create_footer(&self) -> Element<ThumbnailMessage> {
        let info = text("Use ↑↓←→ to navigate, Enter to view, T/Esc to close")
            .size(12)
            .color(Theme::TEXT);

        container(info)
            .padding(10)
            .width(Length::Fill)
            .style(|_theme| container::Style {
                background: Some(iced::Background::Color(Theme::HEADER_BG)),
                ..Default::default()
            })
            .into()
    }

    pub fn move_selection(&mut self, direction: NavigationDirection) {
        const COLUMNS: usize = 5;

        let total = self.thumbnails.len();
        if total == 0 {
            return;
        }

        match direction {
            NavigationDirection::Up => {
                if self.selected_index >= COLUMNS {
                    self.selected_index -= COLUMNS;
                }
            }
            NavigationDirection::Down => {
                if self.selected_index + COLUMNS < total {
                    self.selected_index += COLUMNS;
                }
            }
            NavigationDirection::Left => {
                if self.selected_index > 0 {
                    self.selected_index -= 1;
                }
            }
            NavigationDirection::Right => {
                if self.selected_index + 1 < total {
                    self.selected_index += 1;
                }
            }
        }
    }

    pub fn selected_index(&self) -> usize {
        self.selected_index
    }

    #[allow(dead_code)]
    pub fn add_to_cache(&mut self, path: PathBuf, handle: iced::widget::image::Handle) {
        self.thumbnail_cache.insert(path, handle);
    }
}

#[derive(Debug, Clone)]
pub enum NavigationDirection {
    Up,
    Down,
    Left,
    Right,
}
