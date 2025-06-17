use std::path::Path;
use std::io::{self};
use std::io::Write;
use std::process::Command;
use std::env;
use std::collections::HashMap;

use rustyline::{error, Result};

struct Cmd<'a> {
    exec: &'a str,
    args: Vec<&'a str>,
}

fn parse<'a>(command: Option<&'a str>, args: Vec<&'a str>) -> Option<Cmd<'a>> {
    match command {
        Some(cmd) => Some(Cmd {
            exec: cmd,
            args,
        }),
        None => None,
    }
}

fn execute_command_with_lifetime(cmd: &Cmd, env: &mut HashMap<String, String>) -> io::Result<()> {
    match cmd.exec {
        "export" => {
            for (key,value) in env {
                println!("{}={}", key,value);
            }
            Ok(())
        }
        "exit" => {
            println!("Goodbye!");
            std::process::exit(0);
        }
        "cd" => {
            let path = cmd.args.first().unwrap_or(&".");
            if let Err(e) = std::env::set_current_dir(Path::new(path)) {
                eprintln!("cd: {}", e);
            }
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
                    Ok(())
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
        print!("TinyRush: ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(_) => {
                let input = input.trim();
                if input.is_empty() {
                    continue;
                }

                let mut parts = input.split_whitespace();
                let command = parts.next();
                let args = parts.collect::<Vec<&str>>();

                if let Some(cmd) = parse(command, args) {
                    if let Err(e) = execute_command_with_lifetime(&cmd, & mut env) {
                        eprintln!("Error executing command: {}", e);
                    }
                }
            }
            Err(e) => {
                eprintln!("Error reading input: {}", e);
                break;
            }
        }
    }
}
