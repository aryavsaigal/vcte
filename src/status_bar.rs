use crossterm::{terminal, style::Color};

use crate::colour_string::{ColourString, Info};

pub struct StatusBar {
    pub message: ColourString,

}

impl StatusBar {
    pub fn new() -> Self {
        Self {
            message: ColourString::new(String::from(""), None),
        }
    }

    pub fn set_message(&mut self, message: ColourString) {
        self.message = message;
    }

    pub fn render(&self) -> Vec<ColourString> {
        let (terminal_x, terminal_y) = terminal::size().unwrap();
        let mut frame: Vec<ColourString> = vec![ColourString::new(String::from(""), None); terminal_y as usize];

        frame.last_mut().unwrap().push_colour_string(self.message.clone());

        frame
    }
}