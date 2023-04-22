use unicode_segmentation::UnicodeSegmentation;

use crossterm::{
    style::{Color, Attribute, Stylize},
};

use std::fmt;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Info {
    pub foreground: Color,
    pub background: Color,
    pub attributes: Vec<Attribute>,
}

impl Info {
    pub fn new(foreground: Color, background: Color, attributes: Vec<Attribute>) -> Self {
        Self {
            foreground,
            background,
            attributes,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Char {
    pub content: String,
    pub colour: Info,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ColourString {
    content: Vec<Char>,
}

impl ColourString {
    pub fn new(content: String, colour: Option<Info>) -> Self {
        let content: Vec<String> = content.graphemes(true).map(|x| x.to_string()).collect();
        match colour {
            Some(colour) => Self {
                content: content.into_iter().map(|x| Char { content: x, colour: colour.clone() }).collect(),
            },
            None => Self {
                content: content.into_iter().map(|x| Char { content: x, colour: Info::new(Color::White, Color::Reset, vec![]) }).collect(),
            },
        }
    }

    pub fn get_content(&self) -> &Vec<Char> {
        &self.content
    }

    pub fn truncate(&mut self, len: usize) {
        self.content.truncate(len);
    }

    pub fn l_shift(&mut self, content: String, colour: Option<Info>) {
        self.content.pop();
        self.content.insert(0, Char { content, colour: colour.unwrap_or(Info::new(Color::White, Color::Reset, vec![])) });
    }

    pub fn skip(&mut self, n: usize) -> ColourString {
        ColourString {
            content: self.content.split_at(n).1.to_vec(),
        }
    }

    // pub fn r_shift(&mut self, content: String, colour: Option<Info>) {
    //     self.content.remove(0);
    //     self.content.push(Char { content, colour: colour.unwrap_or(Info::new(Color::White, Color::Reset, vec![])) });
    // }

    pub fn set_colour_pattern(&mut self, pattern: String, colour: Info) {
        let indices = self.content.iter_mut().map(|x| x.content.to_string()).collect::<Vec<String>>().join("");
        let indices = indices.match_indices(&pattern).collect::<Vec<_>>();

        for i in indices {
            self.set_colour(colour.clone(), i.0, i.0 + pattern.graphemes(true).count());
        }
    }

    pub fn replace_char(&mut self, pattern: String, replacement: String, colour: Option<Info>) {
        let mut new_content = Vec::new();
        let colour = colour.unwrap_or(Info::new(Color::White, Color::Reset, vec![]));
        for i in 0..self.content.len() {
            if self.content[i].content == pattern {
                new_content.push(Char { content: replacement.clone(), colour: colour.clone() });
            }
            else {
                new_content.push(self.content[i].clone());
            }
        }
        self.content = new_content;
    }

    pub fn replace(&mut self, pattern: String, replacement: String, colour: Option<Info>) {
        let colour = colour.unwrap_or(Info::new(Color::White, Color::Reset, vec![]));
        let pattern = pattern.graphemes(true).collect::<Vec<&str>>();
        let mut indices = vec![];
        let mut start: Option<usize> = None;
        for (i, c) in self.content.iter().enumerate() {
            if let None = start {
                if c.content == pattern[0 as usize] {
                    start = Some(i);
                    if pattern.len() == 1 {
                        indices.push(i);
                        start = None;
                    }
                }
            }
            else if let Some(s) = start {
                if i-s >= pattern.len()-1 {
                    indices.push(s);
                    if c.content == pattern[0 as usize] {
                        start = Some(i);
                        if pattern.len() == 1 {
                            indices.push(i);
                            start = None;
                        }
                    }
                    else {
                        start = None;
                    }
                }
                else if c.content != pattern[i-s] {
                    start = None;
                }
            }
        }

        for i in indices {
            self.content.splice(i..((i + pattern.join("").graphemes(true).count()) as usize).clamp(0, self.content.len()), replacement.graphemes(true).map(|x| Char { content: x.to_string(), colour: colour.clone() }).collect::<Vec<Char>>());
        }

    }

    pub fn pad(&mut self, len: usize, content: String, colour: Option<Info>) {
        let colour = colour.unwrap_or(Info::new(Color::White, Color::Reset, vec![]));
        while self.content.len() < len {
            self.content.push(Char { content: content.clone(), colour: colour.clone() });
        }
    }

    pub fn set_background(&mut self, colour: Color) {
        for i in 0..self.content.len() {
            self.content[i].colour.background = colour;
        }
    }

    pub fn push_colour_string(&mut self, other: ColourString) {
        self.content.extend(other.content);
    }

    pub fn push_str(&mut self, content: &str, colour: Option<Info>) {
        let content = content.graphemes(true).map(|x| x.to_string()).collect::<Vec<String>>();
        let colour = colour.unwrap_or(Info::new(Color::White, Color::Reset, vec![]));
        self.content.extend(content.into_iter().map(|x| Char { content: x, colour: colour.clone() }));
    }

    pub fn replace_range(&mut self, start: usize, end: usize, content: ColourString) {
        self.content.splice(start..end, content.content);
    }

    pub fn set_colour(&mut self, colour: Info, start: usize, end: usize) {
        for i in start..end.clamp(0, self.content.len()) {
            self.content[i].colour = colour.clone();
        }
    }

    // pub fn set_colour_pattern(&mut self, colour: Info, pattern: String) {
    //     for i in 0..self.content.len() {
    //         if self.content[i].content == pattern {
    //             self.content[i].colour = colour.clone();
    //         }
    //     }
    // }

    pub fn insert(&mut self, index: usize, content: String, colour: Option<Info>) {
        let colour = colour.unwrap_or(Info::new(Color::White, Color::Reset, vec![]));
        let content = content.graphemes(true).map(|x| x.to_string()).collect::<Vec<String>>();
        self.content.splice(index..index, content.into_iter().map(|x| Char { content: x, colour: colour.clone() }));
    }

    pub fn render(&self) -> String {
        let mut output = String::new();
        let mut group = String::new();
        for (i, c) in self.content.iter().enumerate() {
            let info = &self.content[i].colour;
            if info == &self.content[i.saturating_sub(1)].colour {
                group.push_str(&c.content);
            }
            else {
                let mut rendered_group = group.with(self.content[i.saturating_sub(1)].colour.foreground).on(self.content[i.saturating_sub(1)].colour.background).to_string();
                for attribute in &self.content[i.saturating_sub(1)].colour.attributes {
                    rendered_group = rendered_group.attribute(*attribute).to_string();
                }
                output.push_str(&rendered_group);
                group = c.content.clone();
            }
        }
        let mut rendered_group = group.with(self.content[self.content.len() - 1].colour.foreground).on(self.content[self.content.len() - 1].colour.background).to_string();
        for attribute in &self.content[self.content.len() - 1].colour.attributes {
            rendered_group = rendered_group.attribute(*attribute).to_string();
        }
        output.push_str(&rendered_group);

        output
    }

    pub fn render_vector(vector: Vec<ColourString>) -> String {
        let mut output = ColourString::new(String::new(), None);
        for mut line in vector {
            output.push_colour_string(line.clone());
            line.push_str("\r\n", Some(line.content.last().unwrap_or(&Char { content: String::new(), colour: Info::new(Color::White, Color::Reset, vec![]) }).colour.clone()));
        }
        output.render().trim_end().to_string()
    }

    pub fn parse_ansi_string(string: String) -> ColourString {
        let mut output = ColourString::new(String::new(), None);
        let mut current_colour = Info::new(Color::White, Color::Reset, vec![]);
        let mut escape_code_pos: Option<usize> = None;

        for (i, c) in string.char_indices() {
            if c == '\x1b' {
                escape_code_pos = Some(i);
            }
            else if escape_code_pos.is_some() {
                if c == 'm' {
                    let sequence = &string[escape_code_pos.unwrap()..i + 1];
                    escape_code_pos = None;
                    let sequence = sequence.split(';').skip(2).collect::<Vec<&str>>();
                    let r = sequence[0].parse().unwrap_or(0);
                    let g = sequence[1].parse().unwrap_or(0);
                    let b = sequence[2].replace("m", "").parse().unwrap_or(0);
                    current_colour = Info::new(Color::Rgb { r, g, b }, Color::Reset, vec![]);
                }
            }
            else {
                output.push_str(&c.to_string(), Some(current_colour.clone()));
            }
        }
        output
    }

    pub fn join(vector: Vec<ColourString>, separator: ColourString) -> ColourString {
        let mut output = ColourString::new(String::new(), None);
        output.push_colour_string(vector.first().unwrap_or(&ColourString::new("".to_string(), None)).clone());
        for line in vector.iter().skip(1) {
            output.push_colour_string(separator.clone());
            output.push_colour_string(line.clone());
        }
        output
    }

}

impl fmt::Display for ColourString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.render())
    }
}
