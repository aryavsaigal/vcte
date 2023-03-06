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

#[derive(Clone)]
pub struct File {
    pub content: Vec<String>,
    pub path: String,
    pub offset_x: u16,
    pub offset_y: u16,
}

impl File {
    pub fn new(path: &Path) -> Self {
        let content = editor::open_file(&Path::new(path));
        Self {
            content,
            path: path.to_str().unwrap().to_string(),
            offset_x: 0,
            offset_y: 0,
        }
    }
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
    pub open_files: Vec<File>,
    pub editor_mode: editor::Mode,
    pub inserted_char: Option<InsertedChar>,
    pub current_file_index: usize,

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
            current_file_index: 0,
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
                    self.open_files.push(File::new(&Path::new(commands[1])));
                    self.current_file_index = self.open_files.len() - 1;
                    self.tab = Tab::Editor;
                    self.cursor.move_to(0, 0, &mut self.renderer)?;
                } else {
                    self.status_message.error = "No file specified".to_string();
                    self.status_message.mode = status_message::Mode::Error;
                }
            },
            "s" | "save" => {
                if let Tab::Editor = self.tab {
                    editor::save_file(&self.open_files[self.current_file_index])?;
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
                            else if let Tab::Editor = self.tab {
                                if let editor::Mode::Insert = self.editor_mode {
                                    match key.code {
                                        KeyCode::Esc => {
                                            self.editor_mode = editor::Mode::View;
                                            queue!(self.renderer, cursor::SetCursorStyle::DefaultUserShape)?;
                                        },
                                        KeyCode::Char(c) => {
                                            self.inserted_char = Some(InsertedChar::new(c, self.cursor.x, self.cursor.y, false));
                                        },
                                        KeyCode::Backspace => {
                                            self.inserted_char = Some(InsertedChar::new(' ', self.cursor.x, self.cursor.y, true));
                                        },
                                        direction @ (KeyCode::Up | KeyCode::Down | KeyCode::Left | KeyCode::Right) => {
                                            self.cursor.move_cursor(direction, &mut self.renderer, &mut self.open_files[self.current_file_index])?;
                                            self.status_message.mode = status_message::Mode::Disabled;
                                        },
                                        _ => {}
                                    }
                                }
                                else if let editor::Mode::View = self.editor_mode {
                                    match key.code {
                                        KeyCode::Char('i') => {
                                            self.editor_mode = editor::Mode::Insert;
                                            queue!(self.renderer, cursor::SetCursorStyle::BlinkingBar)?;
                                        },
                                        KeyCode::Char('n') => {
                                            self.current_file_index = if self.current_file_index == self.open_files.len() - 1 {
                                                0
                                            } else {
                                                self.current_file_index + 1
                                            };
                                            self.cursor.move_to(0, 0, &mut self.renderer)?;
                                        }
                                        KeyCode::Char('b') => {
                                            self.current_file_index = if self.current_file_index == 0 {
                                                self.open_files.len() - 1
                                            } else {
                                                self.current_file_index - 1
                                            };
                                            self.cursor.move_to(0, 0, &mut self.renderer)?;
                                        },
                                        KeyCode::Char(':') => {
                                            self.status_message.mode = status_message::Mode::Enabled;
                                        },
                                        _ => {
                                            self.cursor.move_cursor(key.code, &mut self.renderer, &mut self.open_files[self.current_file_index])?;
                                            self.status_message.mode = status_message::Mode::Disabled;
                                        }
                                    }
                                }
                            } 
                            else {
                                match key.code {
                                    KeyCode::Char(':') => {
                                        self.status_message.mode = status_message::Mode::Enabled;
                                    },
                                    _ => {
                                        self.cursor.move_cursor(key.code, &mut self.renderer, &mut self.open_files[self.current_file_index])?;
                                        self.status_message.mode = status_message::Mode::Disabled;
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
            if let Tab::Editor = self.tab {
                if self.cursor.y == 0 {
                    self.cursor.y = 2 
                }
            }
            queue!(self.renderer, terminal::Clear(terminal::ClearType::UntilNewLine))?;
            let (terminal_x, terminal_y) = terminal::size()?;
            match self.status_message.mode {
                status_message::Mode::Enabled => {
                    write!(self.renderer, ":{}", self.status_message.command)?;
                    queue!(self.renderer, cursor::MoveTo(1+self.status_message.command.len() as u16,terminal_y-1))?;
                    self.cursor.movable = false;
                },
                status_message::Mode::Error => {
                    queue!(self.renderer, SetForegroundColor(Color::Red) , Print(format!("Error: {}", self.status_message.error).as_str()), ResetColor)?;
                    queue!(self.renderer, cursor::MoveTo(self.cursor.x, self.cursor.y))?;
                },
                status_message::Mode::Success => {
                    queue!(self.renderer, SetForegroundColor(Color::Green) , Print(format!("{}", self.status_message.success).as_str()), ResetColor)?;
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