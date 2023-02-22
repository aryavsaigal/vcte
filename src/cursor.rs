use std::io::Stdout;

use crossterm::{
    cursor,
    event::KeyCode,
    queue,
    Result
};

pub struct Cursor {
    pub x: u16,
    pub y: u16,
    pub movable: bool
}

impl Cursor {
    pub fn new() -> Self {
        Self {
            x: 0,
            y: 0,
            movable: true
        }
    }

    fn update_coords(&mut self, coords: (u16, u16)) {
        self.x = coords.0;
        self.y = coords.1;
    }

    pub fn move_cursor(&mut self, direction: KeyCode, renderer: &mut Stdout) -> Result<()> {
        if self.movable {
            match direction {
                KeyCode::Up | KeyCode::Char('w') => {
                    queue!(renderer, cursor::MoveUp(1))?;
                },
                KeyCode::Down | KeyCode::Char('s')=> {
                    queue!(renderer, cursor::MoveDown(1))?;
                },
                KeyCode::Left | KeyCode::Char('a') => {
                    queue!(renderer, cursor::MoveLeft(1))?;
                },
                KeyCode::Right | KeyCode::Char('d') => {
                    queue!(renderer, cursor::MoveRight(1))?;
                },
                _ => {}
            }
            self.update_coords(cursor::position()?);
        }
        Ok(())
    }
}