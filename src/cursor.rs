use std::io::Stdout;

use crossterm::{self, queue, Result, event::KeyCode, cursor, execute};

pub struct Cursor {
    pub x: u16,
    pub y: u16,
    pub x_limit: u16,
    pub y_limit: u16,
}

impl Cursor {
    pub fn new() -> Self {
        let (x_limit, y_limit) = crossterm::terminal::size().unwrap();
        Self {
            x: 0,
            y: 0,
            x_limit,
            y_limit,
        }
    }

    pub fn set_limit(&mut self, x: u16, y: u16) {
        self.x_limit = x;
        self.y_limit = y;
    }

    pub fn update(&mut self, x: u16, y: u16) {
        self.x = x;
        self.y = y;
    }

    pub fn move_to(&mut self, x: u16, y: u16) {
        if x > self.x_limit {
            self.x_limit = x;
        }
        if y > self.y_limit {
            self.y_limit = y;
        }
        self.update(x, y);
        
    }

    pub fn parse_direction(&mut self, direction: KeyCode) {
        match direction {
            KeyCode::Up | KeyCode::Char('w') => {
                if self.y > 0 {
                    self.move_to(self.x, self.y - 1);
                }
            },
            KeyCode::Down | KeyCode::Char('s') => {
                self.move_to(self.x, self.y + 1);
            },
            KeyCode::Left | KeyCode::Char('a') => {
                if self.x > 0 {
                    self.move_to(self.x - 1, self.y);
                }
            },
            KeyCode::Right | KeyCode::Char('d') => {
                self.move_to(self.x + 1, self.y);
            },
            _ => {},
        }
    }
    
}