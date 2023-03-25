use crossterm::{terminal};

use crate::colour_string::{ColourString};

pub struct StatusBar {
    pub message: ColourString,
    pub command_output: Option<ColourString>
}

impl StatusBar {
    pub fn new() -> Self {
        Self {
            message: ColourString::new(String::from(""), None),
            command_output: None
        }
    }

    pub fn set_message(&mut self, message: ColourString) {
        self.message = message;
    }

    pub fn set_command_output(&mut self, command_output: ColourString) {
        self.command_output = Some(command_output);
    }

    pub fn render(&self) -> Vec<ColourString> {
        let (terminal_x, terminal_y) = terminal::size().unwrap();
        let mut frame: Vec<ColourString> = vec![ColourString::new(String::from(""), None); terminal_y as usize];

        frame.last_mut().unwrap().push_colour_string(self.command_output.clone().unwrap_or(self.message.clone()));
        frame.last_mut().unwrap().truncate(terminal_x as usize);
        frame
    }
}