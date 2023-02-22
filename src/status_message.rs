pub enum Mode {
    Enabled, 
    Disabled,
    Error
}

pub struct StatusMessage {
    pub mode: Mode,
    pub command: String,
    pub error: String,
}

impl StatusMessage {
    pub fn new() -> Self {
        Self {
            mode: Mode::Disabled,
            command: String::new(),
            error: String::new(),
        }
    }
}