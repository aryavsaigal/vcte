use std::io::Stdout;

use crossterm::{
    cursor,
    event::KeyCode,
    queue,
    Result,
    terminal
};

use crate::window::File;

pub struct Cursor {
    pub movable: bool
}

impl Cursor {
    pub fn new() -> Self {
        Self {
            movable: true
        }
    }

    pub fn update_coords(&mut self, coords: (u16, u16), file: &mut File) {
        file.x = coords.0;
        file.y = coords.1;
    }

    pub fn move_to(&mut self, x: u16, y: u16, renderer: &mut Stdout, file: &mut File) -> Result<()> {
        let (terminal_x, terminal_y) = terminal::size()?;
        // if x > terminal_x-1 {
        //     file.offset_x += x-file.x;
        // }
        if y >= file.content.len() as u16 - terminal_y + 3 {
            file.offset_y = file.content.len() as u16 - terminal_y + 3;

            let new_y = if y > file.content.len() as u16 {
                file.content.len() as u16 - (file.content.len() as u16 - terminal_y + 3)
            }
            else {
                y-file.offset_y+1
            };

            queue!(renderer, cursor::MoveTo(x, new_y))?;
            self.update_coords((x, new_y), file);
            return Ok(())
        } 
        else {
            if y > terminal_y-1 {
                file.offset_y = y-1;
                queue!(renderer, cursor::MoveTo(x, 1))?;
                self.update_coords((x, 1), file);
                return Ok(())
            }
            else if y < file.offset_y {
                file.offset_y = y-1;
                queue!(renderer, cursor::MoveTo(x, 1))?;
                self.update_coords((x, 1), file);
                return Ok(())
            }
        }

        queue!(renderer, cursor::MoveTo(x, y+1))?;
        self.update_coords((x, y+1), file);
        Ok(())
    }

    pub fn move_cursor(&mut self, direction: KeyCode, renderer: &mut Stdout, file: &mut File) -> Result<()> {
        if self.movable {
            let (terminal_x, terminal_y) = terminal::size()?;
            match direction {
                KeyCode::Up | KeyCode::Char('w') => {
                    if file.y == 2 && file.path != "[for init]" {
                        file.offset_y = file.offset_y.saturating_sub(1);
                    } 
                    else {
                        queue!(renderer, cursor::MoveUp(1))?;
                    }
                },
                KeyCode::Down | KeyCode::Char('s')=> {
                    if file.y == terminal_y-3 {
                        if file.offset_y+file.y < file.content.len() as u16 {
                            file.offset_y += 1;
                        }
                    } 
                    else {
                        queue!(renderer, cursor::MoveDown(1))?;
                    }
                },
                KeyCode::Left | KeyCode::Char('a') => {
                    if file.x == 0 {
                        file.offset_x = file.offset_x.saturating_sub(1)
                    } 
                    else {
                        queue!(renderer, cursor::MoveLeft(1))?;
                    }
                },
                KeyCode::Right | KeyCode::Char('d') => {
                    if file.x == terminal_x-1 {
                        file.offset_x += 1;
                    } 
                    else {
                        queue!(renderer, cursor::MoveRight(1))?;
                    }
                },
                _ => {}
            }
            self.update_coords(cursor::position()?, file);
        }
        Ok(())
    }
}