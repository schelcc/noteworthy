use std::collections::HashMap;

#[macro_use]
use lazy_static::lazy_static;
use tui::style::Color;

use crate::{intern_error, main};

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
    pub static ref SETTINGS: HashMap<String, String> = {
        let mut output: HashMap<String, String> = HashMap::new();

        output.insert(String::from("default-local-dir"), String::from("/home/"));
        output.insert(String::from("default-remote-dir"), String::from("root"));

        output.insert(String::from("host"), String::from("192.168.2.39:22"));
        output.insert(String::from("username"), String::from("root"));
        output.insert(String::from("password"), String::from("vFQqgk7Ngd"));

        output
    };
}
