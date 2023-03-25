use std::{io::{Stdout, stdout, Write, ErrorKind, Error}, time::Duration, path::Path, fmt::format};
use crossterm::{
    event::{poll, read, Event, KeyCode, KeyModifiers},
    Result, 
    queue,
    cursor, terminal, style::Color
};

use crate::{colour_string::{ColourString, Info}, editor::File};
use crate::home::Home;
use crate::command_palette::CommandPalette;
use crate::file_explorer::FileExplorer;
use crate::status_bar::StatusBar;

pub struct Frame {
    pub content: Vec<ColourString>,
    pub push: bool,
    pub ignore_whitespace: bool
}

impl Frame {
    pub fn new(content: Vec<ColourString>, push: bool, ignore_whitespace: bool) -> Self {
        Self {
            content,
            push,
            ignore_whitespace
        }
    }
}

pub struct Window {
    pub renderer: Stdout,
    pub command_palette: CommandPalette,
    pub file_explorer: FileExplorer,
    pub status_bar: StatusBar,
    pub files: Vec<File>,
    pub file_index: usize,
    pub frames: Vec<Frame>,
    pub home: Home,
    pub overlay: bool,
}

impl Window {
    pub fn new() -> Self {
        Self {
            renderer: stdout(),
            frames: Vec::new(),
            files: Vec::new(),
            file_index: 0,
            home: Home::new(),
            command_palette: CommandPalette::new(),
            file_explorer: FileExplorer::new(),
            status_bar: StatusBar::new(),
            overlay: false,
        }
    }

    pub fn register(&mut self, frame: Vec<ColourString>, push: bool, ignore_whitespace: bool) {
        self.frames.push(Frame::new(frame, push, ignore_whitespace));
    }

    fn render_frames(&mut self) -> Result<()> {
        let (terminal_x, terminal_y) = terminal::size()?;
        let mut final_frame: Vec<ColourString> = vec![ColourString::new(String::from(" ".repeat(terminal_x as usize)), None); terminal_y as usize];
        let mut it = self.frames.iter_mut().peekable();
        while let Some(frame) = it.next() {
            frame.content.truncate(terminal_y as usize);

            let is_last = it.peek().is_none();
            for line_index in 0..frame.content.len() {
                if frame.content[line_index].get_content().is_empty() {
                    continue;
                }

                for (char_index, char) in frame.content[line_index].get_content().clone().iter_mut().enumerate().rev() {
                    if self.overlay && !is_last && char.content != "█" {
                        char.colour = Info::new(Color::DarkGrey, Color::Reset, vec![]);
                    }

                    if frame.push {
                        final_frame[line_index].l_shift(char.content.clone(), Some(char.colour.clone()));
                    } else {
                        if char.content != " " || frame.ignore_whitespace {
                            final_frame[line_index].replace_range(char_index, char_index+1, ColourString::new(char.content.clone(), Some(char.colour.clone())));
                        }
                    }

                    final_frame[line_index].truncate(terminal_x as usize);
                }
            }
        }

        queue!(self.renderer, cursor::MoveTo(0,0))?;
        write!(self.renderer, "{}", ColourString::render_vector(final_frame))?;
        self.overlay = false;
        self.frames.clear();
        Ok(())
    }


