use std::{io::{Stdout, Write, Error, ErrorKind}, time::Duration, path::Path};
use crossterm::{
    event::{poll, read, Event, KeyCode, KeyModifiers},
    terminal,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    cursor,
    Result,
    queue, execute,
};

use crate::cursor::Cursor;
use crate::status_message::{self, StatusMessage};
use crate::home::home;
use crate::editor::editor;

pub enum Tab {
    Home,
    Editor,
}

pub struct Window {
    pub renderer: Stdout,
    pub cursor: Cursor,
    pub tab: Tab,
    pub status_message: StatusMessage,
    pub open_files: Vec<String>,
}

impl Window {
    pub fn new() -> Self {
        Self {
            renderer: std::io::stdout(),
            cursor: Cursor::new(),
            tab: Tab::Home,
            status_message: StatusMessage::new(),
            open_files: Vec::new(),
        }
    }

    pub fn parse_command(&mut self) -> Result<()> {
        let commands: Vec<&str> = self.status_message.command.split_whitespace().collect();
        match commands[0] {
            "q" => {
               return Err(Error::new(ErrorKind::Other, "Quit").into());
            },
            "o" => {
                if commands.len() > 1 && Path::new(commands[1]).exists(){
                    self.open_files.push(commands[1].to_string());
                    self.tab = Tab::Editor;
                    self.cursor.move_to(0, 0, &mut self.renderer)?;
                    self.cursor.clear_offset();
                } else {
                    self.status_message.error = "No file specified".to_string();
                    self.status_message.mode = status_message::Mode::Error;
                }
            },
            _ => {
                self.status_message.error = format!("Not a command: {}", self.status_message.command);
                self.status_message.mode = status_message::Mode::Error;
            }
        }
        Ok(())
    }

    fn parse_input(&mut self) -> Result<()> {
        if poll(Duration::from_millis(50))? {
            match read()? {
                Event::Key(key) => {
                    match key.modifiers {
                        KeyModifiers::NONE | KeyModifiers::SHIFT => {
                            if let status_message::Mode::Enabled = self.status_message.mode {
                                match key.code {
                                    KeyCode::Char(c) => {
                                        self.status_message.command.push(c);
                                    },
                                    KeyCode::Esc => {
                                        self.status_message.mode = status_message::Mode::Disabled;
                                        self.status_message.command.clear();
                                        self.cursor.movable = true;
                                    },
                                    KeyCode::Enter => {
                                        self.status_message.mode = status_message::Mode::Disabled;
                                        self.parse_command()?;
                                        self.status_message.command.clear();
                                        self.cursor.movable = true;
                                    },
                                    KeyCode::Backspace => {
                                        if let None = self.status_message.command.pop() {
                                            self.status_message.mode = status_message::Mode::Disabled;
                                        }
                                    },
                                    _ => {}
                                }
                            }
                            else {
                                match key.code {
                                    KeyCode::Char(':') => {
                                        self.status_message.mode = status_message::Mode::Enabled;
                                    },
                                    direction @ (KeyCode::Up | KeyCode::Down | KeyCode::Left | KeyCode::Right | KeyCode::Char('w') | KeyCode::Char('a') | KeyCode::Char('s') | KeyCode::Char('d')) => {
                                        self.cursor.move_cursor(direction, &mut self.renderer)?;
                                    },
                                    _ => {}
                                }
                            }
                        },
                        _ => {}
                    }
                },
                _ => {}
            }
        };
        Ok(())
    }

    pub fn ui(&mut self) -> Result<()> {
        loop {
            self.parse_input()?;
            execute!(self.renderer, cursor::Hide)?;
            match self.tab {
                Tab::Home => {
                    home(self)?;
                },
                Tab::Editor => {
                    editor(self)?;
                }
            }
            let (terminal_x, terminal_y) = terminal::size()?;
            match self.status_message.mode {
                status_message::Mode::Enabled => {
                    write!(self.renderer, ":{}\r", self.status_message.command)?;
                    queue!(self.renderer, cursor::MoveTo(1+self.status_message.command.len() as u16,terminal_y-1))?;
                    self.cursor.movable = false;
                },
                status_message::Mode::Error => {
                    queue!(self.renderer, SetForegroundColor(Color::Red) , Print(format!("Error: {}\r", self.status_message.error).as_str()), ResetColor)?;
                    queue!(self.renderer, cursor::MoveTo(self.cursor.x, self.cursor.y))?;
                },
                _ => {
                    queue!(self.renderer, cursor::MoveTo(self.cursor.x, self.cursor.y))?;
                }
            } 
            queue!(self.renderer, cursor::Show)?;
            self.renderer.flush()?;
        }
    }
}