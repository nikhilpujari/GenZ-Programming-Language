//! ZLang Parser - Turns tokens into an Abstract Syntax Tree
//! This is where we figure out what the code actually means

use crate::token::{Token, TokenType};
use crate::ast::{Expr, Stmt, BinaryOp, UnaryOp, Literal};
use crate::error::ZLangError;

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }
    
    pub fn parse(&mut self) -> Result<Vec<Stmt>, ZLangError> {
        let mut statements = Vec::new();
        
        while !self.is_at_end() {
            // Skip newlines at the top level
            if self.match_token(&TokenType::Newline) {
                continue;
            }
            
            statements.push(self.declaration()?);
        }
        
        Ok(statements)
    }
    
    fn declaration(&mut self) -> Result<Stmt, ZLangError> {
        if self.match_token(&TokenType::Flex) {
            self.function_declaration()
        } else if self.match_token(&TokenType::Bet) {
            self.var_declaration()
        } else {
            self.statement()
        }
    }
    
    fn function_declaration(&mut self) -> Result<Stmt, ZLangError> {
        let name = if let TokenType::Identifier(name) = &self.peek().token_type {
            let name = name.clone();
            self.advance();
            name
        } else {
            return Err(ZLangError::new("Expected function name bestie 📝"));
        };
        
        self.consume(&TokenType::LeftParen, "Expected '(' after function name, that's how functions work!")?;
        
        let mut params = Vec::new();
        if !self.check(&TokenType::RightParen) {
            loop {
                if let TokenType::Identifier(param) = &self.peek().token_type {
                    params.push(param.clone());
                    self.advance();
                } else {
                    return Err(ZLangError::new("Expected parameter name in function declaration 📋"));
                }
                
                if !self.match_token(&TokenType::Comma) {
                    break;
                }
            }
        }
        
        self.consume(&TokenType::RightParen, "Expected ')' after parameters, close it up!")?;
        self.consume(&TokenType::LeftBrace, "Expected '{' before function body, gotta have that block!")?;
        
        let body = self.block_statement()?;
        
        if let Stmt::Block(statements) = body {
            Ok(Stmt::Function { name, params, body: statements })
        } else {
            unreachable!("block_statement should always return a Block")
        }
    }
    
    fn var_declaration(&mut self) -> Result<Stmt, ZLangError> {
        let name = if let TokenType::Identifier(name) = &self.peek().token_type {
            let name = name.clone();
            self.advance();
            name
        } else {
            return Err(ZLangError::new("Expected variable name after 'bet', gotta name your variables bestie 📛"));
        };
        
        let initializer = if self.match_token(&TokenType::Equal) {
            Some(self.expression()?)
        } else {
            None
        };
        
        self.consume_statement_end("Expected ';' or newline after variable declaration 📍")?;
        
        Ok(Stmt::VarDeclaration { name, initializer })
    }
    
    fn statement(&mut self) -> Result<Stmt, ZLangError> {
        if self.match_token(&TokenType::Sus) {
            self.if_statement()
        } else if self.match_token(&TokenType::LowkeySus) {
            self.if_statement()
        } else if self.match_token(&TokenType::NoSus) {
            self.if_statement()
        } else if self.match_token(&TokenType::Lowkey) {
            self.while_statement()
        } else if self.match_token(&TokenType::Highkey) || self.match_token(&TokenType::Grind) {
            self.for_statement()
        } else if self.match_token(&TokenType::VibeCheck) {
            self.switch_statement()
        } else if self.match_token(&TokenType::Manifest) {
            self.try_statement()
        } else if self.match_token(&TokenType::Drama) {
            self.throw_statement()
        } else if self.match_token(&TokenType::LeftBrace) {
            self.block_statement()
        } else if self.match_token(&TokenType::Vibe) {
            self.return_statement()
        } else if self.match_token(&TokenType::Slay) {
            self.consume_statement_end("Expected ';' or newline after 'slay'")?;
            Ok(Stmt::Break)
        } else if self.match_token(&TokenType::Ghost) || self.match_token(&TokenType::NoChill) {
            self.consume_statement_end("Expected ';' or newline after continue")?;
            Ok(Stmt::Continue)
        } else if self.match_token(&TokenType::Bruh) {
            self.print_statement()
        } else {
            self.expression_statement()
        }
    }
    
    fn if_statement(&mut self) -> Result<Stmt, ZLangError> {
        self.consume(&TokenType::LeftParen, "Expected '(' after 'sus'")?;
        let condition = self.expression()?;
        self.consume(&TokenType::RightParen, "Expected ')' after condition")?;
        
        let then_branch = Box::new(self.statement()?);
        let else_branch = if self.match_token(&TokenType::LowkeySus) {
            // Handle else if chain
            Some(Box::new(self.if_statement()?))
        } else if self.match_token(&TokenType::Bussin) || self.match_token(&TokenType::NoSus) {
            Some(Box::new(self.statement()?))
        } else {
            None
        };
        
        Ok(Stmt::If {
            condition,
            then_branch,
            else_branch,
        })
    }
    
    fn while_statement(&mut self) -> Result<Stmt, ZLangError> {
        self.consume(&TokenType::LeftParen, "Expected '(' after 'lowkey', wrap that condition bestie! 🔄")?;
        let condition = self.expression()?;
        self.consume(&TokenType::RightParen, "Expected ')' after condition 🔒")?;
        
        let body = Box::new(self.statement()?);
        
        Ok(Stmt::While { condition, body })
    }
    
    fn for_statement(&mut self) -> Result<Stmt, ZLangError> {
        self.consume(&TokenType::LeftParen, "Expected '(' after for loop")?;
        
        let variable = if let TokenType::Identifier(name) = &self.peek().token_type {
            let name = name.clone();
            self.advance();
            name
        } else {
            return Err(ZLangError::new("Expected variable name in for loop"));
        };
        
        self.consume(&TokenType::In, "Expected 'in' after loop variable")?;
        let iterable = self.expression()?;
        self.consume(&TokenType::RightParen, "Expected ')' after iterable")?;
        
        let body = Box::new(self.statement()?);
        
        Ok(Stmt::For { variable, iterable, body })
    }
    
    fn switch_statement(&mut self) -> Result<Stmt, ZLangError> {
        self.consume(&TokenType::LeftParen, "Expected '(' after 'vibe check'")?;
        let expr = self.expression()?;
        self.consume(&TokenType::RightParen, "Expected ')' after switch expression")?;
        self.consume(&TokenType::LeftBrace, "Expected '{' after switch expression")?;
        
        let mut cases = Vec::new();
        let mut default = None;
        
        while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
            if self.match_token(&TokenType::Newline) {
                continue;
            }
            
            if self.check(&TokenType::Identifier("default".to_string())) {
                self.advance();
                self.consume(&TokenType::Colon, "Expected ':' after default")?;
                let mut statements = Vec::new();
                while !self.check(&TokenType::RightBrace) && !self.check(&TokenType::Identifier("case".to_string())) && !self.is_at_end() {
                    if self.match_token(&TokenType::Newline) {
                        continue;
                    }
                    statements.push(self.declaration()?);
                }
                default = Some(statements);
            } else {
                let case_expr = self.expression()?;
                self.consume(&TokenType::Colon, "Expected ':' after case value")?;
                let mut statements = Vec::new();
                while !self.check(&TokenType::RightBrace) && !self.check(&TokenType::Identifier("case".to_string())) && !self.check(&TokenType::Identifier("default".to_string())) && !self.is_at_end() {
                    if self.match_token(&TokenType::Newline) {
                        continue;
                    }
                    statements.push(self.declaration()?);
                }
                cases.push((case_expr, statements));
            }
        }
        
        self.consume(&TokenType::RightBrace, "Expected '}' after switch cases")?;
        Ok(Stmt::Switch { expr, cases, default })
    }
    
    fn try_statement(&mut self) -> Result<Stmt, ZLangError> {
        self.consume(&TokenType::LeftBrace, "Expected '{' after 'manifest'")?;
        let mut try_block = Vec::new();
        while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
            if self.match_token(&TokenType::Newline) {
                continue;
            }
            try_block.push(self.declaration()?);
        }
        self.consume(&TokenType::RightBrace, "Expected '}' after try block")?;
        
        let catch_block = if self.match_token(&TokenType::Caught) {
            self.consume(&TokenType::LeftParen, "Expected '(' after 'caught'")?;
            let error_var = if let TokenType::Identifier(name) = &self.peek().token_type {
                let name = name.clone();
                self.advance();
                name
            } else {
                return Err(ZLangError::new("Expected error variable name"));
            };
            self.consume(&TokenType::RightParen, "Expected ')' after error variable")?;
            self.consume(&TokenType::LeftBrace, "Expected '{' after catch clause")?;
            
            let mut catch_stmts = Vec::new();
            while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
                if self.match_token(&TokenType::Newline) {
                    continue;
                }
                catch_stmts.push(self.declaration()?);
            }
            self.consume(&TokenType::RightBrace, "Expected '}' after catch block")?;
            Some((error_var, catch_stmts))
        } else {
            None
        };
        
        let finally_block = if self.match_token(&TokenType::Frfr) {
            self.consume(&TokenType::LeftBrace, "Expected '{' after 'frfr'")?;
            let mut finally_stmts = Vec::new();
            while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
                if self.match_token(&TokenType::Newline) {
                    continue;
                }
                finally_stmts.push(self.declaration()?);
            }
            self.consume(&TokenType::RightBrace, "Expected '}' after finally block")?;
            Some(finally_stmts)
        } else {
            None
        };
        
        Ok(Stmt::Try { try_block, catch_block, finally_block })
    }
    
    fn throw_statement(&mut self) -> Result<Stmt, ZLangError> {
        let expr = self.expression()?;
        self.consume_statement_end("Expected ';' or newline after throw expression")?;
        Ok(Stmt::Throw(expr))
    }
    
    fn block_statement(&mut self) -> Result<Stmt, ZLangError> {
        let mut statements = Vec::new();
        
        while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
            // Skip newlines in blocks
            if self.match_token(&TokenType::Newline) {
                continue;
            }
            
            statements.push(self.declaration()?);
        }
        
        self.consume(&TokenType::RightBrace, "Expected '}' after block, gotta close that block bestie! 🏁")?;
        Ok(Stmt::Block(statements))
    }
    
    fn return_statement(&mut self) -> Result<Stmt, ZLangError> {
        let value = if self.check(&TokenType::Semicolon) || self.check(&TokenType::Newline) {
            None
        } else {
            Some(self.expression()?)
        };
        
        self.consume_statement_end("Expected ';' or newline after return value 📤")?;
        Ok(Stmt::Return(value))
    }
    
    fn print_statement(&mut self) -> Result<Stmt, ZLangError> {
        let expr = self.expression()?;
        self.consume_statement_end("Expected ';' or newline after print statement 🖨️")?;
        Ok(Stmt::Print(expr))
    }
    
    fn expression_statement(&mut self) -> Result<Stmt, ZLangError> {
        let expr = self.expression()?;
        self.consume_statement_end("Expected ';' or newline after expression 📝")?;
        Ok(Stmt::Expression(expr))
    }
    
    fn expression(&mut self) -> Result<Expr, ZLangError> {
        self.assignment()
    }
    
    fn assignment(&mut self) -> Result<Expr, ZLangError> {
        let expr = self.or()?;
        
        if self.match_token(&TokenType::Equal) {
            let value = self.assignment()?;
            
            if let Expr::Variable(name) = expr {
                return Ok(Expr::Assign {
                    name,
                    value: Box::new(value),
                });
            }
            
            return Err(ZLangError::new("Invalid assignment target, can't assign to that bestie! 🎯"));
        }
        
        Ok(expr)
    }
    
    fn or(&mut self) -> Result<Expr, ZLangError> {
        let mut expr = self.and()?;
        
        while self.match_token(&TokenType::Or) {
            let right = self.and()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator: BinaryOp::Or,
                right: Box::new(right),
            };
        }
        
        Ok(expr)
    }
    
    fn and(&mut self) -> Result<Expr, ZLangError> {
        let mut expr = self.equality()?;
        
        while self.match_token(&TokenType::And) {
            let right = self.equality()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator: BinaryOp::And,
                right: Box::new(right),
            };
        }
        
        Ok(expr)
    }
    
    fn equality(&mut self) -> Result<Expr, ZLangError> {
        let mut expr = self.comparison()?;
        
        while let Some(op) = self.match_equality_op() {
            let right = self.comparison()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator: op,
                right: Box::new(right),
            };
        }
        
        Ok(expr)
    }
    
    fn match_equality_op(&mut self) -> Option<BinaryOp> {
        if self.match_token(&TokenType::BangEqual) {
            Some(BinaryOp::NotEqual)
        } else if self.match_token(&TokenType::EqualEqual) {
            Some(BinaryOp::Equal)
        } else {
            None
        }
    }
    
    fn comparison(&mut self) -> Result<Expr, ZLangError> {
        let mut expr = self.term()?;
        
        while let Some(op) = self.match_comparison_op() {
            let right = self.term()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator: op,
                right: Box::new(right),
            };
        }
        
        Ok(expr)
    }
    
    fn match_comparison_op(&mut self) -> Option<BinaryOp> {
        if self.match_token(&TokenType::Greater) {
            Some(BinaryOp::Greater)
        } else if self.match_token(&TokenType::GreaterEqual) {
            Some(BinaryOp::GreaterEqual)
        } else if self.match_token(&TokenType::Less) {
            Some(BinaryOp::Less)
        } else if self.match_token(&TokenType::LessEqual) {
            Some(BinaryOp::LessEqual)
        } else {
            None
        }
    }
    
    fn term(&mut self) -> Result<Expr, ZLangError> {
        let mut expr = self.factor()?;
        
        while let Some(op) = self.match_term_op() {
            let right = self.factor()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator: op,
                right: Box::new(right),
            };
        }
        
        Ok(expr)
    }
    
    fn match_term_op(&mut self) -> Option<BinaryOp> {
        if self.match_token(&TokenType::Minus) {
            Some(BinaryOp::Subtract)
        } else if self.match_token(&TokenType::Plus) {
            Some(BinaryOp::Add)
        } else {
            None
        }
    }
    
    fn factor(&mut self) -> Result<Expr, ZLangError> {
        let mut expr = self.unary()?;
        
        while let Some(op) = self.match_factor_op() {
            let right = self.unary()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator: op,
                right: Box::new(right),
            };
        }
        
        Ok(expr)
    }
    
    fn match_factor_op(&mut self) -> Option<BinaryOp> {
        if self.match_token(&TokenType::Slash) {
            Some(BinaryOp::Divide)
        } else if self.match_token(&TokenType::Star) {
            Some(BinaryOp::Multiply)
        } else if self.match_token(&TokenType::Percent) {
            Some(BinaryOp::Modulo)
        } else {
            None
        }
    }
    
    fn unary(&mut self) -> Result<Expr, ZLangError> {
        if let Some(op) = self.match_unary_op() {
            let right = self.unary()?;
            Ok(Expr::Unary {
                operator: op,
                right: Box::new(right),
            })
        } else {
            self.call()
        }
    }
    
    fn match_unary_op(&mut self) -> Option<UnaryOp> {
        if self.match_token(&TokenType::Bang) {
            Some(UnaryOp::Not)
        } else if self.match_token(&TokenType::Minus) {
            Some(UnaryOp::Minus)
        } else {
            None
        }
    }
    
    fn call(&mut self) -> Result<Expr, ZLangError> {
        let mut expr = self.primary()?;
        
        loop {
            if self.match_token(&TokenType::LeftParen) {
                expr = self.finish_call(expr)?;
            } else if self.match_token(&TokenType::LeftBracket) {
                let index = self.expression()?;
                self.consume(&TokenType::RightBracket, "Expected ']' after array index, close that bracket bestie! 📚")?;
                expr = Expr::Index {
                    object: Box::new(expr),
                    index: Box::new(index),
                };
            } else {
                break;
            }
        }
        
        Ok(expr)
    }
    
    fn finish_call(&mut self, callee: Expr) -> Result<Expr, ZLangError> {
        let mut arguments = Vec::new();
        
        if !self.check(&TokenType::RightParen) {
            loop {
                arguments.push(self.expression()?);
                if !self.match_token(&TokenType::Comma) {
                    break;
                }
            }
        }
        
        self.consume(&TokenType::RightParen, "Expected ')' after arguments, close those parentheses! 📞")?;
        
        Ok(Expr::Call {
            callee: Box::new(callee),
            arguments,
        })
    }
    
    fn primary(&mut self) -> Result<Expr, ZLangError> {
        match &self.peek().token_type {
            TokenType::Fr => {
                self.advance();
                Ok(Expr::Literal(Literal::Boolean(true)))
            }
            TokenType::Cap => {
                self.advance();
                Ok(Expr::Literal(Literal::Boolean(false)))
            }
            TokenType::Number(n) => {
                let n = *n;
                self.advance();
                Ok(Expr::Literal(Literal::Number(n)))
            }
            TokenType::String(s) => {
                let s = s.clone();
                self.advance();
                Ok(Expr::Literal(Literal::String(s)))
            }
            TokenType::Identifier(name) => {
                let name = name.clone();
                self.advance();
                Ok(Expr::Variable(name))
            }
            TokenType::LeftParen => {
                self.advance();
                let expr = self.expression()?;
                self.consume(&TokenType::RightParen, "Expected ')' after expression, balance those parentheses! ⚖️")?;
                Ok(expr)
            }
            TokenType::LeftBracket => {
                self.advance();
                let mut elements = Vec::new();
                
                if !self.check(&TokenType::RightBracket) {
                    loop {
                        elements.push(self.expression()?);
                        if !self.match_token(&TokenType::Comma) {
                            break;
                        }
                    }
                }
                
                self.consume(&TokenType::RightBracket, "Expected ']' after array elements, close that array bestie! 📝")?;
                Ok(Expr::Array(elements))
            }
            TokenType::LeftBrace => {
                self.advance();
                let mut pairs = Vec::new();
                
                if !self.check(&TokenType::RightBrace) {
                    loop {
                        let key = if let TokenType::Identifier(name) = &self.peek().token_type {
                            let name = name.clone();
                            self.advance();
                            name
                        } else if let TokenType::String(s) = &self.peek().token_type {
                            let s = s.clone();
                            self.advance();
                            s
                        } else {
                            return Err(ZLangError::new("Expected property name in object, objects need keys bestie! 🗝️"));
                        };
                        
                        self.consume(&TokenType::Colon, "Expected ':' after property name, that's how objects work! 🎯")?;
                        let value = self.expression()?;
                        pairs.push((key, value));
                        
                        if !self.match_token(&TokenType::Comma) {
                            break;
                        }
                    }
                }
                
                self.consume(&TokenType::RightBrace, "Expected '}' after object properties, close that object! 🏁")?;
                Ok(Expr::Object(pairs))
            }
            _ => Err(ZLangError::new(&format!(
                "Unexpected token at line {}, that's not valid in this context bestie 🤷‍♀️",
                self.peek().line
            ))),
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
        matches!(self.peek().token_type, TokenType::Eof)
    }
    
    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }
    
    fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }
    
    fn consume(&mut self, token_type: &TokenType, message: &str) -> Result<&Token, ZLangError> {
        if self.check(token_type) {
            Ok(self.advance())
        } else {
            Err(ZLangError::new(message))
        }
    }
    
    fn consume_statement_end(&mut self, message: &str) -> Result<(), ZLangError> {
        if self.match_token(&TokenType::Semicolon) || self.match_token(&TokenType::Newline) || self.is_at_end() {
            Ok(())
        } else if self.check(&TokenType::RightBrace) || 
                  self.check(&TokenType::Bussin) ||
                  self.check(&TokenType::LowkeySus) ||
                  self.check(&TokenType::NoSus) ||
                  self.check(&TokenType::Caught) ||
                  self.check(&TokenType::Frfr) {
            // Allow statements to end before closing braces or else keywords
            Ok(())
        } else {
            Err(ZLangError::new(message))
        }
    }
}
