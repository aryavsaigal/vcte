use std::collections::{HashMap, HashSet};
use crossterm::style::Color;
use rand::{self, seq::SliceRandom};
use crate::colour_string::{ColourString, Info, Char};

pub struct SyntaxHighlighter {
    pub colour_key: HashMap<String, Color>,
}

pub fn rgb(r: u8, g: u8, b: u8) -> Color {
    Color::Rgb { r, g, b }
}

impl SyntaxHighlighter {
    pub fn new() -> Self {
        Self {
            colour_key: HashMap::new(),
        }
    }

    pub fn init(&mut self, lines: Vec<String>) {
        self.colour_key = HashMap::new();
        for line in lines {
            for word in HashSet::<&str>::from_iter(line.split(&[' ', '.', ':'])) {
                self.colour_key.entry(word.trim().to_string()).or_insert_with(|| {
                    if word.contains("()") {
                        Color::AnsiValue(120)
                    }
                    else if word.contains(|c: char| c.is_numeric()) {
                        Color::AnsiValue(222)
                    }
                    else if ["(", ")", "{", "}", "[", "]"].contains(&word) {
                        Color::AnsiValue(103)
                    }
                    else if ["+", "-", "*", "/", "%", "=", ">", "<", "!"].iter().any(|&c| word.contains(c)) {
                        Color::AnsiValue(212)
                    }
                    else {
                        let mut rng = rand::thread_rng();
                        *[rgb(248, 248, 242), rgb(139, 233, 253), rgb(80, 250, 123),rgb(255, 184, 108),rgb(255, 121, 198),rgb(189, 147, 249),rgb(255, 85, 85),rgb(241, 250, 140)].choose(&mut rng).unwrap()
                    }
                });
            }
        }
    }

    pub fn highlight(&mut self, line: String) -> ColourString {
        let mut highlighted_line = ColourString::new(line.clone(), None);
        
        let mut words: Vec<&str> = HashSet::<&str>::from_iter(line.split(&[' ', '.', ':'])).into_iter().collect();
        words.sort_by(|a, b| a.len().cmp(&b.len()));
        for word in words {
            if let Some(colour) = self.colour_key.get(word.trim()) {
                highlighted_line.set_colour_pattern(word.to_string(), Info::new(*colour, Color::Reset, vec![]));
            }
            else {
                let mut rng = rand::thread_rng();
                self.colour_key.insert(word.trim().to_string(), *[rgb(248, 248, 242), rgb(139, 233, 253), rgb(80, 250, 123),rgb(255, 184, 108),rgb(255, 121, 198),rgb(189, 147, 249),rgb(255, 85, 85),rgb(241, 250, 140)].choose(&mut rng).unwrap());
                highlighted_line.set_colour_pattern(word.to_string(), Info::new(self.colour_key.get(word.trim()).unwrap().clone(), Color::Reset, vec![]));
            }
        }
        let mut start = None;
        let mut index = 0;

        let mut indices = Vec::new();

        for char in highlighted_line.get_content() {
            if char.content == "\"" {
                if start.is_none() {
                    start = Some(index);
                }
                else {
                    indices.push((start.unwrap(), index+1));
                    start = None;
                }
            }
            index += 1;
        }

        for (start, end) in indices {
            highlighted_line.set_colour(Info::new(Color::AnsiValue(229), Color::Reset, vec![]), start, end)
        }

        highlighted_line
    }

}