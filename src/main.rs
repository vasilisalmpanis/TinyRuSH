use rustyline::error::ReadlineError;
use rustyline::{DefaultEditor, Result};

fn parse_line(line: String) -> i32 {
    let parts = line.split(' ');
    let collection = parts.collect::<Vec<&str>>();
    dbg!(collection);
    0
}

fn main() -> Result<()> {
    let mut rl = DefaultEditor::new()?;
    // TODO: change signal handlers for CTRL-C, CTRL-\
    loop {
        let readline = rl.readline("tinyRuSH: >> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str())?;
                parse_line(line);
            },
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break
            },
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break
            },
            Err(err) => {
                println!("Error: {:?}", err);
                break
            }
        }
    }
    Ok(())
}
