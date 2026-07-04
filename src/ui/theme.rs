use iced::Color;

pub struct Theme;

impl Theme {
    pub const BACKGROUND: Color = Color::from_rgb(0.169, 0.169, 0.169); // #2b2b2b
    pub const TEXT: Color = Color::from_rgb(0.8, 0.8, 0.8); // #cccccc
    pub const ACCENT: Color = Color::from_rgb(0.29, 0.62, 1.0); // #4a9eff
    pub const HEADER_BG: Color = Color::from_rgb(0.15, 0.15, 0.15); // #262626
}
