use std::{io::{Stdout, Write, Error, ErrorKind}, time::Duration, path::Path};
use crossterm::{
    event::{poll, read, Event, KeyCode, KeyModifiers, MouseEventKind, MouseButton},
    terminal,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor, Stylize},
    cursor,
    Result,
    queue, execute,
};

use crate::{cursor::Cursor, readonly};
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
    pub readonly: bool,
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
            readonly: false,
        }
    }
    pub fn new_readonly(content: Vec<String>, title: String) -> Self {
        Self {
            content,
            path: title,
            offset_x: 0,
            offset_y: 0,
            x: 0,
            y: 0,
            readonly: true,
        }
    }
    pub fn for_home() -> Self {
        Self {
            content: vec![],
            path: "[for init]".to_string(),
            offset_x: 0,
            offset_y: 0,
            x: 0,
            y: 0,
            readonly: true,
        }
    }
}

pub struct InsertedChar {
    pub character: KeyCode,
    pub modifier: KeyModifiers,
    pub x: u16,
    pub y: u16,
}

impl InsertedChar {
    fn new(character: KeyCode, modifier: KeyModifiers, x: u16, y: u16) -> Self {
        Self {
            character,
            modifier,
            x,
            y,
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
            open_files: vec![File::for_home()],
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
                    // check if file is already open
                    if self.open_files.iter().any(|file| file.path ==  Path::new(commands[1]).to_str().unwrap().to_string()) {
                        if let Some(index) = self.open_files.iter().position(|file| file.path ==  Path::new(commands[1]).to_str().unwrap().to_string()) {
                            self.current_file_index = index;
                            self.tab = Tab::Editor;
                            return Ok(());
                        }
                    }
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
            },
            "h" | "help" => {
                if self.open_files[0].path == "[for init]" {
                    self.open_files.remove(0);
                }
                self.open_files.push(File::new_readonly(readonly::help(), "Help".to_string()));
                self.current_file_index = self.open_files.len() - 1;
                self.tab = Tab::Editor;
                self.cursor.move_to(self.open_files[self.current_file_index].x, self.open_files[self.current_file_index].x, &mut self.renderer, &mut self.open_files[self.current_file_index])?;
            },
            _ => {
                self.status_message.error = format!("Not a command: {}", self.status_message.command);
                self.status_message.mode = status_message::Mode::Error;
            }
        }
        Ok(())
    }

    fn parse_quick_command(&mut self) -> Result<()> {
        let command = self.status_message.quick_command.clone();
        let current_file = &mut self.open_files[self.current_file_index];
        match command.chars().last().unwrap() {
            'j' => {
                if command.len() > 1 {
                    match command[0..command.len()-1].parse::<u16>() {
                        Ok(y) => {
                            self.cursor.move_to(self.open_files[self.current_file_index].x, y, &mut self.renderer, &mut self.open_files[self.current_file_index])?;
                        },
                        Err(_) => {
                            self.status_message.error = "Not a number".to_string();
                            self.status_message.mode = status_message::Mode::Error;
                        }
                    }
                } else {
                    self.status_message.error = "No number specified".to_string();
                    self.status_message.mode = status_message::Mode::Error;
                }
                self.status_message.quick_command.clear();
            },
            'r' => {
                if command == "rr" {
                    current_file.content.remove((current_file.y + current_file.offset_y - 2) as usize);
                    self.cursor.move_to(current_file.x, current_file.y-1, &mut self.renderer, &mut self.open_files[self.current_file_index])?;
                    self.status_message.quick_command.clear();
                }
            }
            _ => ()
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
                                    self.inserted_char = Some(InsertedChar::new(key.code, key.modifiers, self.open_files[self.current_file_index].x, self.open_files[self.current_file_index].y));
                                }
                                else if let editor::Mode::View = self.editor_mode {
                                    match key.code {
                                        KeyCode::Char('i') => {
                                            if self.open_files[self.current_file_index].readonly {
                                                self.status_message.error = "File is readonly".to_string();
                                                self.status_message.mode = status_message::Mode::Error;
                                            }
                                            else {
                                                self.editor_mode = editor::Mode::Insert;
                                                queue!(self.renderer, cursor::SetCursorStyle::BlinkingBar)?;
                                            }
                                        },
                                        KeyCode::Char('n') | KeyCode::Char('N') => {
                                            if let KeyModifiers::SHIFT = key.modifiers {
                                                let file = self.open_files.remove(self.current_file_index);
                                                self.current_file_index = if self.current_file_index == self.open_files.len() {
                                                    0
                                                } else {
                                                    self.current_file_index + 1
                                                };
                                                self.open_files.insert(self.current_file_index, file);
                                            }
                                            else {
                                                self.current_file_index = if self.current_file_index == self.open_files.len() - 1 {
                                                    0
                                                } else {
                                                    self.current_file_index + 1
                                                };
                                                self.cursor.move_to(self.open_files[self.current_file_index].x, self.open_files[self.current_file_index].y, &mut self.renderer, &mut self.open_files[self.current_file_index])?;
                                            }
                                        }
                                        KeyCode::Char('b') | KeyCode::Char('B') => {
                                            if let KeyModifiers::SHIFT = key.modifiers {
                                                let file = self.open_files.remove(self.current_file_index);
                                                self.current_file_index = if self.current_file_index == 0 {
                                                    self.open_files.len()
                                                } else {
                                                    self.current_file_index - 1
                                                };
                                                self.open_files.insert(self.current_file_index, file);
                                            }
                                            else {
                                                self.current_file_index = if self.current_file_index == 0 {
                                                    self.open_files.len() - 1
                                                } else {
                                                    self.current_file_index - 1
                                                };
                                                self.cursor.move_to(self.open_files[self.current_file_index].x, self.open_files[self.current_file_index].y, &mut self.renderer, &mut self.open_files[self.current_file_index])?;
                                            }
                                        },
                                        KeyCode::Char('x') => {
                                            self.open_files.remove(self.current_file_index);
                                            self.current_file_index = self.current_file_index.saturating_sub(1);
                                            if self.open_files.len() == 0 {
                                                self.open_files.push(File::for_home());
                                                self.tab = Tab::Home;
                                            }
                                        },
                                        KeyCode::Char(':') => {
                                            self.status_message.mode = status_message::Mode::Enabled;
                                        },
                                        directions @ (KeyCode::Up | KeyCode::Down | KeyCode::Left | KeyCode::Right | KeyCode::Char('w') | KeyCode::Char('a') | KeyCode::Char('s') | KeyCode::Char('d')) => {
                                            self.cursor.move_cursor(directions, &mut self.renderer, &mut self.open_files[self.current_file_index])?;
                                        },
                                        KeyCode::Char(c) => {
                                            self.status_message.quick_command.push(c);
                                            match c {
                                                'j' | 'r' => self.parse_quick_command()?,
                                                // '%' => self.status_message.quick_command.clear(),
                                                _ => {}
                                            }
                                        },
                                        KeyCode::Esc => {
                                            self.status_message.quick_command.clear();
                                        },
                                        _ => {
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
                Event::Mouse(event) => {
                    match event.kind {
                        MouseEventKind::Down(button) => {
                            if let MouseButton::Left = button {
                                self.cursor.move_to(event.column as u16, event.row as u16, &mut self.renderer, &mut self.open_files[self.current_file_index])?;
                            }
                        }
                        MouseEventKind::ScrollDown => {
                            self.cursor.move_cursor(KeyCode::Down, &mut self.renderer, &mut self.open_files[self.current_file_index])?;
                        }
                        MouseEventKind::ScrollUp => {
                            self.cursor.move_cursor(KeyCode::Up, &mut self.renderer, &mut self.open_files[self.current_file_index])?;
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
            execute!(self.renderer, cursor::Hide, cursor::DisableBlinking)?;
            match self.tab {
                Tab::Home => {
                    home(self)?;
                },
                Tab::Editor => {
                    editor::editor(self)?;
                }
            }
            if let Tab::Editor = self.tab {
                if self.open_files[self.current_file_index].y == 0 || self.open_files[self.current_file_index].y == 1 {
                    self.open_files[self.current_file_index].y = 2 
                }
            }

            self.render_status_message()?;

            queue!(self.renderer, cursor::Show, cursor::EnableBlinking)?;
            self.renderer.flush()?;
        }
    }
    pub fn render_status_message(&mut self) -> Result<()> {
        let current_file = &self.open_files[self.current_file_index];
        let (terminal_x, terminal_y) = terminal::size()?;
        let (x,y) = if let status_message::Mode::Enabled = self.status_message.mode { (1+self.status_message.command.len() as u16, terminal_y-1) } else { (current_file.x, current_file.y) };

        let mode = match self.editor_mode {
            editor::Mode::Insert => String::from("--insert mode--").red().to_string(),
            editor::Mode::View => String::from("--view mode--").green().to_string(),
            _ => String::new()
        };

        let top_right = match self.tab {
            Tab::Home => String::from("Home "),
            Tab::Editor => {
                let (file_x, file_y) = (current_file.x+current_file.offset_x+1,current_file.y+current_file.offset_y-1);
                format!("{} {}:{} {}", self.status_message.quick_command.clone().dark_red(), file_y, file_x, mode)
            },
            _ => String::new()
        };

        let top_left = match self.tab {
            Tab::Editor => String::new(),
            _ => String::new()
        };

        let bottom = match self.status_message.mode {
            status_message::Mode::Enabled => {
                self.cursor.movable = false;
                format!(":{}", self.status_message.command)
            },
            status_message::Mode::Error => format!("Error: {}", self.status_message.error).red().to_string(),
            status_message::Mode::Success => format!("{}", self.status_message.success).green().to_string(),
            _ => String::new()
        };

        let status_bar = format!("{}{}{} \r\n", top_left, " ".repeat(terminal_x as usize-strip_ansi_escapes::strip(&top_right).unwrap().len()-strip_ansi_escapes::strip(&top_left).unwrap().len()-1), top_right); // theres a random -1 because of padding on the right for better view idfk
        let status_message = format!("{}", bottom);

        queue!(self.renderer, terminal::Clear(terminal::ClearType::CurrentLine))?;
        queue!(self.renderer, Print(status_bar))?;
        queue!(self.renderer, terminal::Clear(terminal::ClearType::CurrentLine))?;
        queue!(self.renderer, Print(status_message))?;
        queue!(self.renderer, cursor::MoveTo(x,y))?;
        Ok(())
    }
}