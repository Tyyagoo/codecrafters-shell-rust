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
        match input.trim() {
            "exit 0" => {
                break;
            }

            cmd => {
                println!("{}: command not found", cmd);
            }
        }

        input.clear();
    }
}
