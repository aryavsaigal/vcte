use crate::colour_string::{ColourString, Info};
use crate::cursor::Cursor;
use crossterm::{Result, terminal};
use crossterm::style::Color;
use std::fs;
use std::path::Path;

pub struct File {
    pub path: String,
    pub name: String,
    pub lines: Vec<String>,
    pub cursor: Cursor,
    pub modified: bool,
}

impl File {
    pub fn new(path: String) -> Result<Self> {
        let path = Path::new(&path);
        let name = path.file_name().unwrap().to_str().unwrap().to_string();
        let lines = fs::read_to_string(path)?.lines().map(|s| s.to_string()).collect();
        
        Ok(Self {
            path: path.to_str().unwrap().to_string(),
            name,
            lines,
            cursor: Cursor::new(),
            modified: false,
        })
    }

    pub fn render(&mut self) -> Vec<ColourString> {
        let (terminal_x, terminal_y) = terminal::size().unwrap();
        let mut frame: Vec<ColourString> = vec![ColourString::new(String::from(" ".repeat(terminal_x as usize)), None); terminal_y as usize];
        let default = String::new();

        let mut line_number = 1;
        for i in 0..terminal_y {
            let line = self.lines.get(i as usize).unwrap_or(&default);
            let mut colour_line = ColourString::new(line_number.to_string(), Some(Info::new(Color::DarkGrey, Color::Reset, vec![])));
            colour_line.insert(0, "█".repeat(4-line_number.to_string().len()), Some(Info::new(Color::Black, Color::Reset, vec![])));
            colour_line.push_str(&format!(" {}", line), None);
            colour_line.truncate(terminal_x as usize);

            frame[i as usize].replace_range(0, terminal_x as usize, colour_line);
            line_number += 1;
        }
        frame
    }
}