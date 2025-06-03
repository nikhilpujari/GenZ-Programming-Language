use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use crate::{Lexer, Parser, Interpreter};

pub fn start_web_server() -> Result<(), Box<dyn std::error::Error>> {
    let port = std::env::var("PORT").unwrap_or_else(|_| "5000".to_string());
    let addr = format!("0.0.0.0:{}", port);
    let listener = TcpListener::bind(&addr)?;
    println!("🌐 ZLang Web Server running on http://{}", addr);
    
    for stream in listener.incoming() {
        let stream = stream?;
        handle_connection(stream)?;
    }
    
    Ok(())
}

fn handle_connection(mut stream: TcpStream) -> Result<(), Box<dyn std::error::Error>> {
    use std::io::BufReader;
    use std::io::BufRead;
    
    let mut reader = BufReader::new(&mut stream);
    let mut request_lines = Vec::new();
    let mut content_length = 0;
    
    // Read headers
    loop {
        let mut line = String::new();
        reader.read_line(&mut line)?;
        if line.trim().is_empty() {
            break; // End of headers
        }
        
        if line.to_lowercase().starts_with("content-length:") {
            if let Some(length_str) = line.split(':').nth(1) {
                content_length = length_str.trim().parse().unwrap_or(0);
            }
        }
        
        request_lines.push(line);
    }
    
    // Read body if present
    let mut body = String::new();
    if content_length > 0 {
        let mut body_buffer = vec![0u8; content_length];
        reader.read_exact(&mut body_buffer)?;
        body = String::from_utf8_lossy(&body_buffer).to_string();
    }
    
    let request = format!("{}\r\n\r\n{}", request_lines.join(""), body);
    let request_line = request_lines.first().map(|s| s.as_str()).unwrap_or("");
    
    let (status_line, contents) = if request_line.starts_with("OPTIONS") {
        ("HTTP/1.1 200 OK", String::new())
    } else if request_line.starts_with("GET / ") {
        ("HTTP/1.1 200 OK", get_html_page())
    } else if request_line.starts_with("POST /execute") {
        let body = extract_post_body(&request);
        eprintln!("DEBUG: Extracted body from request: '{}'", body);
        let result = execute_zlang_code(&body);
        ("HTTP/1.1 200 OK", format_json_response(&result))
    } else {
        ("HTTP/1.1 404 NOT FOUND", "404 Not Found".to_string())
    };
    
    let response = format!(
        "{}\r\nContent-Type: {}\r\nAccess-Control-Allow-Origin: *\r\nAccess-Control-Allow-Methods: GET, POST, OPTIONS\r\nAccess-Control-Allow-Headers: Content-Type\r\nContent-Length: {}\r\n\r\n{}",
        status_line,
        if request_line.starts_with("POST") { "application/json" } else { "text/html" },
        contents.len(),
        contents
    );
    
    stream.write(response.as_bytes())?;
    stream.flush()?;
    
    Ok(())
}

fn extract_post_body(request: &str) -> String {
    // Find the start of the body after HTTP headers
    if let Some(body_start) = request.find("\r\n\r\n") {
        let body = &request[body_start + 4..];
        
        // Get the actual body content, trimming null bytes
        let body = body.trim_end_matches('\0').trim();
        eprintln!("DEBUG: Raw HTTP body: '{}'", body);
        
        // Parse JSON manually: {"code": "..."}
        if let Some(code_pos) = body.find("\"code\":") {
            let after_code = &body[code_pos + 7..]; // Skip "code":
            let after_code = after_code.trim_start();
            
            if after_code.starts_with('"') {
                // Find the closing quote, handling escaped quotes
                let content = &after_code[1..]; // Skip opening quote
                let mut chars = content.chars();
                let mut result = String::new();
                let mut escaped = false;
                
                while let Some(ch) = chars.next() {
                    if escaped {
                        match ch {
                            'n' => result.push('\n'),
                            't' => result.push('\t'),
                            'r' => result.push('\r'),
                            '\\' => result.push('\\'),
                            '"' => result.push('"'),
                            _ => {
                                result.push('\\');
                                result.push(ch);
                            }
                        }
                        escaped = false;
                    } else if ch == '\\' {
                        escaped = true;
                    } else if ch == '"' {
                        // Found closing quote
                        eprintln!("DEBUG: Successfully parsed code: '{}'", result);
                        return result;
                    } else {
                        result.push(ch);
                    }
                }
            }
        }
    }
    
    eprintln!("DEBUG: Failed to parse JSON body");
    String::new()
}

