use std::{path::Path, fs::{DirEntry, self}};
use crossterm::{terminal, style::Color};
use crate::{colour_string::{ColourString, Info}, cursor::Cursor, editor::File};
use unicode_segmentation::UnicodeSegmentation;

#[derive(Clone, Debug)]
pub struct Content {
    pub path: String,
    pub file_name: String,
    pub is_dir: bool,
    pub parent: String,
    pub y: usize,
}

impl Content {
    pub fn new(path: String, y: usize) -> Self {
        let path = Path::new(&path);
        let file_name = path.file_name().unwrap().to_str().unwrap().to_string();
        let is_dir = path.is_dir();
        let parent = path.parent().unwrap().to_str().unwrap().to_string();
        Self {
            path: path.to_str().unwrap().to_string(),
            file_name,
            is_dir,
            parent,
            y
        }
    }
}

#[derive(Clone)]
pub struct Name {
    pub name: String,
    pub current_file: bool,
}

impl Name {
    pub fn new(name: String, current_file: bool) -> Self {
        Self {
            name,
            current_file,
        }
    }
}

pub struct FileExplorer {
    pub enabled: bool,
    pub selected: bool,
    pub cursor: Cursor,
    pub open_dirs: Vec<String>,
    pub contents: Vec<Content>,
}

impl FileExplorer {
    pub fn new() -> Self {
        let mut new = Self {
            enabled: false,
            selected: false,
            cursor: Cursor::new(),
            open_dirs: vec![],
            contents: vec![],
        };
        new.cursor.set_min(0, 1);
        new.cursor.y = 1;
        new
    }

    pub fn load_child(&mut self, child: DirEntry, file: &File, i: usize) -> Vec<Name> {
        let child = Content::new(child.path().to_str().unwrap().to_string(), 0);
        let mut names = vec![];
        let mut name = format!("{}{}", " ".repeat(i), child.file_name.clone());
        if child.is_dir {
            if self.open_dirs.contains(&child.path) {
                name.push_str(" ▼");
            } else {
                name.push_str(" ▶");
            }
        }

        self.contents.push(child.clone());
        names.push(Name::new(name, file.path == child.path));

        if self.open_dirs.contains(&child.path) {
            fs::read_dir(&child.path).unwrap().for_each(|child| {
                names.extend(self.load_child(child.unwrap(), file, i + 1));
            })
        }
        names
    }

    pub fn parse_input(&mut self) -> Option<String> {
        let mut new = self.contents.clone();
        let y = self.cursor.y as usize + self.cursor.y_offset as usize + self.cursor.y_min as usize;

        new.retain(|content| content.y == y);
        if new.len() > 0 {
            let new = new[0].clone();
            if new.is_dir {
                if self.open_dirs.contains(&new.path) {
                    self.open_dirs.retain(|dir| dir != &new.path);
                    self.contents.retain(|content| content.parent != new.path);
                } else {
                    self.open_dirs.push(new.path.clone());
                }
            }
            else {
                return Some(new.path);
            }
        }
        None
    }

    pub fn render(&mut self, file: &File) -> Vec<ColourString> {
        let (terminal_x, terminal_y) = terminal::size().unwrap();
        let max_x = terminal_x / 5;
        let mut frame: Vec<ColourString> = vec![ColourString::new(String::from(format!("{}▕", " ".repeat(max_x as usize))), None); terminal_y as usize];
        let mut files = vec![];
        self.contents.clear();

        for child in Path::new(&file.path).parent().unwrap().read_dir().unwrap() {
            if let Ok(child) = child {
                files.extend(self.load_child(child, file, 0));
            }
        } // change

        let mut y = self.cursor.y_offset as usize + self.cursor.y_min as usize + 1;
        for content in &mut self.contents {
            content.y = y;
            y += 1;
        }

        for (i, file) in files.iter_mut().enumerate() {
            if i > frame.len() - 1 {
                frame.push(ColourString::new(String::from(format!("{}▕", " ".repeat(max_x as usize))), None));
            }

            let mut name = file.name.graphemes(true).skip(self.cursor.x_offset as usize).collect::<Vec<&str>>();
            name.truncate(max_x as usize);
            let name = name.join("");
            frame[i].replace_range(0, name.graphemes(true).count(), ColourString::new(name.clone(), if file.current_file { Some(Info::new(Color::DarkGrey, Color::Reset, vec![])) } else { None }));
        }

        self.cursor.set_max(max_x, terminal_y-2);
        frame[self.cursor.y_offset as usize..].to_vec()
    }
}

