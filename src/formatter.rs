//! ZLang Code Formatter - Making your code look fresh! ✨
//! Automatically formats ZLang code with proper indentation and spacing

use crate::lexer::Lexer;
use crate::token::{Token, TokenType};
use crate::error::ZLangError;

#[allow(dead_code)]
pub struct Formatter {
    tokens: Vec<Token>,
    current: usize,
    output: String,
    indent_level: usize,
    indent_size: usize,
}

#[allow(dead_code)]
impl Formatter {
    pub fn new() -> Self {
        Self {
            tokens: Vec::new(),
            current: 0,
            output: String::new(),
            indent_level: 0,
            indent_size: 4, // 4 spaces per indent level
        }
    }
    
    pub fn format(&mut self, source: &str) -> Result<String, ZLangError> {
        // Tokenize the source code
        let mut lexer = Lexer::new(source);
        self.tokens = lexer.tokenize()?;
        self.current = 0;
        self.output.clear();
        self.indent_level = 0;
        
        self.format_tokens()?;
        
        Ok(self.output.trim().to_string() + "\n")
    }
    
    fn format_tokens(&mut self) -> Result<(), ZLangError> {
        while !self.is_at_end() {
            self.format_statement()?;
        }
        Ok(())
    }
    
    fn format_statement(&mut self) -> Result<(), ZLangError> {
        // Skip multiple newlines
        while self.match_token(&TokenType::Newline) {
            // Only add one newline max
            if !self.output.ends_with('\n') {
                self.output.push('\n');
            }
        }
        
        if self.is_at_end() {
            return Ok(());
        }
        
        // Add proper indentation
        self.add_indent();
        
        match &self.peek().token_type {
            TokenType::Flex => self.format_function()?,
            TokenType::Bet => self.format_variable_declaration()?,
            TokenType::Sus => self.format_if_statement()?,
            TokenType::Lowkey => self.format_while_statement()?,
            TokenType::Highkey => self.format_for_statement()?,
            TokenType::Bruh => self.format_print_statement()?,
            TokenType::LeftBrace => self.format_block()?,
            TokenType::Vibe => self.format_return_statement()?,
            TokenType::Slay | TokenType::Ghost => {
                self.add_token();
                self.consume_statement_end();
            }
            _ => self.format_expression_statement()?,
        }
        
        Ok(())
    }
    
    fn format_function(&mut self) -> Result<(), ZLangError> {
        self.add_token(); // flex
        self.add_space();
        self.add_token(); // function name
        
        self.add_token(); // (
        self.format_parameter_list()?;
        self.add_token(); // )
        self.add_space();
        
        self.format_block()?;
        Ok(())
    }
    
    fn format_parameter_list(&mut self) -> Result<(), ZLangError> {
        if !self.check(&TokenType::RightParen) {
            loop {
                self.add_token(); // parameter name
                
                if !self.match_token(&TokenType::Comma) {
                    break;
                }
                self.output.push_str(", ");
            }
        }
        Ok(())
    }
    
    fn format_variable_declaration(&mut self) -> Result<(), ZLangError> {
        self.add_token(); // bet
        self.add_space();
        self.add_token(); // variable name
        
        if self.match_token(&TokenType::Equal) {
            self.output.push_str(" = ");
            self.format_expression()?;
        }
        
        self.consume_statement_end();
        Ok(())
    }
    
    fn format_if_statement(&mut self) -> Result<(), ZLangError> {
        self.add_token(); // sus
        self.add_space();
        
        self.add_token(); // (
        self.format_expression()?;
        self.add_token(); // )
        self.add_space();
        
        self.format_statement_or_block()?;
        
        if self.match_token(&TokenType::Bussin) {
            self.output.push_str(" bussin ");
            self.format_statement_or_block()?;
        }
        
        Ok(())
    }
    