fn execute_zlang_code(code: &str) -> Result<String, String> {
    if code.trim().is_empty() {
        return Ok("// Enter some ZLang code and hit Run!".to_string());
    }
    
    let mut lexer = Lexer::new(code);
    let tokens = match lexer.tokenize() {
        Ok(tokens) => tokens,
        Err(e) => return Err(format!("Lexer Error: {}", e)),
    };
    
    let mut parser = Parser::new(tokens);
    let statements = match parser.parse() {
        Ok(statements) => statements,
        Err(e) => return Err(format!("Parser Error: {}", e)),
    };
    
    let mut interpreter = Interpreter::new();
    let mut output = String::new();
    
    for statement in &statements {
        match interpreter.execute_stmt(statement) {
            Ok(Some(result)) => {
                output.push_str(&result);
                output.push('\n');
            },
            Ok(None) => {},
            Err(e) => return Err(format!("Runtime Error: {}", e)),
        }
    }
    
    if output.is_empty() {
        output = "// Code executed successfully (no output)".to_string();
    }
    
    Ok(output.trim_end().to_string())
}

fn format_json_response(result: &Result<String, String>) -> String {
    match result {
        Ok(output) => format!("{{\"success\": true, \"output\": \"{}\"}}", escape_json(output)),
        Err(error) => format!("{{\"success\": false, \"error\": \"{}\"}}", escape_json(error)),
    }
}

fn escape_json(s: &str) -> String {
    s.replace("\\", "\\\\")
     .replace("\"", "\\\"")
     .replace("\n", "\\n")
     .replace("\r", "\\r")
     .replace("\t", "\\t")
}

