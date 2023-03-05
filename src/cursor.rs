use std::io::Stdout;

use crossterm::{
    cursor,
    event::KeyCode,
    queue,
    Result,
    terminal
};

pub struct Cursor {
    pub x: u16,
    pub y: u16,
    pub offset_x: u16,
    pub offset_y: u16,
    pub movable: bool
}

impl Cursor {
    pub fn new() -> Self {
        Self {
            x: 0,
            y: 0,
            offset_x: 0,
            offset_y: 0,
            movable: true
        }
    }

    pub fn update_coords(&mut self, coords: (u16, u16)) {
        self.x = coords.0;
        self.y = coords.1;
    }

    pub fn move_to(&mut self, x: u16, y: u16, renderer: &mut Stdout) -> Result<()> {
        queue!(renderer, cursor::MoveTo(x, y))?;
        self.update_coords((x, y));
        Ok(())
    }

    pub fn clear_offset(&mut self) {
        self.offset_x = 0;
        self.offset_y = 0;
    }

    pub fn move_cursor(&mut self, direction: KeyCode, renderer: &mut Stdout) -> Result<()> {
        if self.movable {
            let (terminal_x, terminal_y) = terminal::size()?;
            match direction {
                KeyCode::Up | KeyCode::Char('w') => {
                    if self.y == 0 {
                        self.offset_y = self.offset_y.saturating_sub(1);
                    } 
                    else {
                        queue!(renderer, cursor::MoveUp(1))?;
                    }
                },
                KeyCode::Down | KeyCode::Char('s')=> {
                    if self.y == terminal_y-2 {
                        self.offset_y += 1;
                    } 
                    else {
                        queue!(renderer, cursor::MoveDown(1))?;
                    }
                },
                KeyCode::Left | KeyCode::Char('a') => {
                    if self.x == 0 {
                        self.offset_x = self.offset_x.saturating_sub(1)
                    } 
                    else {
                        queue!(renderer, cursor::MoveLeft(1))?;
                    }
                },
                KeyCode::Right | KeyCode::Char('d') => {
                    if self.x == terminal_x-1 {
                        self.offset_x += 1;
                    } 
                    else {
                        queue!(renderer, cursor::MoveRight(1))?;
                    }
                },
                _ => {}
            }
            self.update_coords(cursor::position()?);
        }
        Ok(())
    }
}