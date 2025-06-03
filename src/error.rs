//! Error handling for ZLang with Gen Z vibes
//! When things go wrong, we gotta tell the user in their language 💯

use std::fmt;

#[derive(Debug, Clone)]
pub struct ZLangError {
    pub message: String,
}

impl ZLangError {
    pub fn new(message: &str) -> Self {
        Self {
            message: message.to_string(),
        }
    }
}

impl fmt::Display for ZLangError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for ZLangError {}
