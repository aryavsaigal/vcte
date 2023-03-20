use std::{io::{Stdout, stdout, Write, ErrorKind, Error}, time::Duration, path::Path};
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

pub struct Frame {
    pub content: Vec<ColourString>,
    pub push: bool,
}

impl Frame {
    pub fn new(content: Vec<ColourString>, push: bool) -> Self {
        Self {
            content,
            push,
        }
    }
}

pub struct Window {
    pub renderer: Stdout,
    pub command_palette: CommandPalette,
    pub file_explorer: FileExplorer,
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
            overlay: false,
        }
    }

    pub fn register(&mut self, frame: Vec<ColourString>, push: bool) {
        self.frames.push(Frame::new(frame, push));
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
                    if char.content != " " {
                        if self.overlay && !is_last && char.content != "█" {
                            char.colour = Info::new(Color::DarkGrey, Color::Reset, vec![]);
                        }

                        if frame.push {
                            final_frame[line_index].l_shift(char.content.clone(), Some(char.colour.clone()));
                        } else {
                            final_frame[line_index].replace_range(char_index, char_index+1, ColourString::new(char.content.clone(), Some(char.colour.clone())));
                        }

                        final_frame[line_index].truncate(terminal_x as usize);
                    }
                }
            }
        }

        queue!(self.renderer, cursor::MoveTo(0,0))?;
        queue!(self.renderer, terminal::Clear(terminal::ClearType::All))?;
        write!(self.renderer, "{}", ColourString::render_vector(final_frame))?;
        self.overlay = false;
        self.frames.clear();
        Ok(())
    }


    pub fn parse_input(&mut self) -> Result<()> {
        if poll(Duration::from_millis(50))? {
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
                            else {
                                match key.code {
                                    KeyCode::Char(':') => {
                                        self.command_palette.enabled = true;
                                    },
                                    KeyCode::Char('c') => {
                                        self.file_explorer.enabled = !self.file_explorer.enabled;
                                    },
                                    direction @ (KeyCode::Up | KeyCode::Down | KeyCode::Left | KeyCode::Right | KeyCode::Char('w') | KeyCode::Char('a') | KeyCode::Char('s') | KeyCode::Char('d')) => {
                                        if self.files.is_empty() {
                                            debug!("YADA");
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
                    let file = File::new(args[1].to_string())?;
                    self.files.push(file);
                    self.file_index = self.files.len() - 1;
                }
            },
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
                self.register(frame, false);
            }
            else {
                let frame = self.files[self.file_index].render();
                self.register(frame, false);
            }

            if self.file_explorer.enabled {
                let frame = self.file_explorer.render();
                self.register(frame, true);
            }

            if self.command_palette.enabled {
                let frame = self.command_palette.render();
                self.register(frame, false);
                self.overlay = true;
            }
            
            self.render_frames()?;

            // if self.command_palette.enabled {
            //     queue!(self.renderer, crossterm::cursor::SetCursorStyle::BlinkingBar)?;
            //     queue!(self.renderer, cursor::MoveTo(self.command_palette.cursor.x, self.command_palette.cursor.y))?;
            // }
            // else if self.files.is_empty() {
            //     queue!(self.renderer, cursor::MoveTo(self.home.cursor.x, self.home.cursor.y))?;
            // }
            // else {
            //     queue!(self.renderer, cursor::MoveTo(self.files[self.file_index].cursor.x, self.files[self.file_index].cursor.y))?;
            // }

            queue!(self.renderer, cursor::Show, cursor::EnableBlinking)?;
            self.renderer.flush()?;
        }
    }
}