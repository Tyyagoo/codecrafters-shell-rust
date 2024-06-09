#[allow(unused_imports)]
use std::io::{self, Write};

fn main() {
    // Wait for user input
    let stdin = io::stdin();
    let mut input = String::new();

    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        stdin.read_line(&mut input).unwrap();
        let mut command = input.split_whitespace();

        match command.next() {
            Some("exit") => {
                break;
            }

            Some("echo") => {
                let args: Vec<&str> = command.collect();
                println!("{}", args.join(" "));
            }

            cmd => {
                println!("{}: command not found", cmd.unwrap_or(""));
            }
        }

        input.clear();
    }
}
