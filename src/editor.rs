use crate::window::Window;
use crossterm::{
    terminal,
    queue,
    Result,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    cursor,
    event::KeyCode
};
use std::{io::Write, path::Path, fs};

pub enum Mode {
    View, 
    Insert
}

pub fn editor(window: &mut Window) -> Result<()> {
    let (terminal_x, terminal_y) = terminal::size()?;
    
    let longest_string = window.open_file.iter().max_by_key(|x| x.len()).unwrap_or(&String::new()).to_string();

    if let Some(inserted_char) = &window.inserted_char {
        if inserted_char.backspace {
            let x = inserted_char.x as usize + window.cursor.offset_x as usize;
            let y = inserted_char.y as usize + window.cursor.offset_y as usize;
            let line_len = window.open_file[y].len();
            if x >= line_len {
                window.open_file[y].push_str(" ".repeat(x - line_len).as_str());
            }
    
            window.open_file[y].remove(x-1);
            window.inserted_char = None;
            window.cursor.move_cursor(KeyCode::Left, &mut window.renderer)?;
        }
        else {
            let x = inserted_char.x as usize + window.cursor.offset_x as usize;
            let y = inserted_char.y as usize + window.cursor.offset_y as usize;
            let line_len = window.open_file[y].len();
            if x > line_len {
                window.open_file[y].push_str(" ".repeat(x - line_len).as_str());
            }
    
            window.open_file[y].insert(x, inserted_char.character);
            window.inserted_char = None;
            window.cursor.move_cursor(KeyCode::Right, &mut window.renderer)?;
        }
    }

    queue!(window.renderer, cursor::MoveTo(0,0))?;

    for i in 0..terminal_y-1 {
        if (i + window.cursor.offset_y) >= window.open_file.len() as u16 {
            queue!(window.renderer, terminal::Clear(terminal::ClearType::CurrentLine))?;
            write!(window.renderer, "\r\n")?;
        }
        else {
            if (longest_string[window.cursor.offset_x as usize..].len()+1) - terminal_x as usize == 0 {
                window.cursor.offset_x -= 1;
            }
            let mut line = &mut window.open_file[i as usize + window.cursor.offset_y as usize];
            if window.cursor.offset_x >= line.len() as u16 {
                line.insert_str(0, " ".repeat((window.cursor.offset_x - line.len() as u16) as usize).as_str());
            }
            let mut line  = line[window.cursor.offset_x as usize..].to_string();
            line.truncate(terminal_x as usize);
            queue!(window.renderer, terminal::Clear(terminal::ClearType::CurrentLine))?;
            write!(window.renderer, "{}\r\n", line)?;
        }
    }
    Ok(())
}

pub fn open_file(path: &Path) -> Vec<String> {
    fs::read_to_string(path)
        .expect("Something went wrong reading the file")
        .lines()
        .map(|s| s.to_string())
        .collect()
}

pub fn save_file(path: &Path, file: &Vec<String>) -> Result<()> {
    fs::write(path, file.join("\n"))
}