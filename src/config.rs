#[macro_use]
use lazy_static::lazy_static;
use tui::style::Color;

use crate::intern_error;

pub struct Theme {
    pub background: Color,
    pub foreground: Color,
    pub success: Color,
    pub alert: Color,
}

lazy_static! {
    pub static ref THEME: Theme = Theme {
        background: Color::Black,
        foreground: Color::White,
        success: Color::Green,
        alert: Color::Red,
    };
}