    pub fn parse_input(&mut self) -> Result<()> {
        if poll(Duration::from_millis(50))? {
            let (terminal_x, terminal_y) = terminal::size()?;
            match read()? {
                Event::Key(key) => {
                    match key.modifiers {
                        KeyModifiers::NONE | KeyModifiers::SHIFT => {
                            if self.command_palette.enabled {
                                match key.code {
                                    KeyCode::Char(c) => {
                                        self.command_palette.command.push(c);
                                    },
                                    KeyCode::Backspace => {
                                        self.command_palette.command.pop();
                                        if self.command_palette.command.is_empty() {
                                            self.command_palette.enabled = false;
                                        }
                                    },
                                    KeyCode::Enter => {
                                        self.command_palette.enabled = false;
                                        self.parse_command()?;
                                    },
                                    KeyCode::Esc => {
                                        self.command_palette.enabled = false;
                                        self.command_palette.command = String::new();
                                    },
                                    _ => {}
                                }
                            }
                            else if !self.files.is_empty() && self.files[self.file_index].insert {
                                match key.code {
                                    KeyCode::Char(c) => {
                                        self.files[self.file_index].insert_char(c);
                                    },
                                    KeyCode::Backspace => {
                                        self.files[self.file_index].backspace();
                                    },
                                    KeyCode::Enter => {
                                        self.files[self.file_index].enter();
                                    },
                                    KeyCode::Esc => {
                                        self.files[self.file_index].insert = false;
                                    },
                                    _ => {}
                                }
                            }
                            else {
                                match key.code {
                                    KeyCode::Char(':') => {
                                        self.command_palette.enabled = true;
                                    },
                                    KeyCode::Char('c') => {
                                        if self.file_explorer.enabled {
                                            self.file_explorer.enabled = false;

                                            if self.files.is_empty() {
                                                self.home.cursor.set_min(0, 0);
                                                self.home.cursor.x -= terminal_x / 5 + 1;
                                            }
                                            else {
                                                for file in &mut self.files {
                                                    file.cursor.set_min(5, 0);
                                                    file.cursor.x -= terminal_x / 5 + 1;
                                                }
                                            }
                                        }
                                        else {
                                            self.file_explorer.enabled = true;

                                            if self.files.is_empty() {
                                                self.home.cursor.x += terminal_x / 5 + 1;
                                                self.home.cursor.set_min(terminal_x / 5 + 1, 0);
                                            }
                                            else {
                                                for file in &mut self.files {
                                                    file.cursor.x += terminal_x / 5 + 1;
                                                    file.cursor.set_min(terminal_x / 5 + 1, 0);
                                                }
                                            }
                                        }
                                    },
                                    KeyCode::Char('C') => {
                                        self.file_explorer.selected = !self.file_explorer.selected;
                                    }
                                    KeyCode::Char('i') => {
                                        if !self.files.is_empty() {
                                            self.files[self.file_index].insert = true;
                                        }
                                    },
                                    direction @ (KeyCode::Up | KeyCode::Down | KeyCode::Left | KeyCode::Right | KeyCode::Char('w') | KeyCode::Char('a') | KeyCode::Char('s') | KeyCode::Char('d')) => {
                                        if self.file_explorer.selected {
                                            self.file_explorer.cursor.parse_direction(direction);
                                        }
                                        else if self.files.is_empty() {
                                            self.home.cursor.parse_direction(direction);
                                        }
                                        else {
                                            self.files[self.file_index].cursor.parse_direction(direction);
                                        }
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

    pub fn parse_command(&mut self) -> Result<()> {
        let args: Vec<&str> = self.command_palette.command.split_whitespace().collect();

        match args[0].to_lowercase().as_str() {
            "q" | "quit" => {
                return Err(Error::new(ErrorKind::Other, "Quit").into());
            },
            "o" | "open" => {
                if args.len() > 1 {
                    if !(Path::new(&args[1]).exists() && Path::new(&args[1]).is_file()) {
                        return Ok(()); // PLACEHOLDER
                    }
                    let mut file = File::new(args[1].to_string())?;
                    file.cursor.set_min(5, 0);
                    file.cursor.x = 5;
                    self.files.push(file);
                    self.file_index = self.files.len() - 1;
                }
            },
            "s" | "save" => {
                self.files[self.file_index].save()?;
            }
            _ => {
                
            }
        }
        self.command_palette.command.clear();
        Ok(())
    }

    pub fn render(&mut self) -> Result<()> {
        loop {
            self.parse_input()?;
            queue!(self.renderer, cursor::Hide, cursor::DisableBlinking)?;

            if self.files.is_empty() {
                let frame = self.home.render()?;
                self.register(frame, false, false);
            }
            else {
                let frame = self.files[self.file_index].render();
                self.register(frame, false, false);
            }

            if self.file_explorer.enabled {
                let frame = self.file_explorer.render();
                self.register(frame, true, false);
            }

            self.register(self.status_bar.render(), false, true);

            if self.command_palette.enabled {
                let frame = self.command_palette.render();
                self.register(frame, false, false);
                self.overlay = true;
            }
            
            self.render_frames()?;

            if self.command_palette.enabled {
                queue!(self.renderer, cursor::MoveTo(self.command_palette.cursor.x, self.command_palette.cursor.y))?;
            }
            else if self.file_explorer.selected {
                queue!(self.renderer, cursor::MoveTo(self.file_explorer.cursor.x, self.file_explorer.cursor.y))?;
            }
            else if self.files.is_empty() {
                queue!(self.renderer, cursor::MoveTo(self.home.cursor.x, self.home.cursor.y))?;
            }
            else {
                queue!(self.renderer, cursor::MoveTo(self.files[self.file_index].cursor.x, self.files[self.file_index].cursor.y))?;
            }

            if !self.files.is_empty() {
                let file = &self.files[self.file_index];
                let (terminal_x, terminal_y) = terminal::size()?;

                let mut message = if file.insert {
                    ColourString::new(format!("insert "), Some(Info::new(Color::Red, Color::Reset, vec![])))
                }
                else {
                    ColourString::new(format!("view "), Some(Info::new(Color::Green, Color::Reset, vec![])))
                };
                let end_message = format!("Ln {}, Col {}", file.cursor.y - file.cursor.y_min + file.cursor.y_offset + 1, file.cursor.x - file.cursor.x_min + file.cursor.x_offset + 1);

                message.push_str(&format!("{} {}", file.name, format_size(file.lines.join("\n").len() as u64)), None);
                message.push_str(&format!("{}", " ".repeat(terminal_x as usize - message.get_content().len()-end_message.len())), None);
                message.push_str(&end_message, None);

                self.status_bar.set_message(message);
            }

            queue!(self.renderer, cursor::Show, cursor::EnableBlinking)?;
            self.renderer.flush()?;
        }
    }
}

fn format_size(size_in_bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    
    let (divisor, unit) = if size_in_bytes >= GB {
        (GB, "gb")
    } else if size_in_bytes >= MB {
        (MB, "mb")
    } else if size_in_bytes >= KB {
        (KB, "kb")
    } else {
        (1, "b")
    };

    let size_in_units = size_in_bytes as f64 / divisor as f64;
    let size_number = format!("{:.2}", size_in_units).trim_end_matches("0").trim_end_matches(".").to_string();
    format!("{}{}", size_number, unit)
}
