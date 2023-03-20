use crossterm::{terminal, style::Color};

use crate::colour_string::{ColourString, Info};

pub struct FileExplorer {
    pub enabled: bool,
}

impl FileExplorer {
    pub fn new() -> Self {
        Self {
            enabled: false,
        }
    }

    pub fn render(&mut self) -> Vec<ColourString> {
        let (terminal_x, terminal_y) = terminal::size().unwrap();
        let mut frame: Vec<ColourString> = vec![ColourString::new(String::from(" ".repeat(terminal_x as usize)), None); terminal_y as usize];

        for line in frame.iter_mut() {
            let mut new_line = ColourString::new("█".repeat((terminal_x / 5) as usize), Some(Info::new(Color::Black, Color::Reset, vec![])));
            new_line.push_str("▕", None);
            line.replace_range(0, terminal_x as usize, new_line);
        }

        frame
    }
}

