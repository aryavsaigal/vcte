pub enum Mode {
    Enabled, 
    Disabled,
    Error,
    Success
}

pub struct StatusMessage {
    pub mode: Mode,
    pub command: String,
    pub error: String,
    pub success: String,
    pub quick_command: String
}

impl StatusMessage {
    pub fn new() -> Self {
        Self {
            mode: Mode::Disabled,
            command: String::new(),
            error: String::new(),
            success: String::new(),
            quick_command: String::new()
        }
    }
}