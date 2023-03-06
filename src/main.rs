pub(crate) mod window;
pub(crate) mod cursor;
pub(crate) mod status_message;
pub(crate) mod home;
pub(crate) mod editor;

use crossterm::{
    terminal::{ disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen}, 
    event::{EnableMouseCapture, DisableMouseCapture},
    Result, execute,
};

use std::io::{stdout, Write};
use std::fs::{self, File};
use std::path::Path;
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

fn create_config_file() -> Result<()> {
    let path = Path::new("~/.vcte/config.toml");
    if !path.exists() {
        let prefix = path.parent().unwrap_or(Path::new("~/.vcte/"));
        fs::create_dir_all(prefix)?;
        let mut file = File::create(path)?;
        file.write_all(b"[general]")?;
    }
    Ok(())
}

fn main() -> Result<()> {
    enable_raw_mode()?;
    create_config_file()?;
    let _ = WriteLogger::init(LevelFilter::Debug, Config::default(), File::create("debug.log").unwrap());
    let _disable_raw_mode = DisableRawMode;
    let mut window = Window::new();
    execute!(window.renderer, EnableMouseCapture, EnterAlternateScreen)?;
    if let Err(e) = window.ui() {
        println!("Error: {:?}\r", e);
    }

    Ok(())
}