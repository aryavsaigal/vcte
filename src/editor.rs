use crate::{window::{Window, File}, status_message};
use crossterm::{
    terminal,
    queue,
    Result,
    style::{Stylize},
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
    let open_files_tab = window.open_files.clone();
    let mut file = &mut window.open_files[window.current_file_index];
    let longest_string = file.content.iter().max_by_key(|x| x.len()).unwrap_or(&String::new()).to_string();
    if let Some(inserted_char) = &window.inserted_char {
        let x = inserted_char.x as usize + file.offset_x as usize;
        let y = inserted_char.y as usize + file.offset_y as usize - 2; // -2 because 2 lines on top
        if y >= file.content.len() {
            for _ in 0..((y+1)-file.content.len()) {
                file.content.push(String::new());
            }
        }

        let line_len = file.content[y].len();

        if x >= line_len {
            file.content[y].push_str(" ".repeat(x - line_len).as_str());
        }

        match inserted_char.character {
            KeyCode::Esc => {
                window.editor_mode = Mode::View;
                queue!(window.renderer, cursor::SetCursorStyle::DefaultUserShape)?;
            },
            KeyCode::Char(c) => {
                file.content[y].insert(x, c);
                window.cursor.move_cursor(KeyCode::Right, &mut window.renderer, file)?;
            },
            KeyCode::Backspace => {
                if x == 0 {
                    if y  == 0 {
                        return Ok(());
                    }
                    let line = file.content[y].clone();
                    let prev_line_len = file.content[y-1].len();
                    file.content[y-1].push_str(line.as_str());
                    file.content.remove(y);
                    window.cursor.move_to(prev_line_len as u16, (y+1) as u16, &mut window.renderer, file)?;
                }
                else {
                    file.content[y].remove(x-1);
                    window.cursor.move_cursor(KeyCode::Left, &mut window.renderer, file)?;
                }
            },
            KeyCode::Enter => {
                let new_line = file.content[y].split_off(x);
                file.content.insert(y+1, new_line);
                window.cursor.move_cursor(KeyCode::Down, &mut window.renderer, file)?;
            },
            direction @ (KeyCode::Up | KeyCode::Down | KeyCode::Left | KeyCode::Right) => {
                window.cursor.move_cursor(direction, &mut window.renderer, file)?;
                window.status_message.mode = status_message::Mode::Disabled;
            },
            _ => {}
        }
        window.inserted_char = None;
    }

    queue!(window.renderer, cursor::MoveTo(0,0))?;
    queue!(window.renderer, terminal::Clear(terminal::ClearType::CurrentLine))?;
    write!(window.renderer, "{}\r\n", open_files_tab.iter().enumerate().map(|(i, x)| {
        let path = Path::new(&x.path).file_name().unwrap().to_str().unwrap();
        if i == window.current_file_index {
            format!("{} ", path).bold().cyan().to_string()
        }
        else {
            format!("{} ", path).to_string()
        }
    }).collect::<Vec<String>>().join(" | "))?;
    queue!(window.renderer, terminal::Clear(terminal::ClearType::CurrentLine))?;
    write!(window.renderer, "{}\r\n", "-".repeat(terminal_x as usize))?;
    

    for i in 0..terminal_y-4 { // -3 because im rendering like 2 lines on top and 1 line on bottom
        if (i + file.offset_y) >= file.content.len() as u16 {
            queue!(window.renderer, terminal::Clear(terminal::ClearType::CurrentLine))?;
            write!(window.renderer, "\r\n")?;
        }
        else {
            if (longest_string[file.offset_x as usize..].len()+2).saturating_sub(terminal_x as usize) == 0 {
                file.offset_x = file.offset_x.saturating_sub(1);
            }
            let line = &mut file.content[i as usize + file.offset_y as usize];
            if file.offset_x >= line.len() as u16 {
                line.insert_str(0, " ".repeat((file.offset_x - line.len() as u16) as usize).as_str());
            }
            let mut line  = line[file.offset_x as usize..].to_string();
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

pub fn save_file(file: &File) -> Result<()> {
    fs::write(&file.path, file.content.join("\n"))
}