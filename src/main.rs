pub(crate) mod window;
pub(crate) mod cursor;
pub(crate) mod status_message;
pub(crate) mod home;
pub(crate) mod editor;

use crossterm::{
    terminal::{self, disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen}, 
    event::{EnableMouseCapture, DisableMouseCapture},
    Result, execute, cursor::{SavePosition, RestorePosition}
};

use std::io::stdout;
use std::fs::File;
use window::Window;

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
    let _ = WriteLogger::init(LevelFilter::Debug, Config::default(), File::create("debug.log").unwrap());
    let _disable_raw_mode = DisableRawMode;
    let mut window = Window::new();
    execute!(window.renderer, EnableMouseCapture, EnterAlternateScreen)?;
    if let Err(e) = window.ui() {
        println!("Error: {:?}\r", e);
    }

    Ok(())
}