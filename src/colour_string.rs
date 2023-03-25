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

#[derive(Clone, Debug)]
pub struct Char {
    pub content: String,
    pub colour: Info,
}

#[derive(Clone, Debug)]
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

    // pub fn r_shift(&mut self, content: String, colour: Option<Info>) {
    //     self.content.remove(0);
    //     self.content.push(Char { content, colour: colour.unwrap_or(Info::new(Color::White, Color::Reset, vec![])) });
    // }

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
        for i in start..end {
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

}

impl fmt::Display for ColourString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.render())
    }
}