use std::env;
use std::fs;
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
            Err(_) => {
                let path = args.last().unwrap();
                match find_executable(path, key) {
                    Ok(exe) => println!("{} is {}", key, exe),
                    Err(err) => println!("{}", err),
                }
            }
        }
    }),
];

static NOT_FOUND: Executor = |name, _args| {
    println!("{}: command not found", name);
};

fn find_builtin(key: &str) -> Result<usize, usize> {
    BUILTINS.binary_search_by(|(k, _)| k.cmp(&key))
}

fn find_executable(path: &str, name: &str) -> io::Result<String> {
    for dir in path.split(':') {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                if let Ok(file_type) = entry.file_type() {
                    if file_type.is_dir() {
                        match find_executable(entry.path().to_str().unwrap(), name) {
                            Ok(exe) => {
                                return Ok(exe);
                            }
                            Err(_) => {
                                continue;
                            }
                        }
                    }

                    if file_type.is_file() && entry.file_name() == name {
                        let exe = entry.path().into_os_string().into_string().unwrap();
                        return Ok(exe);
                    }
                }
            }
        }
    }
    io::Result::Err(io::Error::new(
        io::ErrorKind::NotFound,
        format!("{} not found", name),
    ))
}

fn main() {
    let path = env::var("PATH").unwrap_or("".to_string());

    // Wait for user input
    let stdin = io::stdin();
    let mut input = String::new();

    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        stdin.read_line(&mut input).unwrap();
        let mut split = input.split_whitespace();
        let cmd = split.next().unwrap_or("");
        let mut args: Vec<&str> = split.collect();

        if cmd == "type" {
            args.push(path.as_str());
        }

        find_builtin(cmd).map_or(NOT_FOUND, |idx| BUILTINS[idx].1)(cmd, args);

        input.clear();
    }
}
