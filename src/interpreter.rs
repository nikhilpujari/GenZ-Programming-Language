//! ZLang Interpreter - Executes the Abstract Syntax Tree
//! This is where the magic happens and code actually runs! ✨

use std::collections::HashMap;
use crate::ast::{Expr, Stmt, BinaryOp, UnaryOp, Literal};
use crate::environment::Environment;
use crate::error::ZLangError;

#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub params: Vec<String>,
    pub body: Vec<Stmt>,
}

pub struct Interpreter {
    environment: Environment,
    functions: HashMap<String, Function>,
    return_value: Option<Literal>,
    should_break: bool,
    should_continue: bool,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            environment: Environment::new(),
            functions: HashMap::new(),
            return_value: None,
            should_break: false,
            should_continue: false,
        }
    }
    
    pub fn interpret(&mut self, statements: Vec<Stmt>) -> Result<String, ZLangError> {
        let mut output = Vec::new();
        
        for stmt in statements {
            if let Some(result) = self.execute_stmt(&stmt)? {
                output.push(result);
            }
            
            // Handle early returns from functions
            if self.return_value.is_some() {
                break;
            }
        }
        
        Ok(output.join("\n"))
    }
    
    pub fn execute_stmt(&mut self, stmt: &Stmt) -> Result<Option<String>, ZLangError> {
        match stmt {
            Stmt::Expression(expr) => {
                self.evaluate_expr(expr)?;
                Ok(None)
            }
            Stmt::VarDeclaration { name, initializer } => {
                let value = if let Some(init) = initializer {
                    self.evaluate_expr(init)?
                } else {
                    Literal::Nil
                };
                
                // Try to assign to existing variable first, if that fails, define new one
                if self.environment.assign(name, value.clone()).is_err() {
                    self.environment.define(name.clone(), value);
                }
                Ok(None)
            }
            Stmt::Block(statements) => {
                self.environment.push_scope();
                let mut result = None;
                
                for stmt in statements {
                    if let Some(output) = self.execute_stmt(stmt)? {
                        result = Some(output);
                    }
                    
                    if self.return_value.is_some() || self.should_break || self.should_continue {
                        break;
                    }
                }
                
                self.environment.pop_scope()?;
                Ok(result)
            }
            Stmt::If { condition, then_branch, else_branch } => {
                let condition_value = self.evaluate_expr(condition)?;
                
                if self.is_truthy(&condition_value) {
                    self.execute_stmt(then_branch)
                } else if let Some(else_stmt) = else_branch {
                    self.execute_stmt(else_stmt)
                } else {
                    Ok(None)
                }
            }
            Stmt::While { condition, body } => {
                loop {
                    let condition_value = self.evaluate_expr(condition)?;
                    if !self.is_truthy(&condition_value) {
                        break;
                    }
                    
                    self.execute_stmt(body)?;
                    
                    if self.should_break {
                        self.should_break = false;
                        break;
                    }
                    
                    if self.should_continue {
                        self.should_continue = false;
                        continue;
                    }
                    
                    if self.return_value.is_some() {
                        break;
                    }
                }
                Ok(None)
            }
            Stmt::For { variable, iterable, body } => {
                let iterable_value = self.evaluate_expr(iterable)?;
                
                match iterable_value {
                    Literal::Array(arr) => {
                        self.environment.push_scope();
                        
                        for item in arr {
                            self.environment.define(variable.clone(), item);
                            self.execute_stmt(body)?;
                            
                            if self.should_break {
                                self.should_break = false;
                                break;
                            }
                            
                            if self.should_continue {
                                self.should_continue = false;
                                continue;
                            }
                            
                            if self.return_value.is_some() {
                                break;
                            }
                        }
                        
                        self.environment.pop_scope()?;
                    }
                    _ => return Err(ZLangError::new("Can only iterate over arrays bestie! 📚")),
                }
                Ok(None)
            }
            Stmt::Function { name, params, body } => {
                let function = Function {
                    name: name.clone(),
                    params: params.clone(),
                    body: body.clone(),
                };
                
                self.functions.insert(name.clone(), function);
                Ok(None)
            }
            Stmt::Return(expr) => {
                let value = if let Some(expr) = expr {
                    self.evaluate_expr(expr)?
                } else {
                    Literal::Nil
                };
                
                self.return_value = Some(value);
                Ok(None)
            }
            Stmt::Break => {
                self.should_break = true;
                Ok(None)
            }
            Stmt::Continue => {
                self.should_continue = true;
                Ok(None)
            }
            Stmt::Print(expr) => {
                let value = self.evaluate_expr(expr)?;
                Ok(Some(format!("{}", value)))
            }
            Stmt::Switch { expr, cases, default } => {
                let switch_value = self.evaluate_expr(expr)?;
                let mut executed = false;
                
                for (case_expr, statements) in cases {
                    let case_value = self.evaluate_expr(case_expr)?;
                    if self.values_equal(&switch_value, &case_value) {
                        for stmt in statements {
                            match self.execute_stmt(stmt) {
                                Ok(_) => {},
                                Err(e) if e.message.contains("break") => return Ok(None),
                                Err(e) => return Err(e),
                            }
                        }
                        executed = true;
                        break;
                    }
                }
                
                if !executed {
                    if let Some(default_stmts) = default {
                        for stmt in default_stmts {
                            self.execute_stmt(stmt)?;
                        }
                    }
                }
                
                Ok(None)
            }
            Stmt::Try { try_block, catch_block, finally_block } => {
                let mut try_result = Ok(None);
                
                // Execute try block
                for stmt in try_block {
                    match self.execute_stmt(stmt) {
                        Ok(_) => {},
                        Err(e) => {
                            try_result = Err(e);
                            break;
                        }
                    }
                }
                
                // Execute catch block if there was an error
                if try_result.is_err() {
                    if let Some((error_var, catch_stmts)) = catch_block {
                        if let Err(error) = &try_result {
                            self.environment.define(error_var.clone(), Literal::String(error.to_string()));
                        }
                        for stmt in catch_stmts {
                            self.execute_stmt(stmt)?;
                        }
                        try_result = Ok(None); // Error was handled
                    }
                }
                
                // Always execute finally block
                if let Some(finally_stmts) = finally_block {
                    for stmt in finally_stmts {
                        self.execute_stmt(stmt)?;
                    }
                }
                
                try_result
            }
            Stmt::Throw(expr) => {
                let error_value = self.evaluate_expr(expr)?;
                let error_message = match error_value {
                    Literal::String(s) => s,
                    _ => "Thrown error".to_string(),
                };
                Err(ZLangError::new(&error_message))
            }
        }
    }
    
    fn values_equal(&self, left: &Literal, right: &Literal) -> bool {
        match (left, right) {
            (Literal::Number(a), Literal::Number(b)) => (a - b).abs() < f64::EPSILON,
            (Literal::String(a), Literal::String(b)) => a == b,
            (Literal::Boolean(a), Literal::Boolean(b)) => a == b,
            (Literal::Nil, Literal::Nil) => true,
            _ => false,
        }
    }
    
    fn evaluate_expr(&mut self, expr: &Expr) -> Result<Literal, ZLangError> {
        match expr {
            Expr::Literal(literal) => Ok(literal.clone()),
            Expr::Variable(name) => self.environment.get(name),
            Expr::Assign { name, value } => {
                let val = self.evaluate_expr(value)?;
                self.environment.assign(name, val.clone())?;
                Ok(val)
            }
            Expr::Binary { left, operator, right } => {
                let left_val = self.evaluate_expr(left)?;
                let right_val = self.evaluate_expr(right)?;
                self.apply_binary_op(&left_val, operator, &right_val)
            }
            Expr::Unary { operator, right } => {
                let right_val = self.evaluate_expr(right)?;
                self.apply_unary_op(operator, &right_val)
            }
            Expr::Call { callee, arguments } => {
                if let Expr::Variable(name) = callee.as_ref() {
                    // Built-in functions
                    match name.as_str() {
                        "sqrt" => {
                            if arguments.len() != 1 {
                                return Err(ZLangError::new("sqrt expects 1 argument bestie! 📊"));
                            }
                            let arg = self.evaluate_expr(&arguments[0])?;
                            if let Literal::Number(n) = arg {
                                if n < 0.0 {
                                    return Err(ZLangError::new("Can't sqrt negative numbers, that's imaginary! 🤔"));
                                }
                                Ok(Literal::Number(n.sqrt()))
                            } else {
                                Err(ZLangError::new("sqrt only works with numbers! 🔢"))
                            }
                        }
                        "abs" => {
                            if arguments.len() != 1 {
                                return Err(ZLangError::new("abs expects 1 argument bestie! 📊"));
                            }
                            let arg = self.evaluate_expr(&arguments[0])?;
                            if let Literal::Number(n) = arg {
                                Ok(Literal::Number(n.abs()))
                            } else {
                                Err(ZLangError::new("abs only works with numbers! 🔢"))
                            }
                        }
                        "random" => {
                            if arguments.len() != 0 {
                                return Err(ZLangError::new("random takes no arguments bestie! 🎲"));
                            }
                            // Simple pseudo-random number (0.0 to 1.0)
                            use std::collections::hash_map::DefaultHasher;
                            use std::hash::{Hash, Hasher};
                            use std::time::{SystemTime, UNIX_EPOCH};
                            
                            let mut hasher = DefaultHasher::new();
                            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos().hash(&mut hasher);
                            let hash = hasher.finish();
                            let random_val = (hash % 1000) as f64 / 1000.0;
                            Ok(Literal::Number(random_val))
                        }
                        "length" => {
                            if arguments.len() != 1 {
                                return Err(ZLangError::new("length expects 1 argument bestie! 📏"));
                            }
                            let arg = self.evaluate_expr(&arguments[0])?;
                            match arg {
                                Literal::String(s) => Ok(Literal::Number(s.len() as f64)),
                                Literal::Array(arr) => Ok(Literal::Number(arr.len() as f64)),
                                _ => Err(ZLangError::new("length only works with strings and arrays! 📝")),
                            }
                        }
                        "uppercase" => {
                            if arguments.len() != 1 {
                                return Err(ZLangError::new("uppercase expects 1 argument bestie! 📝"));
                            }
                            let arg = self.evaluate_expr(&arguments[0])?;
                            if let Literal::String(s) = arg {
                                Ok(Literal::String(s.to_uppercase()))
                            } else {
                                Err(ZLangError::new("uppercase only works with strings! 📝"))
                            }
                        }
                        "split" => {
                            if arguments.len() != 2 {
                                return Err(ZLangError::new("split expects 2 arguments (string, delimiter) bestie! ✂️"));
                            }
                            let string_arg = self.evaluate_expr(&arguments[0])?;
                            let delimiter_arg = self.evaluate_expr(&arguments[1])?;
                            
                            if let (Literal::String(s), Literal::String(delim)) = (string_arg, delimiter_arg) {
                                let parts: Vec<Literal> = s.split(&delim)
                                    .map(|part| Literal::String(part.to_string()))
                                    .collect();
                                Ok(Literal::Array(parts))
                            } else {
                                Err(ZLangError::new("split needs two strings (text, delimiter)! ✂️"))
                            }
                        }
                        _ => {
                            // User-defined function
                            if let Some(function) = self.functions.get(name).cloned() {
                                self.call_function(function, arguments)
                            } else {
                                Err(ZLangError::new(&format!("Undefined function '{}', that function doesn't exist bestie! 📞", name)))
                            }
                        }
                    }
                } else {
                    Err(ZLangError::new("Can only call functions, not other expressions! 🤙"))
                }
            }
            Expr::Array(elements) => {
                let mut values = Vec::new();
                for element in elements {
                    values.push(self.evaluate_expr(element)?);
                }
                Ok(Literal::Array(values))
            }
            Expr::Object(pairs) => {
                let mut map = std::collections::HashMap::new();
                for (key, value_expr) in pairs {
                    let value = self.evaluate_expr(value_expr)?;
                    map.insert(key.clone(), value);
                }
                Ok(Literal::Object(map))
            }
            Expr::Index { object, index } => {
                let obj_value = self.evaluate_expr(object)?;
                let index_value = self.evaluate_expr(index)?;
                
                match (obj_value, index_value) {
                    (Literal::Array(arr), Literal::Number(idx)) => {
                        let idx = idx as usize;
                        if idx < arr.len() {
                            Ok(arr[idx].clone())
                        } else {
                            Err(ZLangError::new("Array index out of bounds bestie! 📚"))
                        }
                    }
                    (Literal::Object(obj), Literal::String(key)) => {
                        Ok(obj.get(&key).cloned().unwrap_or(Literal::Nil))
                    }
                    _ => Err(ZLangError::new("Invalid indexing operation, check your types! 🎯")),
                }
            }
        }
    }
    
    fn call_function(&mut self, function: Function, arguments: &[Expr]) -> Result<Literal, ZLangError> {
        if arguments.len() != function.params.len() {
            return Err(ZLangError::new(&format!(
                "Function '{}' expects {} arguments but got {}, check your parameters bestie! 📊",
                function.name, function.params.len(), arguments.len()
            )));
        }
        
        // Evaluate arguments
        let mut arg_values = Vec::new();
        for arg in arguments {
            arg_values.push(self.evaluate_expr(arg)?);
        }
        
        // Create new scope for function
        self.environment.push_scope();
        
        // Bind parameters
        for (param, value) in function.params.iter().zip(arg_values.iter()) {
            self.environment.define(param.clone(), value.clone());
        }
        
        // Execute function body
        let mut result = Literal::Nil;
        for stmt in &function.body {
            self.execute_stmt(stmt)?;
            
            if let Some(return_val) = &self.return_value {
                result = return_val.clone();
                self.return_value = None;
                break;
            }
        }
        
        // Clean up scope
        self.environment.pop_scope()?;
        
        Ok(result)
    }
    
    fn apply_binary_op(&self, left: &Literal, op: &BinaryOp, right: &Literal) -> Result<Literal, ZLangError> {
        match (left, right) {
            (Literal::Number(l), Literal::Number(r)) => {
                match op {
                    BinaryOp::Add => Ok(Literal::Number(l + r)),
                    BinaryOp::Subtract => Ok(Literal::Number(l - r)),
                    BinaryOp::Multiply => Ok(Literal::Number(l * r)),
                    BinaryOp::Divide => {
                        if *r == 0.0 {
                            Err(ZLangError::new("Division by zero bestie, that's undefined! ➗"))
                        } else {
                            Ok(Literal::Number(l / r))
                        }
                    }
                    BinaryOp::Modulo => {
                        if *r == 0.0 {
                            Err(ZLangError::new("Modulo by zero, that's not how math works! 🤓"))
                        } else {
                            Ok(Literal::Number(l % r))
                        }
                    }
                    BinaryOp::Greater => Ok(Literal::Boolean(l > r)),
                    BinaryOp::GreaterEqual => Ok(Literal::Boolean(l >= r)),
                    BinaryOp::Less => Ok(Literal::Boolean(l < r)),
                    BinaryOp::LessEqual => Ok(Literal::Boolean(l <= r)),
                    BinaryOp::Equal => Ok(Literal::Boolean((l - r).abs() < f64::EPSILON)),
                    BinaryOp::NotEqual => Ok(Literal::Boolean((l - r).abs() >= f64::EPSILON)),
                    _ => Err(ZLangError::new("Invalid operation for numbers, that's not it! 🔢")),
                }
            }
            (Literal::String(l), Literal::String(r)) => {
                match op {
                    BinaryOp::Add => Ok(Literal::String(format!("{}{}", l, r))),
                    BinaryOp::Equal => Ok(Literal::Boolean(l == r)),
                    BinaryOp::NotEqual => Ok(Literal::Boolean(l != r)),
                    _ => Err(ZLangError::new("Invalid operation for strings, strings don't do that! 📝")),
                }
            }
            // String + other types (concatenation)
            (Literal::String(l), other) => {
                match op {
                    BinaryOp::Add => Ok(Literal::String(format!("{}{}", l, other))),
                    _ => Err(ZLangError::new("Can only concatenate with strings using +, that's the vibe! 🔗")),
                }
            }
            // Other types + String (concatenation)
            (other, Literal::String(r)) => {
                match op {
                    BinaryOp::Add => Ok(Literal::String(format!("{}{}", other, r))),
                    _ => Err(ZLangError::new("Can only concatenate with strings using +, that's the vibe! 🔗")),
                }
            }
            (Literal::Boolean(l), Literal::Boolean(r)) => {
                match op {
                    BinaryOp::And => Ok(Literal::Boolean(*l && *r)),
                    BinaryOp::Or => Ok(Literal::Boolean(*l || *r)),
                    BinaryOp::Equal => Ok(Literal::Boolean(l == r)),
                    BinaryOp::NotEqual => Ok(Literal::Boolean(l != r)),
                    _ => Err(ZLangError::new("Invalid operation for booleans, booleans are limited bestie! ❌")),
                }
            }
            _ => {
                // Mixed types or unsupported operations
                match op {
                    BinaryOp::Equal => Ok(Literal::Boolean(false)),
                    BinaryOp::NotEqual => Ok(Literal::Boolean(true)),
                    _ => Err(ZLangError::new("Type mismatch in operation, these types don't work together! 🔀")),
                }
            }
        }
    }
    
    fn apply_unary_op(&self, op: &UnaryOp, operand: &Literal) -> Result<Literal, ZLangError> {
        match op {
            UnaryOp::Minus => {
                if let Literal::Number(n) = operand {
                    Ok(Literal::Number(-n))
                } else {
                    Err(ZLangError::new("Can only negate numbers, that's basic math! ➖"))
                }
            }
            UnaryOp::Not => Ok(Literal::Boolean(!self.is_truthy(operand))),
        }
    }
    
    fn is_truthy(&self, literal: &Literal) -> bool {
        match literal {
            Literal::Boolean(b) => *b,
            Literal::Nil => false,
            Literal::Number(n) => *n != 0.0,
            Literal::String(s) => !s.is_empty(),
            Literal::Array(arr) => !arr.is_empty(),
            Literal::Object(obj) => !obj.is_empty(),
        }
    }
}
