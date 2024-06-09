use std::env;
use std::fs;
#[allow(unused_imports)]
use std::io::{self, Write};

type Executor = fn(name: &str, args: Vec<&str>, path: &str) -> ();
static BUILTINS: &[(&str, Executor)] = &[
    ("cd", |_name, args, _path| {
        match args.first() {
            Some(dest) => {
                let home = env::var("HOME").unwrap();
                match fs::canonicalize(dest.replace('~', home.as_str())) {
                    Ok(buf) => {
                        env::set_current_dir(buf).unwrap();
                    }
                    Err(_) => {
                        println!("cd: {}: No such file or directory", dest);
                    }
                }
            }
            None => {
                println!("USAGE: cd <DIR>");
            }
        };
    }),
    ("echo", |_name, args, _path| {
        println!("{}", args.join(" "));
    }),
    ("exit", |_name, _args, _path| {
        // TODO: use exit status
        std::process::exit(0);
    }),
    ("pwd", |_name, _args, _path| {
        let path = env::current_dir().unwrap();
        println!("{}", path.display());
    }),
    ("type", |_name, args, path| {
        let key = match args.first() {
            Some(k) => k,
            None => {
                println!("USAGE: type <COMMAND>");
                return;
            }
        };

        match find_builtin(key) {
            Ok(idx) => println!("{} is a shell builtin", BUILTINS[idx].0),
            Err(_) => match find_executable(path, key) {
                Some(exe) => println!("{} is {}", key, exe),
                None => println!("{} not found", key),
            },
        };
    }),
];

static NOT_BUILTIN: Executor = |name, args, path| match find_executable(path, name) {
    Some(exe) => {
        let out = std::process::Command::new(exe)
            .args(args)
            .output()
            .expect("Failed to start process");

        print!("{}", String::from_utf8(out.stdout).unwrap());
        io::stdout().flush().unwrap();
    }

    None => println!("{}: command not found", name),
};

fn find_builtin(key: &str) -> Result<usize, usize> {
    BUILTINS.binary_search_by(|(k, _)| k.cmp(&key))
}

fn find_executable(path: &str, name: &str) -> Option<String> {
    for dir in path.split(':') {
        let entries = match fs::read_dir(dir) {
            Ok(e) => e,
            Err(_) => continue,
        };

        for entry in entries.flatten() {
            let file_type = match entry.file_type() {
                Ok(ft) => ft,
                Err(_) => continue,
            };

            if file_type.is_dir() {
                if let Some(exe) = find_executable(entry.path().to_str().unwrap(), name) {
                    return Some(exe);
                }
            }

            // TODO: verify executable flag.
            if file_type.is_file() && entry.file_name() == name {
                return entry.path().to_str().map(|s| s.to_string());
            }
        }
    }

    None
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
        let args: Vec<&str> = split.collect();

        find_builtin(cmd).map_or(NOT_BUILTIN, |idx| BUILTINS[idx].1)(cmd, args, path.as_str());

        input.clear();
    }
}
