use std::char;

pub struct Scanner {
    cursor: usize,
    characters: Vec<char>,
}

impl Scanner {
    pub fn new(string: &str) -> Self {
        Self {
            cursor: 0,
            characters: string.chars().collect(),
        }
    }

    pub fn peek(&self) -> Option<&char> {
        self.characters.get(self.cursor)
    }

    pub fn pop(&mut self) -> Option<&char> {
        match self.characters.get(self.cursor) {
            Some(ch) => {
                self.cursor += 1;
                Some(ch)
            }

            None => None,
        }
    }

    pub fn take(&mut self, what: char) -> bool {
        match self.characters.get(self.cursor) {
            Some(ch) if *ch == what || *ch == '\n' => {
                self.cursor += 1;
                true
            }

            _ => false,
        }
    }

    pub fn unquote(&mut self, what: char) -> String {
        let mut chars = Vec::new();

        while let Some(current) = self.pop() {
            let ch = *current;

            if ch == '\n' {
                break;
            }

            if ch == what {
                match self.peek() {
                    Some(' ') => break,
                    None => break,
                    _ => continue,
                }
            }

            if ch == '\\' && what == '"' {
                match self.peek() {
                    Some(esc) if *esc == '\\' || *esc == '$' || *esc == '"' || *esc == '\n' => {
                        chars.push(*esc);
                        self.cursor += 1;
                        continue;
                    }

                    Some(_) => {},
                    None => break,
                }
            }

            chars.push(ch);

        }

        chars.into_iter().collect()
    }

    pub fn take_until(&mut self, what: char) -> String {
        let mut chars = Vec::new();
        while !self.take(what) {
            match self.pop() {
                Some(ch) if *ch == '\\' => {
                    match self.pop() {
                        Some(escaped) => chars.push(*escaped),
                        None => break,
                    }
                }
                Some(ch) => chars.push(*ch),
                None => break,
            }
        }
        chars.into_iter().collect()
    }

    pub fn next(&mut self) -> String {
        let expr = if self.take('\'') {
            self.unquote('\'')
        } else if self.take('"') {
            self.unquote('"')
        } else {
            self.take_until(' ')
        };

        expr
    }
}

pub fn parse(string: &str) -> (String, Vec<String>) {
    let mut scanner = Scanner::new(string);
    let cmd  = scanner.next();

    let mut args: Vec<String> = Vec::new();
    while let Some(_) = scanner.peek() {
        while scanner.take(' ') {}
        args.push(scanner.next());
    }

    (cmd, args)
}