    fn format_while_statement(&mut self) -> Result<(), ZLangError> {
        self.add_token(); // lowkey
        self.add_space();
        
        self.add_token(); // (
        self.format_expression()?;
        self.add_token(); // )
        self.add_space();
        
        self.format_statement_or_block()?;
        Ok(())
    }
    
    fn format_for_statement(&mut self) -> Result<(), ZLangError> {
        self.add_token(); // highkey
        self.add_space();
        
        self.add_token(); // (
        self.add_token(); // variable
        self.add_space();
        self.add_token(); // in
        self.add_space();
        self.format_expression()?;
        self.add_token(); // )
        self.add_space();
        
        self.format_statement_or_block()?;
        Ok(())
    }
    
    fn format_print_statement(&mut self) -> Result<(), ZLangError> {
        self.add_token(); // bruh
        self.add_space();
        self.format_expression()?;
        self.consume_statement_end();
        Ok(())
    }
    
    fn format_return_statement(&mut self) -> Result<(), ZLangError> {
        self.add_token(); // vibe
        
        if !self.check(&TokenType::Semicolon) && !self.check(&TokenType::Newline) {
            self.add_space();
            self.format_expression()?;
        }
        
        self.consume_statement_end();
        Ok(())
    }
    
    fn format_expression_statement(&mut self) -> Result<(), ZLangError> {
        self.format_expression()?;
        self.consume_statement_end();
        Ok(())
    }
    
    fn format_statement_or_block(&mut self) -> Result<(), ZLangError> {
        if self.check(&TokenType::LeftBrace) {
            self.format_block()
        } else {
            self.output.push('\n');
            self.indent_level += 1;
            self.format_statement()?;
            self.indent_level -= 1;
            Ok(())
        }
    }
    
    fn format_block(&mut self) -> Result<(), ZLangError> {
        self.add_token(); // {
        self.output.push('\n');
        self.indent_level += 1;
        
        while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
            self.format_statement()?;
        }
        
        self.indent_level -= 1;
        self.add_indent();
        self.add_token(); // }
        
