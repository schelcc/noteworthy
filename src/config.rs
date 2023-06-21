#[macro_use]
use lazy_static::lazy_static;
use tui::style::Color;

pub struct Theme {
    pub background: Color,
    pub foreground: Color,
}

lazy_static! {
    pub static ref THEME: Theme = Theme {
        background: Color::Green,
        foreground: Color::White,
    };
}