fn get_html_page() -> String {
    r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>ZLang - Programming That Hits Different</title>
    <link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/codemirror/5.65.2/codemirror.min.css">
    <link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/codemirror/5.65.2/theme/monokai.min.css">
    <script src="https://cdnjs.cloudflare.com/ajax/libs/codemirror/5.65.2/codemirror.min.js"></script>
    <script src="https://cdnjs.cloudflare.com/ajax/libs/codemirror/5.65.2/mode/javascript/javascript.min.js"></script>
    <style>
        * {
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }
        
        body {
            font-family: 'Courier New', monospace;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            min-height: 100vh;
            color: white;
            padding: 20px;
        }
        
        .container {
            max-width: 1200px;
            margin: 0 auto;
        }
        
        .header {
            text-align: center;
            margin-bottom: 30px;
        }
        
        .header h1 {
            font-size: 3rem;
            margin-bottom: 10px;
            text-shadow: 2px 2px 4px rgba(0,0,0,0.3);
        }
        
        .header p {
            font-size: 1.2rem;
            opacity: 0.9;
        }
        
        .playground {
            display: grid;
            grid-template-columns: 1fr 1fr;
            gap: 20px;
            margin-bottom: 30px;
        }
        
        @media (max-width: 768px) {
            .playground {
                grid-template-columns: 1fr;
            }
        }
        
        .editor-panel, .output-panel {
            background: rgba(0,0,0,0.2);
            border-radius: 10px;
            padding: 20px;
            backdrop-filter: blur(10px);
        }
        
        .panel-header {
            display: flex;
            justify-content: space-between;
            align-items: center;
            margin-bottom: 15px;
        }
        
        .panel-header h3 {
            color: #ffd700;
        }
        
        .run-button {
            background: linear-gradient(45deg, #ff6b6b, #ee5a24);
            color: white;
            border: none;
            padding: 10px 20px;
            border-radius: 20px;
            cursor: pointer;
            font-weight: bold;
            transition: transform 0.3s ease;
        }
        
        .run-button:hover {
            transform: translateY(-2px);
        }
        
        .run-button:disabled {
            opacity: 0.6;
            cursor: not-allowed;
        }
        
        .editor-wrapper {
            border: 2px solid #ddd;
            border-radius: 8px;
            background: #f8f9fa;
            height: 400px;
            overflow: hidden;
        }
        
        .CodeMirror {
            height: 100%;
            font-family: 'Courier New', monospace;
            font-size: 14px;
        }
        
        .CodeMirror-line {
            line-height: 1.4;
        }
        
        /* Custom syntax highlighting for ZLang keywords */
        .cm-zlang-keyword {
            color: #e74c3c !important;
            font-weight: bold !important;
        }
        
        .cm-zlang-string {
            color: #27ae60 !important;
            font-weight: bold !important;
        }
        
        .cm-zlang-number {
            color: #3498db !important;
            font-weight: bold !important;
        }
        
        .cm-zlang-operator {
            color: #f39c12 !important;
            font-weight: bold !important;
        }
        
        .cm-zlang-comment {
            color: #7f8c8d !important;
            font-style: italic !important;
        }
        
        #output {
            background: #1a1a1a;
            color: #f8f8f2;
            border-radius: 5px;
            padding: 15px;
            font-family: 'Courier New', monospace;
            font-size: 14px;
            line-height: 1.5;
            height: 400px;
            overflow-y: auto;
            white-space: pre-wrap;
        }
        
        .examples {
            margin-bottom: 30px;
        }
        
        .examples h3 {
            color: #ffd700;
            margin-bottom: 15px;
            text-align: center;
        }
        
        .example-buttons {
            display: flex;
            gap: 10px;
            justify-content: center;
            flex-wrap: wrap;
        }
        
        .example-button {
            background: rgba(255,255,255,0.1);
            color: white;
            border: 1px solid rgba(255,255,255,0.3);
            padding: 8px 16px;
            border-radius: 15px;
            cursor: pointer;
            font-size: 12px;
            transition: background 0.3s ease;
        }
        
        .example-button:hover {
            background: rgba(255,255,255,0.2);
        }
        
        .keywords {
            background: rgba(0,0,0,0.2);
            border-radius: 10px;
            padding: 20px;
            backdrop-filter: blur(10px);
        }
        
        .keywords h3 {
            color: #ffd700;
            margin-bottom: 15px;
            text-align: center;
        }
        
        .keywords-grid {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
            gap: 15px;
        }
        
        .keyword-pair {
            display: flex;
            justify-content: space-between;
            padding: 5px 0;
            border-bottom: 1px solid rgba(255,255,255,0.1);
        }
        
        .traditional {
            color: #ff6b6b;
        }
        
        .zlang {
            color: #4ecdc4;
            font-weight: bold;
        }
    </style>
</head>
<body>
    <div class="container">
        <header class="header">
            <h1>ZLang</h1>
            <p>The Language of the Gen Z - Try It Live!</p>
        </header>
        
        <section class="examples">
            <h3>Quick Examples</h3>
            <div class="example-buttons">
                <button class="example-button" onclick="loadExample('hello')">Hello World</button>
                <button class="example-button" onclick="loadExample('variables')">Variables</button>
                <button class="example-button" onclick="loadExample('functions')">Functions</button>
                <button class="example-button" onclick="loadExample('loops')">Loops</button>
                <button class="example-button" onclick="loadExample('conditions')">If/Else</button>
                <button class="example-button" onclick="loadExample('errors')">Error Handling</button>
                <button class="example-button" onclick="loadExample('comprehensive')">All Features</button>
                <button class="example-button" onclick="clearEditor()">New File</button>
            </div>
        </section>
        
        <section class="playground">
            <div class="editor-panel">
                <div class="panel-header">
                    <h3>ZLang Code Editor</h3>
                    <button class="run-button" onclick="runCode()" id="runBtn">Run Code</button>
                </div>
                <div class="editor-wrapper">
                    <textarea id="code-editor" style="display: none;"></textarea>
                </div>
            </div>
            
            <div class="output-panel">
                <div class="panel-header">
                    <h3>Output</h3>
                </div>
                <div id="output">// Click 'Run Code' to see output here</div>
            </div>
        </section>
        
        <section class="keywords">
            <h3>Gen Z Keywords Dictionary</h3>
            <div class="keywords-table">
                <table style="width: 100%; border-collapse: collapse; margin: 20px 0; box-shadow: 0 4px 6px rgba(0,0,0,0.1); border-radius: 10px; overflow: hidden;">
                    <thead>
                        <tr style="background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); color: white;">
                            <th style="padding: 15px; text-align: left; font-size: 16px; font-weight: bold;">Traditional</th>
                            <th style="padding: 15px; text-align: left; font-size: 16px; font-weight: bold;">ZLang (Gen Z)</th>
                            <th style="padding: 15px; text-align: left; font-size: 16px; font-weight: bold;">Category</th>
                        </tr>
                    </thead>
                    <tbody style="background: white;">
                        <tr style="border-bottom: 1px solid #e9ecef;"><td style="padding: 12px 15px; color: #2c3e50; font-weight: 500;">let</td><td style="padding: 12px 15px; color: #e74c3c; font-weight: bold; font-size: 15px;">bet</td><td style="padding: 12px 15px; color: #6c757d;">Variables</td></tr>
                        <tr style="background-color: #f8f9fa; border-bottom: 1px solid #e9ecef;"><td style="padding: 12px 15px; color: #2c3e50; font-weight: 500;">true</td><td style="padding: 12px 15px; color: #e74c3c; font-weight: bold; font-size: 15px;">fr</td><td style="padding: 12px 15px; color: #6c757d;">Literals</td></tr>
                        <tr style="border-bottom: 1px solid #e9ecef;"><td style="padding: 12px 15px; color: #2c3e50; font-weight: 500;">false</td><td style="padding: 12px 15px; color: #e74c3c; font-weight: bold; font-size: 15px;">cap</td><td style="padding: 12px 15px; color: #6c757d;">Literals</td></tr>
                        <tr style="background-color: #f8f9fa; border-bottom: 1px solid #e9ecef;"><td style="padding: 12px 15px; color: #2c3e50; font-weight: 500;">if</td><td style="padding: 12px 15px; color: #e74c3c; font-weight: bold; font-size: 15px;">sus</td><td style="padding: 12px 15px; color: #6c757d;">Conditionals</td></tr>
                        <tr style="border-bottom: 1px solid #e9ecef;"><td style="padding: 12px 15px; color: #2c3e50; font-weight: 500;">else if</td><td style="padding: 12px 15px; color: #e74c3c; font-weight: bold; font-size: 15px;">lowkey sus</td><td style="padding: 12px 15px; color: #6c757d;">Conditionals</td></tr>
                        <tr style="background-color: #f8f9fa; border-bottom: 1px solid #e9ecef;"><td style="padding: 12px 15px; color: #2c3e50; font-weight: 500;">else</td><td style="padding: 12px 15px; color: #e74c3c; font-weight: bold; font-size: 15px;">no sus</td><td style="padding: 12px 15px; color: #6c757d;">Conditionals</td></tr>
                        <tr style="border-bottom: 1px solid #e9ecef;"><td style="padding: 12px 15px; color: #2c3e50; font-weight: 500;">switch</td><td style="padding: 12px 15px; color: #e74c3c; font-weight: bold; font-size: 15px;">vibecheck</td><td style="padding: 12px 15px; color: #6c757d;">Conditionals</td></tr>
                        <tr style="background-color: #f8f9fa; border-bottom: 1px solid #e9ecef;"><td style="padding: 12px 15px; color: #2c3e50; font-weight: 500;">for</td><td style="padding: 12px 15px; color: #e74c3c; font-weight: bold; font-size: 15px;">grind</td><td style="padding: 12px 15px; color: #6c757d;">Loops</td></tr>
                        <tr style="border-bottom: 1px solid #e9ecef;"><td style="padding: 12px 15px; color: #2c3e50; font-weight: 500;">while</td><td style="padding: 12px 15px; color: #e74c3c; font-weight: bold; font-size: 15px;">lowkey</td><td style="padding: 12px 15px; color: #6c757d;">Loops</td></tr>
                        <tr style="background-color: #f8f9fa; border-bottom: 1px solid #e9ecef;"><td style="padding: 12px 15px; color: #2c3e50; font-weight: 500;">continue</td><td style="padding: 12px 15px; color: #e74c3c; font-weight: bold; font-size: 15px;">no chill</td><td style="padding: 12px 15px; color: #6c757d;">Control Flow</td></tr>
                        <tr style="border-bottom: 1px solid #e9ecef;"><td style="padding: 12px 15px; color: #2c3e50; font-weight: 500;">break</td><td style="padding: 12px 15px; color: #e74c3c; font-weight: bold; font-size: 15px;">slay</td><td style="padding: 12px 15px; color: #6c757d;">Control Flow</td></tr>
                        <tr style="background-color: #f8f9fa; border-bottom: 1px solid #e9ecef;"><td style="padding: 12px 15px; color: #2c3e50; font-weight: 500;">function</td><td style="padding: 12px 15px; color: #e74c3c; font-weight: bold; font-size: 15px;">flex</td><td style="padding: 12px 15px; color: #6c757d;">Functions</td></tr>
                        <tr style="border-bottom: 1px solid #e9ecef;"><td style="padding: 12px 15px; color: #2c3e50; font-weight: 500;">return</td><td style="padding: 12px 15px; color: #e74c3c; font-weight: bold; font-size: 15px;">vibe</td><td style="padding: 12px 15px; color: #6c757d;">Functions</td></tr>
                        <tr style="background-color: #f8f9fa; border-bottom: 1px solid #e9ecef;"><td style="padding: 12px 15px; color: #2c3e50; font-weight: 500;">print</td><td style="padding: 12px 15px; color: #e74c3c; font-weight: bold; font-size: 15px;">bruh</td><td style="padding: 12px 15px; color: #6c757d;">Output</td></tr>
                        <tr style="border-bottom: 1px solid #e9ecef;"><td style="padding: 12px 15px; color: #2c3e50; font-weight: 500;">try</td><td style="padding: 12px 15px; color: #e74c3c; font-weight: bold; font-size: 15px;">manifest</td><td style="padding: 12px 15px; color: #6c757d;">Error Handling</td></tr>
                        <tr style="background-color: #f8f9fa; border-bottom: 1px solid #e9ecef;"><td style="padding: 12px 15px; color: #2c3e50; font-weight: 500;">catch</td><td style="padding: 12px 15px; color: #e74c3c; font-weight: bold; font-size: 15px;">caught</td><td style="padding: 12px 15px; color: #6c757d;">Error Handling</td></tr>
                        <tr style="border-bottom: 1px solid #e9ecef;"><td style="padding: 12px 15px; color: #2c3e50; font-weight: 500;">throw</td><td style="padding: 12px 15px; color: #e74c3c; font-weight: bold; font-size: 15px;">drama</td><td style="padding: 12px 15px; color: #6c757d;">Error Handling</td></tr>
                        <tr style="background-color: #f8f9fa;"><td style="padding: 12px 15px; color: #2c3e50; font-weight: 500;">finally</td><td style="padding: 12px 15px; color: #e74c3c; font-weight: bold; font-size: 15px;">frfr</td><td style="padding: 12px 15px; color: #6c757d;">Error Handling</td></tr>
                    </tbody>
                </table>
            </div>
        </section>
        
        <section class="technical">
            <h2 style="color: #2c3e50; font-size: 28px; font-weight: bold; margin-bottom: 20px; text-align: center; background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); color: white; padding: 15px; border-radius: 10px;">🛠️ How ZLang Was Built</h2>
            
            <!-- GitHub Collaboration Section -->
            <div style="text-align: center; margin-bottom: 30px;">
                <div style="background: linear-gradient(135deg, #24292e 0%, #2f363d 100%); padding: 20px; border-radius: 15px; box-shadow: 0 8px 16px rgba(0,0,0,0.2);">
                    <h3 style="color: white; margin-bottom: 15px; font-size: 22px;">🤝 Want to Collaborate?</h3>
                    <p style="color: #ffffff; opacity: 0.9; margin-bottom: 20px; font-size: 16px;">
                        Help us improve ZLang! Contribute new features, fix bugs, or add more Gen Z slang to make it even better.
                    </p>
                    <a href="https://github.com/nikhilpujari/GenZ-Programming-Language" target="_blank" 
                       style="display: inline-block; background: linear-gradient(45deg, #28a745, #20c997); color: white; text-decoration: none; padding: 12px 25px; border-radius: 25px; font-weight: bold; font-size: 16px; transition: transform 0.3s ease, box-shadow 0.3s ease; box-shadow: 0 4px 8px rgba(0,0,0,0.2);"
                       onmouseover="this.style.transform='translateY(-2px)'; this.style.boxShadow='0 8px 16px rgba(0,0,0,0.3)'"
                       onmouseout="this.style.transform='translateY(0px)'; this.style.boxShadow='0 4px 8px rgba(0,0,0,0.2)'">
                        ⭐ View on GitHub & Contribute
                    </a>
                </div>
            </div>

            <div class="tech-content" style="background: #ffffff; border: 2px solid #e9ecef; padding: 20px; border-radius: 10px; margin: 20px 0; color: #333333;">
                
                <!-- Simple Explanation Section -->
                <div class="tech-section" style="background: #f8f9fa; padding: 20px; border-radius: 10px; margin-bottom: 25px; border-left: 5px solid #667eea;">
                    <h4 style="color: #2c3e50; margin-bottom: 15px; font-size: 20px;">📚 Simple Explanation: How We Built ZLang</h4>
                    
                    <div style="margin-bottom: 20px;">
                        <h5 style="color: #667eea; margin-bottom: 8px;">Step 1: Teaching the Computer Words</h5>
                        <p style="margin-bottom: 10px; line-height: 1.6;">We taught the computer to recognize Gen Z slang as programming commands.</p>
                        <div style="background: #e9ecef; padding: 10px; border-radius: 5px; font-family: monospace; font-size: 14px;">
                            <strong>Example:</strong> When you type <span style="color: #e74c3c; font-weight: bold;">"bet name = 'Alex'"</span><br>
                            Computer sees: <span style="color: #27ae60;">bet</span> (keyword) + <span style="color: #3498db;">name</span> (identifier) + <span style="color: #f39c12;">=</span> (operator) + <span style="color: #27ae60;">"Alex"</span> (text)
                        </div>
                    </div>

                    <div style="margin-bottom: 20px;">
                        <h5 style="color: #667eea; margin-bottom: 8px;">Step 2: Understanding Grammar Rules</h5>
                        <p style="margin-bottom: 10px; line-height: 1.6;">We built rules so the computer knows how words fit together.</p>
                        <div style="background: #e9ecef; padding: 10px; border-radius: 5px; font-family: monospace; font-size: 14px;">
                            <strong>Rules we taught:</strong><br>
                            • <span style="color: #e74c3c;">bet [name] = [value]</span> → Create variable<br>
                            • <span style="color: #e74c3c;">sus ([condition]) { [code] }</span> → If statement<br>
                            • <span style="color: #e74c3c;">bruh [message]</span> → Print output
                        </div>
                    </div>

                    <div style="margin-bottom: 20px;">
                        <h5 style="color: #667eea; margin-bottom: 8px;">Step 3: Making It Actually Work</h5>
                        <p style="margin-bottom: 10px; line-height: 1.6;">We built an executor that does what the code says.</p>
                        <div style="background: #e9ecef; padding: 10px; border-radius: 5px; font-family: monospace; font-size: 14px;">
                            <strong>What happens:</strong><br>
                            1. You type: <span style="color: #e74c3c;">bet name = "Alex"</span><br>
                            2. Computer stores: Alex → name<br>
                            3. You type: <span style="color: #e74c3c;">bruh name</span><br>
                            4. Computer prints: Alex
                        </div>
                    </div>

                    <div>
                        <h5 style="color: #667eea; margin-bottom: 8px;">Step 4: Which Files Do What</h5>
                        <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 15px; margin-top: 10px;">
                            <div style="background: white; padding: 12px; border-radius: 8px; border: 1px solid #dee2e6;">
                                <strong style="color: #e74c3c;">src/lexer.rs</strong><br>
                                <small>Recognizes words and symbols</small>
                            </div>
                            <div style="background: white; padding: 12px; border-radius: 8px; border: 1px solid #dee2e6;">
                                <strong style="color: #e74c3c;">src/parser.rs</strong><br>
                                <small>Understands grammar rules</small>
                            </div>
                            <div style="background: white; padding: 12px; border-radius: 8px; border: 1px solid #dee2e6;">
                                <strong style="color: #e74c3c;">src/interpreter.rs</strong><br>
                                <small>Actually runs your code</small>
                            </div>
                            <div style="background: white; padding: 12px; border-radius: 8px; border: 1px solid #dee2e6;">
                                <strong style="color: #e74c3c;">src/web_server.rs</strong><br>
                                <small>Creates this website</small>
                            </div>
                        </div>
                    </div>
                </div>

                <div class="tech-section">
                    <h4 style="color: #2c3e50; margin-bottom: 10px;">🔧 Architecture Overview</h4>
                    <p>ZLang is a complete programming language interpreter built entirely in Rust, featuring a three-stage compilation pipeline:</p>
                    <ul style="margin-left: 20px;">
                        <li><strong>Lexical Analysis:</strong> Custom lexer tokenizes Gen Z slang into structured tokens</li>
                        <li><strong>Parsing:</strong> Recursive descent parser builds Abstract Syntax Trees (AST)</li>
                        <li><strong>Interpretation:</strong> Tree-walking interpreter executes the AST with environment management</li>
                    </ul>
                </div>
                
                <div class="tech-section" style="margin-top: 20px;">
                    <h4 style="color: #2c3e50; margin-bottom: 10px;">🚀 Core Components</h4>
                    <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 15px; margin-top: 10px;">
                        <div>
                            <strong>Lexer (src/lexer.rs):</strong>
                            <ul style="font-size: 14px; margin-left: 15px;">
                                <li>Multi-word keyword recognition</li>
                                <li>String/number literal parsing</li>
                                <li>Error handling with line/column tracking</li>
                            </ul>
                        </div>
                        <div>
                            <strong>Parser (src/parser.rs):</strong>
                            <ul style="font-size: 14px; margin-left: 15px;">
                                <li>Expression precedence handling</li>
                                <li>Control flow statement parsing</li>
                                <li>Function declaration support</li>
                            </ul>
                        </div>
                        <div>
                            <strong>Interpreter (src/interpreter.rs):</strong>
                            <ul style="font-size: 14px; margin-left: 15px;">
                                <li>Environment scope management</li>
                                <li>Function call resolution</li>
                                <li>Error propagation system</li>
                            </ul>
                        </div>
                        <div>
                            <strong>Web Server (src/web_server.rs):</strong>
                            <ul style="font-size: 14px; margin-left: 15px;">
                                <li>HTTP request handling</li>
                                <li>JSON API for code execution</li>
                                <li>Interactive web playground</li>
                            </ul>
                        </div>
                    </div>
                </div>
                
                <div class="tech-section" style="margin-top: 20px;">
                    <h4 style="color: #2c3e50; margin-bottom: 10px;">💡 Key Features</h4>
                    <div style="display: flex; flex-wrap: wrap; gap: 10px;">
                        <span style="background: #e3f2fd; padding: 5px 10px; border-radius: 15px; font-size: 14px;">Multi-word Keywords</span>
                        <span style="background: #f3e5f5; padding: 5px 10px; border-radius: 15px; font-size: 14px;">Real-time Execution</span>
                        <span style="background: #e8f5e8; padding: 5px 10px; border-radius: 15px; font-size: 14px;">Error Handling</span>
                        <span style="background: #fff3e0; padding: 5px 10px; border-radius: 15px; font-size: 14px;">Function Support</span>
                        <span style="background: #fce4ec; padding: 5px 10px; border-radius: 15px; font-size: 14px;">Control Flow</span>
                        <span style="background: #e0f2f1; padding: 5px 10px; border-radius: 15px; font-size: 14px;">Web Interface</span>
                    </div>
                </div>
                
                <div class="tech-section" style="margin-top: 20px;">
                    <h4 style="color: #2c3e50; margin-bottom: 10px;">🔬 Implementation Details</h4>
                    <p style="font-size: 14px; line-height: 1.6;">
                        The language uses a <strong>tree-walking interpreter</strong> approach for simplicity and educational value. 
                        Multi-word keywords like "lowkey sus" and "no chill" are handled through a two-stage lexing process that 
                        first identifies the base word, then checks for continuation patterns. The environment system supports 
                        lexical scoping with a stack-based scope management system.
                    </p>
                </div>
                
                <div class="tech-section" style="margin-top: 20px;">
                    <h4 style="color: #2c3e50; margin-bottom: 10px;">📦 Built With</h4>
                    <div style="display: flex; gap: 15px; align-items: center;">
                        <div style="background: #ce422b; color: white; padding: 8px 12px; border-radius: 5px; font-weight: bold;">🦀 Rust</div>
                        <div style="background: #f39c12; color: white; padding: 8px 12px; border-radius: 5px; font-weight: bold;">📦 Cargo</div>
                        <div style="background: #3498db; color: white; padding: 8px 12px; border-radius: 5px; font-weight: bold;">🌐 HTTP Server</div>
                        <div style="background: #9b59b6; color: white; padding: 8px 12px; border-radius: 5px; font-weight: bold;">🎨 Web UI</div>
                    </div>
                </div>
            </div>
        </section>
    </div>
    
    <script>
        const examples = {
            hello: `bruh "Hello, World! ZLang hits different!"
bet name = "Future Programmer"
bruh "Welcome " + name + " to ZLang!"`,
            
            variables: `bet name = "Alex"
bet age = 21
bet is_student = fr
bet gpa = 3.8

bruh "Student: " + name
bruh "Age: " + age
bruh "GPA: " + gpa`,
            
            functions: `flex greet_squad(member) {
    vibe "What's good " + member + "!"
}

flex calculate_grade(score) {
    sus (score >= 90) {
        vibe "A - Absolutely slaying!"
    } no sus {
        vibe "Need to grind harder!"
    }
}

bet greeting = greet_squad("Bestie")
bruh greeting

bet grade = calculate_grade(95)
bruh grade`,
            
            loops: `// For loop with grind keyword
bet numbers = [1, 2, 3, 4, 5]
bruh "Grind time! Let's count:"

grind (num in numbers) {
    sus (num == 3) {
        bruh "Skipping 3!"
        noChill  // continue keyword
    }
    sus (num == 5) {
        bruh "Breaking at 5!"
        slay     // break keyword
    }
    bruh "Number: " + num
}

// While loop with lowkey keyword
bet countdown = 3
bruh "\\nCountdown with lowkey:"
lowkey (countdown > 0) {
    bruh countdown
    bet countdown = countdown - 1
}
bruh "Blast off!"`,
            
            conditions: `// If/Else If/Else with sus/lowkey sus/no sus
bet score = 87

sus (score >= 90) {
    bruh "A+ grade! Absolutely crushing it!"
} lowkey sus (score >= 80) {
    bruh "B grade! Still pretty solid!"
} lowkey sus (score >= 70) {
    bruh "C grade! You can do better!"
} no sus {
    bruh "Time to hit the books harder!"
}

// Switch statement with vibecheck
bet mood = "excited"
bruh "\\nVibe check time:"

vibecheck (mood) {
    "happy": {
        bruh "You're glowing!"
    }
    "excited": {
        bruh "Your energy is contagious!"
    }
    "chill": {
        bruh "Cool vibes detected!"
    }
    default: {
        bruh "Whatever mood, you're valid!"
    }
}`,

            errors: `// Error Handling with manifest/caught/drama/frfr
bruh "=== Error Handling Demo ==="

// Basic try-catch with manifest/caught
manifest {
    bruh "Trying something risky..."
    bet risky_number = 100
    sus (risky_number > 50) {
        drama "Number too high! Drama incoming!"
    }
    bruh "This won't print if error thrown"
} caught (error) {
    bruh "Caught the drama: " + error
}

// Try-catch-finally with frfr
bruh "\\nTry-catch-finally example:"
manifest {
    bruh "Attempting operation..."
    drama "Intentional error for demo"
} caught (err) {
    bruh "Error handled: " + err
} frfr {
    bruh "This always runs - cleanup time!"
}

// Nested error handling
bruh "\\nNested error handling:"
manifest {
    manifest {
        bruh "Inner try block"
        drama "Inner error"
    } caught (inner) {
        bruh "Inner catch: " + inner
        drama "Outer error triggered"
    }
} caught (outer) {
    bruh "Outer catch: " + outer
} frfr {
    bruh "Final cleanup complete!"
}`,

            comprehensive: `// ZLang - All Gen Z Keywords Demo
bruh "=== ZLang Comprehensive Demo ==="

// Variables (bet)
bet name = "Gen Z Coder"
bet age = 20
bet is_coding = fr

bruh "Programmer: " + name
bruh "Age: " + age

// If/Else If/Else (sus/lowkey sus/no sus)
bet score = 95
sus (score >= 90) {
    bruh "A+ grade! Absolutely crushing it!"
} lowkey sus (score >= 80) {
    bruh "B grade! Still pretty solid!"
} no sus {
    bruh "Time to hit the books harder!"
}

// Simple loops
bruh "\\n=== Loop Demo ==="
bet counter = 1
lowkey (counter <= 3) {
    bruh "Counter: " + counter
    bet counter = counter + 1
}

// Switch statement (vibecheck)
bruh "\\n=== Vibe Check (Switch) ==="
bet mood = "excited"
vibecheck (mood) {
    "happy": {
        bruh "You're glowing!"
    }
    "excited": {
        bruh "Your energy is contagious!"
    }
    default: {
        bruh "Whatever mood, you're valid!"
    }
}

// Functions (flex/vibe)
flex calculateGrade(score) {
    sus (score >= 90) {
        vibe "A+"
    } lowkey sus (score >= 80) {
        vibe "B"
    } no sus {
        vibe "F"
    }
}

bet grade = calculateGrade(87)
bruh "Your grade is: " + grade

bruh "\\n=== All Keywords Working! ZLang hits different! ==="`
        };
        
        // Initialize CodeMirror
        let editor;
        
        function initCodeMirror() {
            const textarea = document.getElementById('code-editor');
            
            // Define ZLang mode
            CodeMirror.defineMode("zlang", function(config) {
                const zlangKeywords = {
                    'bet': 'zlang-keyword',
                    'fr': 'zlang-keyword', 
                    'cap': 'zlang-keyword',
                    'sus': 'zlang-keyword',
                    'lowkey': 'zlang-keyword',
                    'grind': 'zlang-keyword',
                    'slay': 'zlang-keyword',
                    'flex': 'zlang-keyword',
                    'vibe': 'zlang-keyword',
                    'bruh': 'zlang-keyword',
                    'manifest': 'zlang-keyword',
                    'caught': 'zlang-keyword',
                    'drama': 'zlang-keyword',
                    'frfr': 'zlang-keyword',
                    'vibecheck': 'zlang-keyword',
                    'ghost': 'zlang-keyword',
                    'no': 'zlang-keyword'
                };
                
                return {
                    startState: function() {
                        return {inString: false, inComment: false};
                    },
                    token: function(stream, state) {
                        // Handle comments
                        if (stream.match(/\/\/.*/)) {
                            return "zlang-comment";
                        }
                        
                        // Handle strings
                        if (stream.match(/"(?:[^"\\\\]|\\\\.)*"/)) {
                            return "zlang-string";
                        }
                        
                        // Handle numbers
                        if (stream.match(/\b\d+\.?\d*\b/)) {
                            return "zlang-number";
                        }
                        
                        // Handle operators
                        if (stream.match(/[+\-*/=<>!&|]+/)) {
                            return "zlang-operator";
                        }
                        
                        // Handle keywords
                        const word = stream.current();
                        if (stream.match(/\b(lowkey sus|no sus|no chill)\b/)) {
                            return "zlang-keyword";
                        }
                        
                        if (stream.match(/\w+/)) {
                            const word = stream.current();
                            if (zlangKeywords[word]) {
                                return zlangKeywords[word];
                            }
                        }
                        
                        stream.next();
                        return null;
                    }
                };
            });
            
            editor = CodeMirror.fromTextArea(textarea, {
                mode: "zlang",
                lineNumbers: true,
                theme: "default",
                indentUnit: 4,
                lineWrapping: true,
                extraKeys: {
                    "Ctrl-Z": function(cm) { cm.undo(); },
                    "Tab": function(cm) { cm.replaceSelection("    "); }
                }
            });
            
            // Set initial content
            editor.setValue(examples.hello);
        }
        
        function loadExample(type) {
            if (editor) {
                editor.setValue(examples[type]);
            }
        }
        
        async function runCode() {
            const code = editor ? editor.getValue() : '';
            const output = document.getElementById('output');
            const runBtn = document.getElementById('runBtn');
            
            // Debug: Show what code we're trying to send
            console.log('Code to execute:', code);
            console.log('Code length:', code.length);
            
            if (!code || code.trim() === '') {
                output.textContent = 'Please enter some ZLang code first!';
                output.style.color = '#ff6b6b';
                return;
            }
            
            runBtn.disabled = true;
            runBtn.textContent = 'Running...';
            output.textContent = 'Executing ZLang code...';
            output.style.color = '#f8f8f2';
            
            try {
                const requestBody = JSON.stringify({ code: code });
                console.log('Sending request body:', requestBody);
                
                const response = await fetch(window.location.origin + '/execute', {
                    method: 'POST',
                    headers: {
                        'Content-Type': 'application/json',
                    },
                    body: requestBody
                });
                
                const result = await response.json();
                console.log('Received response:', result);
                
                if (result.success) {
                    output.textContent = result.output;
                    output.style.color = '#f8f8f2';
                } else {
                    output.textContent = 'Error: ' + result.error;
                    output.style.color = '#ff6b6b';
                }
            } catch (error) {
                console.error('Network error:', error);
                output.textContent = 'Network Error: ' + error.message;
                output.style.color = '#ff6b6b';
            }
            
            runBtn.disabled = false;
            runBtn.textContent = 'Run Code';
        }
        
        function clearEditor() {
            const output = document.getElementById('output');
            if (editor) {
                editor.setValue('// Write your own ZLang code here!\n// Use Gen Z slang keywords like bruh, bet, sus, fr, etc.\n');
            }
            output.textContent = 'Ready for your code! 🚀';
            output.style.color = '#4a90e2';
        }
        
        // Initialize CodeMirror when page loads
        document.addEventListener('DOMContentLoaded', function() {
            initCodeMirror();
        });
    </script>
</body>
</html>"#.to_string()
}