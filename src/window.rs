use std::{io::{Stdout, Write, Error, ErrorKind}, time::Duration, path::Path};
use crossterm::{
    event::{poll, read, Event, KeyCode, KeyModifiers},
    terminal,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor, Stylize},
    cursor,
    Result,
    queue, execute,
};

use crate::cursor::Cursor;
use crate::status_message::{self, StatusMessage};
use crate::home::home;
use crate::editor;

extern crate strip_ansi_escapes;

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
    pub x: u16,
    pub y: u16,
}

impl File {
    pub fn new(path: &Path) -> Self {
        let content = editor::open_file(&Path::new(path));
        Self {
            content,
            path: path.to_str().unwrap().to_string(),
            offset_x: 0,
            offset_y: 0,
            x: 0,
            y: 0,
        }
    }
    pub fn for_init() -> Self {
        Self {
            content: vec![],
            path: "[for init]".to_string(),
            offset_x: 0,
            offset_y: 0,
            x: 0,
            y: 0,
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
            open_files: vec![File::for_init()],
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
                if self.open_files[0].path == "[for init]" {
                    self.open_files.remove(0);
                }
                if commands.len() > 1 && Path::new(commands[1]).exists(){
                    self.open_files.push(File::new(&Path::new(commands[1])));
                    self.current_file_index = self.open_files.len() - 1;
                    self.tab = Tab::Editor;
                    self.cursor.move_to(self.open_files[self.current_file_index].x, self.open_files[self.current_file_index].x, &mut self.renderer, &mut self.open_files[self.current_file_index])?;
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
                                            self.inserted_char = Some(InsertedChar::new(c, self.open_files[self.current_file_index].x, self.open_files[self.current_file_index].y, false));
                                        },
                                        KeyCode::Backspace => {
                                            self.inserted_char = Some(InsertedChar::new(' ', self.open_files[self.current_file_index].x, self.open_files[self.current_file_index].y, true));
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
                                            self.cursor.move_to(self.open_files[self.current_file_index].x, self.open_files[self.current_file_index].y, &mut self.renderer, &mut self.open_files[self.current_file_index])?;
                                        }
                                        KeyCode::Char('b') => {
                                            self.current_file_index = if self.current_file_index == 0 {
                                                self.open_files.len() - 1
                                            } else {
                                                self.current_file_index - 1
                                            };
                                            self.cursor.move_to(self.open_files[self.current_file_index].x, self.open_files[self.current_file_index].y, &mut self.renderer, &mut self.open_files[self.current_file_index])?;
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
                if self.open_files[self.current_file_index].y == 0 {
                    self.open_files[self.current_file_index].y = 2 
                }
            }

            self.render_status_message()?;

            queue!(self.renderer, cursor::Show)?;
            self.renderer.flush()?;
        }
    }
    pub fn render_status_message(&mut self) -> Result<()> {
        let current_file = &self.open_files[self.current_file_index];
        let (terminal_x, terminal_y) = terminal::size()?;
        let (x,y) = if let status_message::Mode::Enabled = self.status_message.mode { (1+self.status_message.command.len() as u16, terminal_y-1) } else { (current_file.x, current_file.y) };

        let top = match self.tab {
            Tab::Home => String::from("Home"),
            Tab::Editor => format!("{}:{} {}", current_file.y+current_file.offset_y-1, current_file.x+current_file.offset_x+1, match self.editor_mode {
                editor::Mode::Insert => String::from("--insert mode--") .red().to_string(),
                editor::Mode::View => String::from("--view mode--") .green().to_string(),
                _ => String::new()
            }),
            _ => String::new()
        };

        let bottom = match self.status_message.mode {
            status_message::Mode::Enabled => {
                self.cursor.movable = false;
                format!(":{}", self.status_message.command)
            },
            status_message::Mode::Error => format!("Error: {}", self.status_message.error) .red().to_string(),
            status_message::Mode::Success => format!("{}", self.status_message.success).green().to_string(),
            _ => String::new()
        };

        let status_bar = format!("{}{}\r\n", " ".repeat(terminal_x as usize-strip_ansi_escapes::strip(&top).unwrap().len()), top);
        let status_message = format!("{}", bottom);

        queue!(self.renderer, terminal::Clear(terminal::ClearType::CurrentLine))?;
        queue!(self.renderer, Print(status_bar))?;
        queue!(self.renderer, terminal::Clear(terminal::ClearType::CurrentLine))?;
        queue!(self.renderer, Print(status_message))?;
        queue!(self.renderer, cursor::MoveTo(x,y))?;
        Ok(())
    }
}