/*
This file is part of Noteworthy.

Noteworthy is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License
as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

Noteworthy is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty
of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

You should have received a copy of the GNU General Public License along with Noteworthy. If not, see <https://www.gnu.org/licenses/>.
*/

use std::collections::HashMap;

// #[macro_use]
use lazy_static::lazy_static;
use tui::style::Color;

// use crate::{intern_error, main};

pub struct Theme {
    pub background: Color,
    pub foreground: Color,
    pub success: Color,
    pub alert: Color,
    pub highlight: Color,
}

lazy_static! {
    pub static ref THEME: Theme = Theme {
        background: Color::Black,
        foreground: Color::White,
        success: Color::Green,
        alert: Color::Red,
        highlight: Color::Yellow,
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
