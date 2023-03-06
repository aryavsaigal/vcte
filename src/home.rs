use crate::window::Window;
use crossterm::{
    terminal,
    queue,
    Result,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    cursor
};
use std::io::Write;

pub fn home(window: &mut Window) -> Result<()> {
    let (terminal_x, terminal_y) = terminal::size()?;
    queue!(window.renderer, cursor::MoveTo(0,0))?;
    write!(window.renderer, "\n")?;
    queue!(window.renderer, terminal::Clear(terminal::ClearType::UntilNewLine))?;

    for i in 0..terminal_y-3 {
        write!(window.renderer, "~")?;

        if i == terminal_y / 3 {
            let mut welcome = format!("vcte (very cool text editor) v{}", env!("CARGO_PKG_VERSION"));
            let padding = ((terminal_x - welcome.len() as u16) / 2) - 1;

            welcome.truncate(terminal_x as usize);

            if padding > 0 {
                write!(window.renderer, "{}", &" ".repeat(padding as usize))?;
            }
            write!(window.renderer, "{}", welcome.as_str())?;
        }
        if i != terminal_y {
            write!(window.renderer, "\r\n")?;
        }
        queue!(window.renderer, terminal::Clear(terminal::ClearType::UntilNewLine))?;
    };
    Ok(())
}