//! Environment for variable and function scoping in ZLang
//! This is where we keep track of what variables exist and their values

use std::collections::HashMap;
use crate::ast::Literal;
use crate::error::ZLangError;

#[derive(Debug, Clone)]
pub struct Environment {
    scopes: Vec<HashMap<String, Literal>>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            scopes: vec![HashMap::new()], // Global scope
        }
    }
    
    pub fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }
    
    pub fn pop_scope(&mut self) -> Result<(), ZLangError> {
        if self.scopes.len() <= 1 {
            return Err(ZLangError::new("Can't pop global scope bestie, that's the foundation! 🏗️"));
        }
        self.scopes.pop();
        Ok(())
    }
    
    pub fn define(&mut self, name: String, value: Literal) {
        if let Some(current_scope) = self.scopes.last_mut() {
            current_scope.insert(name, value);
        }
    }
    
    pub fn get(&self, name: &str) -> Result<Literal, ZLangError> {
        // Search from the most recent scope backwards
        for scope in self.scopes.iter().rev() {
            if let Some(value) = scope.get(name) {
                return Ok(value.clone());
            }
        }
        
        Err(ZLangError::new(&format!("Undefined variable '{}', you haven't declared this bestie! 🤔", name)))
    }
    
    pub fn assign(&mut self, name: &str, value: Literal) -> Result<(), ZLangError> {
        // Search from the most recent scope backwards
        for scope in self.scopes.iter_mut().rev() {
            if scope.contains_key(name) {
                scope.insert(name.to_string(), value);
                return Ok(());
            }
        }
        
        Err(ZLangError::new(&format!("Undefined variable '{}', can't assign to something that doesn't exist! 🚫", name)))
    }
}
