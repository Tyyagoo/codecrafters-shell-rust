#[allow(unused_imports)]
use std::io::{self, Write};

type Executor = fn(name: &str, args: Vec<&str>) -> ();
static BUILTINS: &[(&str, Executor)] = &[
    ("echo", |_name, args| {
        println!("{}", args.join(" "));
    }),
    ("exit", |_name, _args| {
        std::process::exit(0);
    }),
    ("type", |_name, args| {
        let key = match args.first() {
            Some(k) => k,
            None => {
                println!("USAGE: type <COMMAND>");
                return;
            }
        };

        match find_builtin(key) {
            Ok(idx) => println!("{} is a shell builtin", BUILTINS[idx].0),
            Err(_) => println!("{} not found", key),
        }
    }),
];

static NOT_FOUND: Executor = |name, _args| {
    println!("{}: command not found", name);
};

fn find_builtin(key: &str) -> Result<usize, usize> {
    BUILTINS.binary_search_by(|(k, _)| k.cmp(&key))
}

fn main() {
    // Wait for user input
    let stdin = io::stdin();
    let mut input = String::new();

    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        stdin.read_line(&mut input).unwrap();
        let mut split = input.split_whitespace();
        let cmd = split.next().unwrap_or("");
        let args: Vec<&str> = split.collect();

        find_builtin(cmd).map_or(NOT_FOUND, |idx| BUILTINS[idx].1)(cmd, args);

        input.clear();
    }
}
