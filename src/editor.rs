use crate::window::Window;
use crossterm::{
    terminal,
    queue,
    Result,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    cursor
};
use std::{io::Write, path::Path, fs};

pub fn editor(window: &mut Window) -> Result<()> {
    let (terminal_x, terminal_y) = terminal::size()?;
    let mut file = open_file(&Path::new(&window.open_files[0]));
    queue!(window.renderer, cursor::MoveTo(0,0))?;

    for i in 0..terminal_y-1 {
        if (i + window.cursor.offset_y) >= file.len() as u16 {
            write!(window.renderer, "\r\n")?;
            queue!(window.renderer, terminal::Clear(terminal::ClearType::UntilNewLine))?;
        }
        else {
            let mut line = &mut file[i as usize + window.cursor.offset_y as usize];

            if window.cursor.offset_x > line.len() as u16 {
                window.cursor.offset_x = line.len() as u16;
            }

            // let mut line  = line[window.cursor.offset_x as usize..].to_string();
            line.truncate(terminal_x as usize);

            write!(window.renderer, "{}\r\n", line)?;
            queue!(window.renderer, terminal::Clear(terminal::ClearType::UntilNewLine))?;
        }
    }
    Ok(())
}

fn open_file(path: &Path) -> Vec<String> {
    fs::read_to_string(path)
        .expect("Something went wrong reading the file")
        .lines()
        .map(|s| s.to_string())
        .collect()
}