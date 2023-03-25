use crossterm::{terminal, style::Color};

use crate::{colour_string::{ColourString, Info}, cursor::Cursor};

pub struct FileExplorer {
    pub enabled: bool,
    pub selected: bool,
    pub cursor: Cursor
}

impl FileExplorer {
    pub fn new() -> Self {
        Self {
            enabled: false,
            selected: false,
            cursor: Cursor::new()
        }
    }

    pub fn render(&mut self) -> Vec<ColourString> {
        let (terminal_x, terminal_y) = terminal::size().unwrap();
        let mut frame: Vec<ColourString> = vec![ColourString::new(String::from(" ".repeat(terminal_x as usize)), None); terminal_y as usize];
        let max_x = terminal_x / 5;

        for line in frame.iter_mut() {
            let mut new_line = ColourString::new(" ".repeat((max_x) as usize), Some(Info::new(Color::Reset, Color::Reset, vec![])));
            new_line.push_str("▕", None);
            line.replace_range(0, terminal_x as usize, new_line);
        }
        self.cursor.set_max(max_x, terminal_y-2);
        frame
    }
}

