use std::path::Path;

use crossterm::{terminal, style::Color};

use crate::{colour_string::{ColourString, Info}, cursor::Cursor, editor::File};

pub struct FileExplorer {
    pub enabled: bool,
    pub selected: bool,
    pub cursor: Cursor
}

impl FileExplorer {
    pub fn new() -> Self {
        let mut new = Self {
            enabled: false,
            selected: false,
            cursor: Cursor::new()
        };
        new.cursor.set_min(0, 1);
        new.cursor.y = 1;
        new
    }

    pub fn render(&mut self, file: &File) -> Vec<ColourString> {
        let (terminal_x, terminal_y) = terminal::size().unwrap();

        let max_x = terminal_x / 5;
        let mut frame: Vec<ColourString> = vec![ColourString::new(String::from(format!("{}▕", " ".repeat(max_x as usize))), None); terminal_y as usize];

        for (i, child) in Path::new(&file.path).parent().unwrap().read_dir().unwrap().enumerate() {
            if i > frame.len()-1 {
                break;
            }
            if let Ok(child) = child {
                debug!("child: {:?}", child);
                let mut name = child.file_name().into_string().unwrap();
                if child.path().is_dir() {
                    name.push('/');
                }
                name.truncate(max_x as usize);
                frame[i].replace_range(0, name.len(), ColourString::new(name, Some(Info::new(Color::White, Color::Reset, vec![]))));
            }
        } // change

        self.cursor.set_max(max_x, terminal_y-2);
        frame
    }
}

