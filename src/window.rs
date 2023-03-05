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
use crate::editor;

pub enum Tab {
    Home,
    Editor,
}

pub struct InsertedChar {
    pub character: char,
    pub x: u16,
    pub y: u16,
    pub backspace: bool,
}

impl InsertedChar {
    fn new(character: char, x: u16, y: u16, backspace: bool) -> Self {
        Self {
            character,
            x,
            y,
            backspace,
        }
    }
}

pub struct Window {
    pub renderer: Stdout,
    pub cursor: Cursor,
    pub tab: Tab,
    pub status_message: StatusMessage,
    pub open_files: Vec<String>,
    pub editor_mode: editor::Mode,
    pub inserted_char: Option<InsertedChar>,
    pub open_file: Vec<String>,

}

impl Window {
    pub fn new() -> Self {
        Self {
            renderer: std::io::stdout(),
            cursor: Cursor::new(),
            tab: Tab::Home,
            status_message: StatusMessage::new(),
            open_files: Vec::new(),
            editor_mode: editor::Mode::View,
            inserted_char: None,
            open_file: Vec::new(),
        }
    }

    pub fn parse_command(&mut self) -> Result<()> {
        let commands: Vec<&str> = self.status_message.command.split_whitespace().collect();
        match commands[0] {
            "q" | "quit" => {
               return Err(Error::new(ErrorKind::Other, "Quit").into());
            },
            "o" | "open" => {
                if commands.len() > 1 && Path::new(commands[1]).exists(){
                    self.open_files.push(commands[1].to_string());
                    self.tab = Tab::Editor;
                    self.cursor.move_to(0, 0, &mut self.renderer)?;
                    self.open_file = editor::open_file(&Path::new(&self.open_files[0]));
                    self.cursor.clear_offset();
                } else {
                    self.status_message.error = "No file specified".to_string();
                    self.status_message.mode = status_message::Mode::Error;
                }
            },
            "s" | "save" => {
                if let Tab::Editor = self.tab {
                    editor::save_file(&Path::new(&self.open_files[0]), &self.open_file)?;
                    self.status_message.success = "File saved".to_string();
                    self.status_message.mode = status_message::Mode::Success;
                } 
                else {
                    self.status_message.error = "No file open".to_string();
                    self.status_message.mode = status_message::Mode::Error;
                }
            }
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
                            else if let editor::Mode::Insert = self.editor_mode {
                                match key.code {
                                    KeyCode::Esc => {
                                        self.editor_mode = editor::Mode::View;
                                    },
                                    KeyCode::Char(c) => {
                                        self.inserted_char = Some(InsertedChar::new(c, self.cursor.x, self.cursor.y, false));
                                    },
                                    KeyCode::Backspace => {
                                        self.inserted_char = Some(InsertedChar::new(' ', self.cursor.x, self.cursor.y, true));
                                    },
                                    _ => {}
                                }
                            }
                            else {
                                match key.code {
                                    KeyCode::Char(':') => {
                                        self.status_message.mode = status_message::Mode::Enabled;
                                    },
                                    KeyCode::Char('i') => {
                                        if let editor::Mode::View = self.editor_mode {
                                            self.editor_mode = editor::Mode::Insert;
                                        }
                                    },
                                    _ => {
                                        self.cursor.move_cursor(key.code, &mut self.renderer)?;
                                    }
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
                    editor::editor(self)?;
                }
            }
            queue!(self.renderer, terminal::Clear(terminal::ClearType::UntilNewLine))?;
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
                status_message::Mode::Success => {
                    queue!(self.renderer, SetForegroundColor(Color::Green) , Print(format!("{}\r", self.status_message.success).as_str()), ResetColor)?;
                    queue!(self.renderer, cursor::MoveTo(self.cursor.x, self.cursor.y))?;
                },
                _ => {
                    if let Tab::Editor = self.tab {
                        if let editor::Mode::Insert = self.editor_mode {
                            let msg = String::from("--insert mode--");
                            queue!(self.renderer, Print(format!("{}", " ".repeat(terminal_x as usize - msg.len()))), SetForegroundColor(Color::Red), Print(format!("{}", msg)), ResetColor)?;
                        }
                        else if let editor::Mode::View = self.editor_mode {
                            let msg = String::from("--view mode--");
                            queue!(self.renderer, Print(format!("{}", " ".repeat(terminal_x as usize - msg.len()))), SetForegroundColor(Color::Green), Print(format!("{}", msg)), ResetColor)?;
                        }
                    }
                    queue!(self.renderer, cursor::MoveTo(self.cursor.x, self.cursor.y))?;
                }
            } 
            queue!(self.renderer, cursor::Show)?;
            self.renderer.flush()?;
        }
    }
}