pub(crate) mod window;
pub(crate) mod colour_string;
pub(crate) mod home;
pub(crate) mod command_palette;
pub(crate) mod file_explorer;
pub(crate) mod cursor;
pub(crate) mod editor;
pub(crate) mod status_bar;

use crossterm::{
    terminal::{ disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen}, 
    event::{EnableMouseCapture, DisableMouseCapture},
    Result, execute,
};

use window::Window;

use std::{io::{stdout}, fs::File};

#[macro_use] extern crate log;
extern crate simplelog;
use simplelog::*;

struct DisableRawMode;

impl Drop for DisableRawMode {
    fn drop(&mut self) {
        disable_raw_mode().expect("failed to disable raw mode");
        execute!(stdout(), DisableMouseCapture, LeaveAlternateScreen).expect("failed to leave alternate screen");
    }
}

fn main() -> Result<()> {
    enable_raw_mode()?;

    let _disable_raw_mode = DisableRawMode;
    let _ = WriteLogger::init(LevelFilter::Debug, Config::default(), File::create("debug.log").unwrap());
    let mut window = Window::new();

    execute!(stdout(), EnableMouseCapture, EnterAlternateScreen)?;
    if let Err(e) = window.render() {
        drop(_disable_raw_mode);
        println!("Error: {:?}\r", e);
    }

    Ok(())
}