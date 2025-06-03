//! Token definitions for ZLang
//! All the different pieces we can break code into

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    // Literals
    Number(f64),
    String(String),
    Identifier(String),
    
    // Gen Z Keywords
    Fr,        // true
    Cap,       // false
    Bet,       // let/assignment
    Sus,       // if
    Bussin,    // else
    LowkeySus, // else if
    NoSus,     // else (alternative)
    Periodt,   // end statement
    Flex,      // function
    Vibe,      // return
    Lowkey,    // while
    Grind,     // for
    Highkey,   // for (alternative)
    Bruh,      // print
    Slay,      // break
    NoChill,   // continue
    Ghost,     // continue (alternative)
    VibeCheck, // switch
    Manifest,  // try
    Caught,    // catch
    Drama,     // throw
    Frfr,      // finally
    
    // Operators
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Equal,
    EqualEqual,
    BangEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    And,
    Or,
    Bang,
    
    // Delimiters
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    Comma,
    Semicolon,
    Colon,
    In,
    
    // Special
    Newline,
    Eof,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub line: usize,
    #[allow(dead_code)]
    pub column: usize,
}

impl Token {
    pub fn new(token_type: TokenType, line: usize, column: usize) -> Self {
        Self {
            token_type,
            line,
            column,
        }
    }
}
