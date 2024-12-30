use std::env;
use std::fs;
use std::fs::File;
use std::io::{self, Write};

mod parser;
use parser::parse;

type Executor = fn(name: &str, args: Vec<String>, path: &str) -> ();
static BUILTINS: &[(&str, Executor)] = &[
    ("cd", |_name, args, _path| {
        let dest = match args.first() {
            Some(d) => d,
            None => {
                println!("USAGE: cd <DIR>");
                return;
            }
        };

        let home = env::var("HOME").unwrap();
        match fs::canonicalize(dest.replace('~', home.as_str())) {
            Ok(buf) => env::set_current_dir(buf).unwrap(),
            Err(_) => println!("cd: {}: No such file or directory", dest),
        }
    }),
    ("echo", |_name, args, _path| {
        match args.iter().enumerate().find(|(_, s)| s.contains('>')) {
            Some((idx, target)) => {
                let (cmd_args, pipe_to) = args.split_at(idx);
                let mut file = File::create(pipe_to[1].to_owned()).unwrap();
                let args_str = format!("{}\n", cmd_args.join(" "));

                match target.chars().nth(0) {
                    Some('1') | Some('>') => file.write_all(args_str.as_bytes()).unwrap(),
                    Some('2') => print!("{}", args_str),
                    _ => panic!(),
                }

                io::stdout().flush().unwrap();
            }

            None => println!("{}", args.join(" ")),
        }
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
    Some(exe) => match args.iter().enumerate().find(|(_, s)| s.contains('>')) {
        Some((idx, target)) => {
            let (cmd_args, pipe_to) = args.split_at(idx);

            let out = std::process::Command::new(exe)
                .args(cmd_args)
                .output()
                .expect("Failed to start process");

            let mut file = File::create(pipe_to[1].to_owned()).unwrap();

            match target.chars().nth(0) {
                Some('1') | Some('>') => {
                    file.write_all(out.stdout.as_ref()).unwrap();

                    // Huh?
                    let error = String::from_utf8(out.stderr).unwrap();
                    match error.strip_prefix("/bin/") {
                        Some(e) => print!("{}", e),
                        None => print!("{}", error),
                    }
                }

                Some('2') => {
                    print!("{}", String::from_utf8(out.stdout).unwrap());

                    // Huh?
                    let error = String::from_utf8(out.stderr).unwrap();
                    match error.strip_prefix("/bin/") {
                        Some(e) => file.write_all(e.as_bytes()).unwrap(),
                        None => file.write_all(error.as_bytes()).unwrap(),
                    }
                }

                _ => panic!("Invalid pipe."),
            }

            io::stdout().flush().unwrap();
            io::stderr().flush().unwrap();
        }

        None => {
            let out = std::process::Command::new(exe)
                .args(args)
                .output()
                .expect("Failed to start process");

            print!("{}", String::from_utf8(out.stdout).unwrap());
            io::stdout().flush().unwrap();
        }
    },

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
        let (cmd, args) = parse(&input);

        let fun = find_builtin(cmd.as_str()).map_or(NOT_BUILTIN, |idx| BUILTINS[idx].1);

        fun(cmd.as_str(), args, path.as_str());

        input.clear();
    }
}
