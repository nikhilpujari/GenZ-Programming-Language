//! ZLang Lexer - Turns source code into tokens
//! This is where we break down the code into bite-sized pieces

use crate::token::{Token, TokenType};
use crate::error::ZLangError;

pub struct Lexer {
    source: Vec<char>,
    current: usize,
    line: usize,
    column: usize,
}

impl Lexer {
    pub fn new(source: &str) -> Self {
        Self {
            source: source.chars().collect(),
            current: 0,
            line: 1,
            column: 1,
        }
    }
    
    pub fn tokenize(&mut self) -> Result<Vec<Token>, ZLangError> {
        let mut tokens = Vec::new();
        
        while !self.is_at_end() {
            self.skip_whitespace();
            
            if self.is_at_end() {
                break;
            }
            
            let start_line = self.line;
            let start_column = self.column;
            
            match self.scan_token()? {
                Some(token_type) => {
                    tokens.push(Token::new(token_type, start_line, start_column));
                }
                None => {} // Skip whitespace and comments
            }
        }
        
        tokens.push(Token::new(TokenType::Eof, self.line, self.column));
        Ok(tokens)
    }
    
    fn scan_token(&mut self) -> Result<Option<TokenType>, ZLangError> {
        let c = self.advance();
        
        match c {
            // Single character tokens
            '(' => Ok(Some(TokenType::LeftParen)),
            ')' => Ok(Some(TokenType::RightParen)),
            '{' => Ok(Some(TokenType::LeftBrace)),
            '}' => Ok(Some(TokenType::RightBrace)),
            '[' => Ok(Some(TokenType::LeftBracket)),
            ']' => Ok(Some(TokenType::RightBracket)),
            ',' => Ok(Some(TokenType::Comma)),
            ';' => Ok(Some(TokenType::Semicolon)),
            ':' => Ok(Some(TokenType::Colon)),
            '+' => Ok(Some(TokenType::Plus)),
            '-' => Ok(Some(TokenType::Minus)),
            '*' => Ok(Some(TokenType::Star)),
            '/' => {
                if self.match_char('/') {
                    // Single line comment - skip to end of line
                    while self.peek() != '\n' && !self.is_at_end() {
                        self.advance();
                    }
                    Ok(None)
                } else {
                    Ok(Some(TokenType::Slash))
                }
            }
            '%' => Ok(Some(TokenType::Percent)),
            '!' => {
                if self.match_char('=') {
                    Ok(Some(TokenType::BangEqual))
                } else {
                    Ok(Some(TokenType::Bang))
                }
            }
            '=' => {
                if self.match_char('=') {
                    Ok(Some(TokenType::EqualEqual))
                } else {
                    Ok(Some(TokenType::Equal))
                }
            }
            '>' => {
                if self.match_char('=') {
                    Ok(Some(TokenType::GreaterEqual))
                } else {
                    Ok(Some(TokenType::Greater))
                }
            }
            '<' => {
                if self.match_char('=') {
                    Ok(Some(TokenType::LessEqual))
                } else {
                    Ok(Some(TokenType::Less))
                }
            }
            '&' => {
                if self.match_char('&') {
                    Ok(Some(TokenType::And))
                } else {
                    Err(ZLangError::new(&format!("Unexpected character '&' at line {}, that ain't it", self.line)))
                }
            }
            '|' => {
                if self.match_char('|') {
                    Ok(Some(TokenType::Or))
                } else {
                    Err(ZLangError::new(&format!("Unexpected character '|' at line {}, not the vibe", self.line)))
                }
            }
            '\n' => {
                self.line += 1;
                self.column = 1;
                Ok(Some(TokenType::Newline))
            }
            '"' => self.string(),
            _ => {
                if c.is_ascii_digit() {
                    self.number()
                } else if c.is_alphabetic() || c == '_' {
                    self.identifier()
                } else {
                    Err(ZLangError::new(&format!("Unexpected character '{}' at line {}, this ain't valid bestie", c, self.line)))
                }
            }
        }
    }
    
