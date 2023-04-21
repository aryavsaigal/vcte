use std::collections::HashMap;
use crossterm::style::Color;
use rand::{self, Rng, seq::SliceRandom};
use crate::colour_string::{ColourString, Info};

pub struct SyntaxHighlighter {
    pub colour_key: HashMap<String, Color>,
}

pub fn color_from_hex(hex: &str) -> Color {
    let mut hex = hex.to_string();
    if hex.len() == 3 {
        hex = format!("{}{}{}{}{}{}", hex.chars().nth(0).unwrap(), hex.chars().nth(0).unwrap(), hex.chars().nth(1).unwrap(), hex.chars().nth(1).unwrap(), hex.chars().nth(2).unwrap(), hex.chars().nth(2).unwrap());
    }
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap();
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap();
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap();
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
            for word in line.split_whitespace() {
                self.colour_key.entry(word.trim().to_string()).or_insert_with(|| {
                    let mut rng = rand::thread_rng();
                    Color::Rgb {
                        r: rng.gen_range(0..=255),
                        g: rng.gen_range(0..=255),
                        b: rng.gen_range(0..=255),
                    }
                });
            }
        }
    }

    pub fn highlight(&mut self, line: String) -> ColourString {
        let mut highlighted_line = ColourString::new(line.clone(), None);
        for word in line.split(" ") {
            if let Some(colour) = self.colour_key.get(word.trim()) {
                highlighted_line.set_colour_pattern(word.to_string(), Info::new(*colour, Color::Reset, vec![]));
            }
            else {
                let mut rng = rand::thread_rng();
                self.colour_key.insert(word.trim().to_string(), Color::Rgb {
                    r: rng.gen_range(0..=255),
                    g: rng.gen_range(0..=255),
                    b: rng.gen_range(0..=255),
                });
                highlighted_line.set_colour_pattern(word.to_string(), Info::new(self.colour_key.get(word.trim()).unwrap().clone(), Color::Reset, vec![]));
            }
        }
        highlighted_line
    }

}