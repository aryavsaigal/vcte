use crossterm::{terminal, style::Color};
use unicode_segmentation::UnicodeSegmentation;

use crate::{colour_string::{ColourString, Info}, cursor::Cursor};

pub struct CommandPalette {
    pub command: String,
    pub enabled: bool,
    pub cursor: Cursor
}

impl CommandPalette {
    pub fn new() -> Self {
        Self {
            command: String::new(),
            enabled: false,
            cursor: Cursor::new()
        }
    }

    pub fn render(&mut self) -> Vec<ColourString> {
        let (terminal_x, terminal_y) = terminal::size().unwrap();
        let mut frame: Vec<ColourString> = vec![ColourString::new(String::from(" ".repeat(terminal_x as usize)), None); terminal_y as usize];

        let command = format!("▏:{}▕", self.command);
        let command_len = command.graphemes(true).count();
        let start = if terminal_y / 8 > 1 { terminal_y / 8 } else { 2 };
        let padding = ((terminal_x - command.len() as u16)/2) as usize;

        let border_colour = Info::new(Color::White, Color::Reset, vec![]);
        let mut command = ColourString::new(command, Some(Info::new(Color::White, Color::Reset, vec![])));

        command.set_colour(border_colour.clone(), 0, 1);
        command.set_colour(border_colour.clone(), command_len-1, command_len);
        // command.replace_char(" ".to_string(), "█".to_string(), Some(Info::new(Color::Black, Color::Reset, vec![])));

        frame[(start-1) as usize].replace_range(padding, terminal_x as usize, ColourString::new("▁".repeat(command_len), Some(border_colour.clone())));
        frame[start as usize].replace_range(padding, terminal_x as usize, command);
        frame[(start+1) as usize].replace_range(padding, terminal_x as usize, ColourString::new("▔".repeat(command_len), Some(border_colour.clone())));

        self.cursor.update((padding + self.command.graphemes(true).count() + 2) as u16, start);

        frame
    }
}