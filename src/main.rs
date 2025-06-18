use std::path::Path;
use std::io::{self};
use std::io::Write;
use std::process::Command;
use std::env;
use std::collections::HashMap;
use rustyline::error::ReadlineError;
use rustyline::{Result};

struct Cmd<'a> {
    exec: &'a str,
    args: Vec<&'a str>,
}

impl <'a> FromIterator<&'a str> for Cmd<'a> {
    fn from_iter<I: IntoIterator<Item=&'a str>>(iter: I) -> Self {
        let mut iter = iter.into_iter();
        let cmd = iter.next().unwrap();
        Cmd {
            exec: cmd,
            args: iter.collect::<Vec<&str>>(),
        }
    }
}

fn execute_command_with_lifetime(cmd: &Cmd, env: &mut HashMap<String, String>) -> Result<()> {
    match cmd.exec {
        "export" => {
            if cmd.args.len() == 0 {
                for (key,value) in env {
                    println!("{}='{}'", key,value);
                }
            } else {
                for &arg in &cmd.args {
                    _ = arg.find("=");
                }
            }
            Ok(())
        }
        "exit" => {
            println!("Goodbye!");
            std::process::exit(0);
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
                return Err(ReadlineError::Interrupted);
            }
            env.insert("OLD_PWD".to_string(), current_dir.into_os_string().into_string().unwrap());
            env.insert("PWD".to_string(), path.to_string());
            Ok(())
        }
        "pwd" => {
            match std::env::current_dir() {
                Ok(path) => println!("{}", path.display()),
                Err(e) => eprintln!("pwd: {}", e),
            }
            Ok(())
        }
        "echo" => {
            println!("{}", cmd.args.join(" "));
            Ok(())
        }
        _ => {
            let mut command = Command::new(cmd.exec);
            command.args(&cmd.args);

            match command.spawn() {
                Ok(mut child) => {
                    match child.wait() {
                        Ok(_) => Ok(()),
                        Err(e) => {
                            eprintln!("Error waiting for process: {}", e);
                            Ok(())
                        }
                    }
                }
                Err(e) => {
                    eprintln!("{}: command not found ({})", cmd.exec, e);
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

    loop {
        let current_dir = std::env::current_dir().unwrap();
        print!("-> {}: ", current_dir.into_os_string().into_string().unwrap());
        io::stdout().flush().unwrap();

        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(_) => {
                let input = input.trim();
                if input.is_empty() {
                    continue;
                }

                let parts = input.split_whitespace();
                let cmd = Cmd::from_iter(parts);

                if let Err(e) = execute_command_with_lifetime(&cmd, & mut env) {
                    eprintln!("Error executing command: {}", e);
                }
            }
            Err(e) => {
                eprintln!("Error reading input: {}", e);
                break;
            }
        }
    }
}
