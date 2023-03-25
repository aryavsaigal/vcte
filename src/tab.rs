use crossterm::{style::Color, terminal};

use crate::{editor::File, colour_string::{ColourString, Info}};

pub struct Tab;

impl Tab {
    pub fn new() -> Self {
        Self {}
    }

    pub fn render(&mut self, files: &Vec<File>, file_index: usize) -> Vec<ColourString> {
        let terminal_x = terminal::size().unwrap().0;
        let mut tabs = Vec::new(); 

        for (i, file) in files.iter().enumerate() {
            let tab = ColourString::new(format!(" {} ", file.name.clone()), if i == file_index { Some(Info::new(Color::White, Color::DarkGrey, vec![])) } else { Some(Info::new(Color::White, Color::Black, vec![])) });
            tabs.push(tab);
        }

        let mut frame = ColourString::join(tabs, ColourString::new("▕".to_string(), Some(Info::new(Color::Black, Color::Reset, vec![]))));

        frame.truncate(terminal_x as usize);

        if frame.get_content().len() < terminal_x as usize {
            frame.push_str(" ".repeat((terminal_x as usize) - frame.get_content().len()).as_str(), None);
        }

        vec![frame]
    }
}