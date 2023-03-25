use crossterm::{self, event::KeyCode};

pub struct Cursor {
    pub x: u16,
    pub y: u16,
    pub x_offset: u16,
    pub y_offset: u16,
    pub x_max: u16,
    pub y_max: u16,
    pub x_min: u16,
    pub y_min: u16,
}

impl Cursor {
    pub fn new() -> Self {
        let (x_limit, y_limit) = crossterm::terminal::size().unwrap();
        Self {
            x: 0,
            y: 0,
            x_offset: 0,
            y_offset: 0,
            x_max: x_limit,
            y_max: y_limit-2,
            x_min: 0,
            y_min: 0,
        }
    }

    pub fn set_max(&mut self, x: u16, y: u16) {
        self.x_max = x;
        self.y_max = y;
    }

    pub fn set_min(&mut self, x: u16, y: u16) {
        self.x_min = x;
        self.y_min = y;
    }

    pub fn update(&mut self, x: u16, y: u16) {
        self.x = x;
        self.y = y;
    }

    pub fn move_to(&mut self, mut x: u16, mut y: u16) {
        if x > self.x_max {
            self.x_offset += 1;
            x = self.x_max;
        } else if x < self.x_min {
            self.x_offset = self.x_offset.saturating_sub(1);
            x = self.x_min;
        }
        
        if y > self.y_max {
            self.y_offset += 1;
            y = self.y_max;
        } else if y < self.y_min {
            self.y = self.y_offset.saturating_sub(1);
            y = self.y_min; 
        }

        self.update(x, y);
    }

    pub fn parse_direction(&mut self, direction: KeyCode) {
        match direction {
            KeyCode::Up | KeyCode::Char('w') => {
                if self.y == self.y_min {
                    return self.y_offset = self.y_offset.saturating_sub(1);
                }
                self.move_to(self.x, self.y - 1);
            },
            KeyCode::Down | KeyCode::Char('s') => {
                self.move_to(self.x, self.y + 1);
            },
            KeyCode::Left | KeyCode::Char('a') => {
                if self.x == self.x_min {
                    return self.x_offset = self.x_offset.saturating_sub(1);
                }
                self.move_to(self.x - 1, self.y);
            },
            KeyCode::Right | KeyCode::Char('d') => {
                self.move_to(self.x + 1, self.y);
            },
            _ => {},
        }
    }
    
}