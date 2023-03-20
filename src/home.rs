use crossterm::{
    terminal,
    Result
};

use crate::{colour_string::{ColourString}, cursor::Cursor};

pub struct Home {
    pub cursor: Cursor,
}

impl Home {
    pub fn new() -> Self {
        Self {
            cursor: Cursor::new(),
        }
    }

    pub fn render(&mut self) -> Result<Vec<ColourString>> {
        let (terminal_x, terminal_y) = terminal::size()?;
        let mut output = Vec::new();
    
        for i in 0..terminal_y {
            let mut line = ColourString::new("~".to_string(), None);
    
            if i == terminal_y / 3 {
                let mut welcome = format!("vcte (very cool text editor) v{}", env!("CARGO_PKG_VERSION"));
                let padding = ((terminal_x - welcome.len() as u16) / 2) - 1;
    
                welcome.truncate(terminal_x as usize);
                line.push_str(&format!("{}{}", &" ".repeat(padding as usize), welcome), None);
            }
            output.push(line);
        };
        Ok(output)
    }
}

