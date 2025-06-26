use std::path::Path;
use std::process::Command;
use std::env;
use std::collections::HashMap;
use rustyline::error::ReadlineError;
use rustyline::{DefaultEditor, Result};

struct Cmd<'a> {
    exec: &'a str,
    args: Vec<&'a str>,
}

static mut ERROR_CODE: u32 = 0;

fn set_error(code: u32) {
    unsafe {
        ERROR_CODE = code;
    }
}

fn expand_variables(input: &str, vars: &HashMap<String, String>) -> String {
    let mut result = String::new();
    let mut chars = input.chars().peekable();
    let mut in_single_quotes = false;
    
    while let Some(ch) = chars.next() {
        match ch {
            '\'' => {
                in_single_quotes = !in_single_quotes;
                result.push(ch);
            }
            '$' if !in_single_quotes => {
                if let Some(&next_ch) = chars.peek() {
                    if next_ch == '{' {
                        chars.next();
                        let var_name = extract_var_name_braced(&mut chars);
                        if let Some(value) = vars.get(&var_name) {
                            result.push_str(value);
                        }
                    } else if next_ch.is_alphabetic() || next_ch == '_' {
                        let var_name = extract_var_name_simple(&mut chars);
                        if let Some(value) = vars.get(&var_name) {
                            result.push_str(value);
                        }
                    } else {
                        if next_ch == '?' {
                            unsafe {
                                let code = char::from_digit(ERROR_CODE,10);
                                result.push(code.unwrap());
                            }
                            chars.next();
                        } else {
                            result.push(ch);
                        }
                    }
                } else {
                    result.push(ch);
                }
            }
            _ => {
                result.push(ch);
            }
        }
    }
    
    result
}

fn extract_var_name_braced(chars: &mut std::iter::Peekable<std::str::Chars>) -> String {
    let mut var_name = String::new();
    while let Some(&ch) = chars.peek() {
        if ch == '}' {
            chars.next();
            break;
        } else if ch.is_alphanumeric() || ch == '_' {
            var_name.push(chars.next().unwrap());
        } else {
            break;
        }
    }
    var_name
}

fn extract_var_name_simple(chars: &mut std::iter::Peekable<std::str::Chars>) -> String {
    let mut var_name = String::new();
    while let Some(&ch) = chars.peek() {
        if ch.is_alphanumeric() || ch == '_' {
            var_name.push(chars.next().unwrap());
        } else {
            break;
        }
    }
    var_name
}

impl <'a> FromIterator<&'a str> for Cmd<'a> {
    fn from_iter<I: IntoIterator<Item=&'a str>>(iter: I) -> Self {
        let mut iter = iter.into_iter();
        let cmd = iter.next().unwrap();
        let args = iter.collect::<Vec<&str>>();
        Cmd {
            exec: cmd,
            args: args,
        }
    }
}

fn execute_command_with_lifetime(cmd: &Cmd, env: &mut HashMap<String, String>) -> Result<i32> {
    match cmd.exec {
        "export" => {
            if cmd.args.len() == 0 {
                for (key,value) in env {
                    println!("{}='{}'", key,value);
                }
            } else {
                for arg in &cmd.args {
                    let var = arg.to_owned();
                    let mut split = var.split('=');
                    let key = split.next();
                    let value = split.next().unwrap_or_else(|| "");
                    _ = match key {
                        Some(k) => {
                            env.insert(k.into(), value.to_string());
                        }
                        _ => {}   
                    };

                }
            }
            set_error(0);
            Ok(0)
        }
        "exit" => {
            println!("Goodbye!");
            Ok(-1)
        }
        "cd" => {
            let home = env.get("HOME")
                .map(|s| s.to_string())
                .unwrap_or_else(|| "/".to_string());
            let path = match cmd.args.first() {
                Some(path) => {
                    path.to_string()
                },
                _ => {
                    home
                },
            };
            let current_dir = std::env::current_dir().unwrap();
            if let Err(e) = std::env::set_current_dir(Path::new(path.as_str())) {
                eprintln!("cd: {}", e);
                set_error(1);
                return Err(ReadlineError::Interrupted);
            }
            env.insert("OLD_PWD".to_string(), current_dir.into_os_string().into_string().unwrap());
            env.insert("PWD".to_string(), path.to_string());
            Ok(0)
        }
        "unset" => {
            for arg in &cmd.args {
                env.remove(arg.to_owned());
            }
            Ok(0)
        }
        "pwd" => {
            match std::env::current_dir() {
                Ok(path) => println!("{}", path.display()),
                Err(e) => eprintln!("pwd: {}", e),
            }
            Ok(0)
        }
        "echo" => {
            println!("{}", cmd.args.join(" "));
            Ok(0)
        }
        _ => {
            let mut command = Command::new(cmd.exec);
            command.args(&cmd.args);

            match command.spawn() {
                Ok(mut child) => {
                    match child.wait() {
                        Ok(_) => Ok(0),
                        Err(e) => {
                            eprintln!("Error waiting for process: {}", e);
                            Ok(1)
                        }
                    }
                }
                Err(e) => {
                    eprintln!("{}: command not found ({})", cmd.exec, e);
                    set_error(e.raw_os_error().unwrap() as u32);
                    Err(ReadlineError::Interrupted)
                }
            }
        }
    }
}

fn main() {
    println!("Welcome to TinyRush! Type 'exit' to quit.");
    let mut env = HashMap::new();
    for (key, value) in env::vars() {
        env.insert(
            key,
            value
        );
    }

    let mut rl = match DefaultEditor::new() {
        Ok(l) => l,   // Return the editor instance
        Err(_) => return,
    };
    let history_file = format!("{}/.tinyrush_history",
    env.get("HOME").unwrap_or(&".".to_string()));
    if rl.load_history(&history_file).is_err() {
        println!("No previous history.");
    }
    loop {
        let current_dir = std::env::current_dir().unwrap();
        let flag = format!("-> {}: ", current_dir.display());
        let readline = rl.readline(&flag[..]);
        match readline {
            Ok(input) => {
                _ = rl.add_history_entry(input.as_str());
                let mut input = input.trim();
                if input.is_empty() {
                    continue;
                }
                let expanded = expand_variables(&input[..], &env);
                input = &expanded;
                let parts = input.split_whitespace();
                let cmd = Cmd::from_iter(parts);

                _ = match execute_command_with_lifetime(&cmd, & mut env) {
                    Ok(num) => {
                        match num {
                           -1 => break,
                           _ => continue,
                        }
                    },
                    _ => {},
                };
            }
            Err(e) => {
                eprintln!("Error reading input: {}", e);
                break;
            }
        }
    }
    _ = rl.save_history(&history_file);
}
