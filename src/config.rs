#[macro_use]
use lazy_static::lazy_static;
use tui::style::Color;

pub struct Theme {
    pub background: Color,
    pub foreground: Color,
}

lazy_static! {
    pub static ref THEME: Theme = Theme {
        background: Color::Rgb(40, 40, 40),
        foreground: Color::Rgb(253, 244, 193),
    };
}