    fn string(&mut self) -> Result<Option<TokenType>, ZLangError> {
        let mut value = String::new();
        
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
                self.column = 1;
            }
            if self.peek() == '\\' {
                self.advance(); // consume backslash
                match self.advance() {
                    'n' => value.push('\n'),
                    't' => value.push('\t'),
                    'r' => value.push('\r'),
                    '\\' => value.push('\\'),
                    '"' => value.push('"'),
                    c => {
                        return Err(ZLangError::new(&format!("Invalid escape sequence '\\{}' at line {}, that's sus", c, self.line)));
                    }
                }
            } else {
                value.push(self.advance());
            }
        }
        
        if self.is_at_end() {
            return Err(ZLangError::new(&format!("Unterminated string at line {}, where's the closing quote bestie?", self.line)));
        }
        
        // Consume closing quote
        self.advance();
        
        Ok(Some(TokenType::String(value)))
    }
    
    fn number(&mut self) -> Result<Option<TokenType>, ZLangError> {
        while self.peek().is_ascii_digit() {
            self.advance();
        }
        
        // Look for decimal part
        if self.peek() == '.' && self.peek_next().is_ascii_digit() {
            self.advance(); // consume the '.'
            while self.peek().is_ascii_digit() {
                self.advance();
            }
        }
        
        let value: String = self.source[self.current - self.get_current_token_length()..self.current].iter().collect();
        let number = value.parse::<f64>().map_err(|_| {
            ZLangError::new(&format!("Invalid number '{}' at line {}, that's not how numbers work chief", value, self.line))
        })?;
        
        Ok(Some(TokenType::Number(number)))
    }
    
    fn identifier(&mut self) -> Result<Option<TokenType>, ZLangError> {
        while self.peek().is_alphanumeric() || self.peek() == '_' {
            self.advance();
        }
        
        let text: String = self.source[self.current - self.get_current_token_length()..self.current].iter().collect();
        
        // Check for multi-word keywords
        let multi_word_token = self.check_multi_word_keyword(&text)?;
        if let Some(token) = multi_word_token {
            return Ok(Some(token));
        }
        
        let token_type = match text.as_str() {
            "fr" => TokenType::Fr,
            "cap" => TokenType::Cap,
            "bet" => TokenType::Bet,
            "sus" => TokenType::Sus,
            "bussin" => TokenType::Bussin,
            "periodt" => TokenType::Periodt,
            "flex" => TokenType::Flex,
            "vibe" => TokenType::Vibe,
            "lowkey" => TokenType::Lowkey,
            "grind" => TokenType::Grind,
            "highkey" => TokenType::Highkey,
            "bruh" => TokenType::Bruh,
            "slay" => TokenType::Slay,
            "ghost" => TokenType::Ghost,
            "manifest" => TokenType::Manifest,
            "caught" => TokenType::Caught,
            "drama" => TokenType::Drama,
            "frfr" => TokenType::Frfr,
            "in" => TokenType::In,
            _ => TokenType::Identifier(text),
        };
        
        Ok(Some(token_type))
    }
    
    fn check_multi_word_keyword(&mut self, first_word: &str) -> Result<Option<TokenType>, ZLangError> {
        let _saved_pos = self.current;
        
        match first_word {
            "lowkey" => {
                if self.peek_word() == Some("sus".to_string()) {
                    self.consume_word();
                    Ok(Some(TokenType::LowkeySus))
                } else {
                    Ok(None)
                }
            }
            "no" => {
                let next_word = self.peek_word();
                if next_word == Some("sus".to_string()) {
                    self.consume_word();
                    Ok(Some(TokenType::NoSus))
                } else if next_word == Some("chill".to_string()) {
                    self.consume_word();
                    Ok(Some(TokenType::NoChill))
                } else {
                    Ok(None)
                }
            }
            "vibe" => {
                if self.peek_word() == Some("check".to_string()) {
                    self.consume_word();
                    Ok(Some(TokenType::VibeCheck))
                } else {
                    Ok(None)
                }
            }
            _ => Ok(None)
        }
    }
    
    fn peek_word(&self) -> Option<String> {
        let mut pos = self.current;
        
        // Skip whitespace
        while pos < self.source.len() && self.source[pos].is_whitespace() {
            pos += 1;
        }
        
        if pos >= self.source.len() {
            return None;
        }
        
        let start = pos;
        while pos < self.source.len() && (self.source[pos].is_alphanumeric() || self.source[pos] == '_') {
            pos += 1;
        }
        
        if pos > start {
            Some(self.source[start..pos].iter().collect())
        } else {
            None
        }
    }
    
    fn consume_word(&mut self) {
        // Skip whitespace
        while self.current < self.source.len() && self.source[self.current].is_whitespace() {
            if self.source[self.current] == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
            self.current += 1;
        }
        
        // Consume the word
        while self.current < self.source.len() && (self.source[self.current].is_alphanumeric() || self.source[self.current] == '_') {
            self.current += 1;
            self.column += 1;
        }
    }
    
    fn get_current_token_length(&self) -> usize {
        // This is a simple implementation - in a real lexer you'd track this better
        let mut length = 1;
        let mut pos = self.current - 1;
        
        while pos > 0 {
            let c = self.source[pos - 1];
            if c.is_whitespace() || self.is_operator_char(c) {
                break;
            }
            length += 1;
            pos -= 1;
        }
        
        length
    }
    
    fn is_operator_char(&self, c: char) -> bool {
        matches!(c, '(' | ')' | '{' | '}' | '[' | ']' | ',' | ';' | ':' | '+' | '-' | '*' | '/' | '%' | '!' | '=' | '>' | '<' | '&' | '|')
    }
    
    fn skip_whitespace(&mut self) {
        while !self.is_at_end() {
            match self.peek() {
                ' ' | '\r' | '\t' => {
                    self.advance();
                }
                _ => break,
            }
        }
    }
    
    fn advance(&mut self) -> char {
        let c = self.source[self.current];
        self.current += 1;
        self.column += 1;
        c
    }
    
    fn match_char(&mut self, expected: char) -> bool {
        if self.is_at_end() || self.source[self.current] != expected {
            false
        } else {
            self.current += 1;
            self.column += 1;
            true
        }
    }
    
    fn peek(&self) -> char {
        if self.is_at_end() {
            '\0'
        } else {
            self.source[self.current]
        }
    }
    
    fn peek_next(&self) -> char {
        if self.current + 1 >= self.source.len() {
            '\0'
        } else {
            self.source[self.current + 1]
        }
    }
    
    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }
}