        Ok(())
    }
    
    fn format_expression(&mut self) -> Result<(), ZLangError> {
        // Simple expression formatting - could be enhanced further
        while !self.is_statement_end() && !self.is_at_end() {
            match &self.peek().token_type {
                TokenType::LeftParen => {
                    self.add_token();
                    self.format_expression_until(&TokenType::RightParen)?;
                    self.add_token();
                }
                TokenType::LeftBracket => {
                    self.add_token();
                    self.format_array_elements()?;
                    self.add_token();
                }
                TokenType::LeftBrace => {
                    self.add_token();
                    self.format_object_elements()?;
                    self.add_token();
                }
                TokenType::Plus | TokenType::Minus | TokenType::Star | TokenType::Slash |
                TokenType::Equal | TokenType::EqualEqual | TokenType::BangEqual |
                TokenType::Greater | TokenType::GreaterEqual | TokenType::Less | TokenType::LessEqual |
                TokenType::And | TokenType::Or => {
                    self.output.push(' ');
                    self.add_token();
                    self.output.push(' ');
                }
                _ => self.add_token(),
            }
        }
        Ok(())
    }
    
    fn format_expression_until(&mut self, end_token: &TokenType) -> Result<(), ZLangError> {
        while !self.check(end_token) && !self.is_at_end() {
            self.format_expression()?;
            if self.match_token(&TokenType::Comma) {
                self.output.push_str(", ");
            }
        }
        Ok(())
    }
    
    fn format_array_elements(&mut self) -> Result<(), ZLangError> {
        if !self.check(&TokenType::RightBracket) {
            loop {
                self.format_expression()?;
                if !self.match_token(&TokenType::Comma) {
                    break;
                }
                self.output.push_str(", ");
            }
        }
        Ok(())
    }
    
    fn format_object_elements(&mut self) -> Result<(), ZLangError> {
        if !self.check(&TokenType::RightBrace) {
            loop {
                self.add_token(); // key
                self.add_token(); // :
                self.output.push(' ');
                self.format_expression()?;
                
                if !self.match_token(&TokenType::Comma) {
                    break;
                }
                self.output.push_str(", ");
            }
        }
        Ok(())
    }
    
    fn consume_statement_end(&mut self) {
        self.match_token(&TokenType::Semicolon);
        if !self.output.ends_with('\n') {
            self.output.push('\n');
        }
    }
    
    fn is_statement_end(&self) -> bool {
        self.check(&TokenType::Semicolon) || self.check(&TokenType::Newline) || 
        self.check(&TokenType::RightParen) || self.check(&TokenType::RightBrace) ||
        self.check(&TokenType::RightBracket) || self.check(&TokenType::Comma)
    }
    
    fn add_indent(&mut self) {
        for _ in 0..(self.indent_level * self.indent_size) {
            self.output.push(' ');
        }
    }
    
    fn add_space(&mut self) {
        self.output.push(' ');
    }
    
    fn add_token(&mut self) {
        let token_type = self.advance().token_type.clone();
        match &token_type {
            TokenType::Number(n) => self.output.push_str(&n.to_string()),
            TokenType::String(s) => self.output.push_str(&format!("\"{}\"", s)),
            TokenType::Identifier(name) => self.output.push_str(name),
            TokenType::Fr => self.output.push_str("fr"),
            TokenType::Cap => self.output.push_str("cap"),
            TokenType::Bet => self.output.push_str("bet"),
            TokenType::Sus => self.output.push_str("sus"),
            TokenType::Bussin => self.output.push_str("bussin"),
            TokenType::Flex => self.output.push_str("flex"),
            TokenType::Vibe => self.output.push_str("vibe"),
            TokenType::Lowkey => self.output.push_str("lowkey"),
            TokenType::Highkey => self.output.push_str("highkey"),
            TokenType::Bruh => self.output.push_str("bruh"),
            TokenType::Slay => self.output.push_str("slay"),
            TokenType::Ghost => self.output.push_str("ghost"),
            TokenType::In => self.output.push_str("in"),
            TokenType::Plus => self.output.push('+'),
            TokenType::Minus => self.output.push('-'),
            TokenType::Star => self.output.push('*'),
            TokenType::Slash => self.output.push('/'),
            TokenType::Percent => self.output.push('%'),
            TokenType::Equal => self.output.push('='),
            TokenType::EqualEqual => self.output.push_str("=="),
            TokenType::BangEqual => self.output.push_str("!="),
            TokenType::Greater => self.output.push('>'),
            TokenType::GreaterEqual => self.output.push_str(">="),
            TokenType::Less => self.output.push('<'),
            TokenType::LessEqual => self.output.push_str("<="),
            TokenType::And => self.output.push_str("&&"),
            TokenType::Or => self.output.push_str("||"),
            TokenType::Bang => self.output.push('!'),
            TokenType::LeftParen => self.output.push('('),
            TokenType::RightParen => self.output.push(')'),
            TokenType::LeftBrace => self.output.push('{'),
            TokenType::RightBrace => self.output.push('}'),
            TokenType::LeftBracket => self.output.push('['),
            TokenType::RightBracket => self.output.push(']'),
            TokenType::Comma => self.output.push(','),
            TokenType::Semicolon => self.output.push(';'),
            TokenType::Colon => self.output.push(':'),
            _ => {} // Skip newlines and EOF
        }
    }
    
    // Helper methods
    fn match_token(&mut self, token_type: &TokenType) -> bool {
        if self.check(token_type) {
            self.advance();
            true
        } else {
            false
        }
    }
    
    fn check(&self, token_type: &TokenType) -> bool {
        if self.is_at_end() {
            false
        } else {
            std::mem::discriminant(&self.peek().token_type) == std::mem::discriminant(token_type)
        }
    }
    
    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }
    
    fn is_at_end(&self) -> bool {
        self.current >= self.tokens.len() || matches!(self.peek().token_type, TokenType::Eof)
    }
    
    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }
    
    fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }
}