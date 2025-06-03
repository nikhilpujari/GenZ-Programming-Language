//! ZLang - A Gen Z Programming Language Interpreter
//! Built with pure Rust, no cap! 💯

mod lexer;
mod parser;
mod interpreter;
mod token;
mod ast;
mod environment;
mod error;
mod formatter;
mod web_server;

use std::env;
use std::fs;
use std::io::{self, Write};
use std::process;

use lexer::Lexer;
use parser::Parser;
use interpreter::Interpreter;
use error::ZLangError;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    // Print the sick ZLang banner
    print_banner();
    
    match args.len() {
        1 => {
            // No file provided, start REPL
            println!("💬 Starting ZLang REPL... Type 'exit' to bounce!");
            run_repl();
        }
        2 => {
            let arg = &args[1];
            if arg == "--web" || arg == "-w" {
                // Start web server for interactive coding
                println!("🌐 Starting ZLang Web Server for interactive coding...");
                if let Err(e) = web_server::start_web_server() {
                    eprintln!("❌ Web server failed: {}", e);
                    process::exit(1);
                }
            } else {
                // File provided, execute it
                let filename = arg;
                if let Err(e) = run_file(filename) {
                    eprintln!("❌ That's not it chief: {}", e);
                    process::exit(1);
                }
            }
        }
        _ => {
            eprintln!("💀 Usage: zlang [script.zlang] or zlang --web");
            process::exit(1);
        }
    }
}

fn print_banner() {
    println!(r#"
 ______ _                        
|___  /| |                       
   / / | |     __ _ _ __   __ _   
  / /  | |    / _` | '_ \ / _` |  
 / /___| |___| (_| | | | | (_| |  
/_____/|______\__,_|_| |_|\__, |  
                           __/ |  
                          |___/   

🔥 ZLang v0.1.0 - The Programming Language That Hits Different
Built by Gen Z, for Gen Z. No cap! 💯
"#);
}

fn run_repl() {
    let mut interpreter = Interpreter::new();
    
    loop {
        print!("zlang> ");
        io::stdout().flush().unwrap();
        
        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(_) => {
                let input = input.trim();
                
                if input == "exit" || input == "quit" {
                    println!("👋 Peace out! Catch you later!");
                    break;
                }
                
                if input.is_empty() {
                    continue;
                }
                
                match execute_code(&mut interpreter, input) {
                    Ok(result) => {
                        if !result.is_empty() {
                            println!("📤 {}", result);
                        }
                    }
                    Err(e) => eprintln!("❌ {}", e),
                }
            }
            Err(e) => {
                eprintln!("💀 Failed to read input: {}", e);
                break;
            }
        }
    }
}

fn run_file(filename: &str) -> Result<(), ZLangError> {
    let source = fs::read_to_string(filename)
        .map_err(|_| ZLangError::new(&format!("Can't find that file '{}' bestie 📁", filename)))?;
    
    println!("🚀 Running {}...", filename);
    let mut interpreter = Interpreter::new();
    
    match execute_code(&mut interpreter, &source) {
        Ok(result) => {
            if !result.is_empty() {
                println!("{}", result);
            }
            Ok(())
        }
        Err(e) => Err(e),
    }
}

fn execute_code(interpreter: &mut Interpreter, source: &str) -> Result<String, ZLangError> {
    // Lexical analysis - turn source into tokens
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize()?;
    
    // Parsing - turn tokens into AST
    let mut parser = Parser::new(tokens);
    let statements = parser.parse()?;
    
    // Interpretation - execute the AST
    interpreter.interpret(statements)
}
