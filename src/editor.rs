use crate::colour_string::{ColourString, Info};
use crate::cursor::Cursor;
use crate::syntax_highlighter::SyntaxHighlighter;
use crossterm::{Result, terminal};
use crossterm::style::Color;
use std::fs;
use std::path::Path;
use unicode_segmentation::UnicodeSegmentation;

pub struct File {
    pub path: String,
    pub name: String,
    pub lines: Vec<String>,
    pub cursor: Cursor,
    pub modified: bool,
    pub insert: bool,
    pub highlighted_lines: Vec<ColourString>,
    pub syntax_highlighter: SyntaxHighlighter,
}

impl File {
    pub fn new(path: String) -> Result<Self> {
        let path = Path::new(&path);
        let name = path.file_name().unwrap().to_str().unwrap().to_string();
        let lines: Vec<String> = fs::read_to_string(path)?.lines().map(|s| s.to_string().replace("\t", "    ")).collect();

        let mut syntax_highlighter = SyntaxHighlighter::new();
        syntax_highlighter.init(lines.clone());

        let highlighted_lines: Vec<ColourString> = lines.iter().map(|line| {
            syntax_highlighter.highlight(line.to_string())
        }).collect();

        Ok(Self {
            path: fs::canonicalize(path)?.to_str().unwrap().to_string(),
            name,
            lines,
            cursor: Cursor::new(),
            modified: false,
            insert: false,
            highlighted_lines,
            syntax_highlighter,
        })
    }

    pub fn refresh_highlight(&mut self) {
        self.syntax_highlighter.init(self.lines.clone());
        self.highlighted_lines = self.lines.iter().map(|line| {
            self.syntax_highlighter.highlight(line.to_string())
        }).collect();
    }

    pub fn save(&mut self) -> Result<ColourString> {
        fs::write(&self.path, self.lines.join("\n"))?;
        Ok(ColourString::new(format!("saved file to {}", self.path), Some(Info::new(Color::Green, Color::Reset, vec![]))))
    }

    pub fn insert_char(&mut self, c: char) {
        while self.lines.len() <= (self.cursor.y + self.cursor.y_offset - self.cursor.y_min) as usize {
            self.lines.push(String::new());
        }

        let line = self.lines.get_mut((self.cursor.y + self.cursor.y_offset - self.cursor.y_min) as usize).unwrap();
        let mut graphemes = line.graphemes(true).collect::<Vec<&str>>();
        
        let x = self.cursor.x + self.cursor.x_offset - self.cursor.x_min;
        let c = c.to_string();
        if graphemes.len() < (x) as usize {
            while graphemes.len() < (x) as usize {
                graphemes.push(" ");
            }
        }
        graphemes.insert((x) as usize, c.as_str());
        *line = graphemes.join("");
        self.cursor.x += 1;
        self.highlighted_lines[(self.cursor.y + self.cursor.y_offset - self.cursor.y_min) as usize] = self.syntax_highlighter.highlight(line.to_string());
        self.modified = true;
    }

    pub fn backspace(&mut self) {
        let y = self.cursor.y + self.cursor.y_offset - self.cursor.y_min;

        if self.cursor.x > self.cursor.x_min {
            let line = self.lines.get_mut(y as usize).unwrap();
            let mut graphemes = line.graphemes(true).collect::<Vec<&str>>();

            if graphemes.len() > (self.cursor.x + self.cursor.x_offset - self.cursor.x_min - 1) as usize  {
                graphemes.remove((self.cursor.x + self.cursor.x_offset - self.cursor.x_min - 1) as usize);
            }
            *line = graphemes.join("");
            self.highlighted_lines[y as usize] = self.syntax_highlighter.highlight(line.to_string());
            self.cursor.x -= 1;
        }
        else if y != 0{
            let line = self.lines.remove(y as usize);
            self.highlighted_lines.remove(y as usize);
            let prev_line = self.lines.get_mut(y.saturating_sub(1) as usize).unwrap();

            self.cursor.y -= 1;
            self.cursor.x = prev_line.len() as u16 + self.cursor.x_min;

            prev_line.push_str(&line);
            self.highlighted_lines[y as usize - 1] = self.syntax_highlighter.highlight(prev_line.to_string());
        }

        self.modified = true;
    }

    pub fn enter(&mut self) {
        let y = self.cursor.y + self.cursor.y_offset - self.cursor.y_min;
        let line = self.lines.get_mut(y as usize).unwrap();
        let mut graphemes = line.graphemes(true).collect::<Vec<&str>>();

        let x = self.cursor.x + self.cursor.x_offset - self.cursor.x_min;

        let mut new_line = String::new();
        if graphemes.len() > x as usize {
            new_line = graphemes.split_off(x as usize).join("");
        }
        *line = graphemes.join("");

        self.highlighted_lines[y as usize] = self.syntax_highlighter.highlight(line.to_string());
        self.highlighted_lines.insert(y as usize + 1, self.syntax_highlighter.highlight(new_line.to_string()));

        self.lines.insert(y as usize + 1, new_line);

        self.cursor.y += 1;
        self.cursor.x = self.cursor.x_min;
        
        self.modified = true;
    }

    pub fn render(&mut self) -> Vec<ColourString> {
        let (terminal_x, terminal_y) = terminal::size().unwrap();
        let mut frame: Vec<ColourString> = vec![ColourString::new(String::from(" ".repeat(terminal_x as usize)), None); terminal_y as usize];
        let default = String::new();
        self.cursor.set_max(terminal_x, terminal_y-2);

        let mut line_number = 1 + self.cursor.y_offset;
        for i in 0..terminal_y {
            let mut line = self.highlighted_lines.get((i + self.cursor.y_offset)as usize).unwrap_or(&ColourString::new(default.clone(), None)).clone();
            line = line.skip(self.cursor.x_offset as usize);
            line.insert(0, " ".to_string(), None);

            let mut colour_line = ColourString::new(if line_number > self.lines.len() as u16 { "~".to_string() } else { line_number.to_string() }, Some(Info::new(Color::DarkGrey, Color::Reset, vec![])));
            colour_line.insert(0, " ".repeat(if line_number > self.lines.len() as u16 { 3 } else { 4-line_number.to_string().len() }), None);
            
            colour_line.push_colour_string(line);
            colour_line.truncate(terminal_x as usize);

            frame[i as usize].replace_range(0, terminal_x as usize, colour_line);
            line_number += 1;
        }
        frame
    }
}