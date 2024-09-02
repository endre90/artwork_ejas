use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct ErrorMsg {
    info: String,
}

impl ErrorMsg {
    pub fn new(info: &str) -> ErrorMsg {
        ErrorMsg {
            info: info.to_string(),
        }
    }
}

impl fmt::Display for ErrorMsg {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.info)
    }
}

impl Error for ErrorMsg {
    fn description(&self) -> &str {
        &self.info
    }
}