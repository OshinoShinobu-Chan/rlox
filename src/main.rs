use clap::Parser;
use std::fs;
use std::io::Write;

mod error;
mod scanner;
mod token;
mod token_type;

use error::Error;
use scanner::Scanner;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Arg {
    pub script: Option<String>,
}

fn main() {
    let args = Arg::parse();
    if let Some(script) = args.script {
        run_file(script);
    } else {
        run_prompt();
    }
}

fn run_file(script: String) {
    let contents = fs::read_to_string(script).expect("Something went wrong reading the file");
    if let Err(e) = run(contents) {
        println!("{e}");
    }
}

fn run_prompt() {
    let handle_in = std::io::stdin();
    let mut handle_out = std::io::stdout();
    loop {
        print!(">>> ");
        handle_out.flush().unwrap();
        let mut input = String::new();
        handle_in.read_line(&mut input).unwrap();
        if input.is_empty() {
            break;
        }
        if let Err(e) = run(input) {
            println!("{e}");
        }
    }
}

fn run(source: String) -> Result<(), Error> {
    let mut scan = Scanner::new(&source);
    let tokens = scan.scan_tokens();
    if let Err(e) = tokens {
        return Err(e);
    }

    for token in tokens.unwrap() {
        print!("{:?}", token);
    }
    Ok(())
}
