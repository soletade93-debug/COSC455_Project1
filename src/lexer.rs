
pub trait LexicalAnalyzer {
    fn get_char(&mut self) -> char;
    fn add_char(&mut self, c: char);
    fn lookup(&self, s: &str) -> bool;
}

pub struct Lexer {
    source: Vec<char>,
    position: usize,
    current_lexeme: String,
}

impl Lexer {
    pub fn new(source: &str) -> Self {
        Self {
            source: source.chars().collect(),
            position: 0,
            current_lexeme: String::new(),
        }
    }

    pub fn next_token_result(&mut self) -> Result<Option<String>, String> {
        self.current_lexeme.clear();

        if self.position >= self.source.len() {
            return Ok(None);
        }

        if self.source[self.position] == '#' {
            let annotations = [
                "#MAEK PARAGRAF",
                "#MAEK HEAD",
                "#GIMMEH TITLE",
                "#GIMMEH BOLD",
                "#GIMMEH ITEM",
                "#GIMMEH LINX",
                "#GIMMEH VAR",
                "#MAEK LIST",
                "#HAI",
                "#KBYE",
                "#OBTW",
                "#TLDR",
                "#MKAY",
                "#OIC",
                "#NEWLINE",
                "#IHAZ",
                "#ITIZ",
            ];

            let remaining: String =
                self.source[self.position..].iter().collect();

            for annotation in annotations {
                if remaining
                    .to_ascii_uppercase()
                    .starts_with(annotation)
                {
                    let end = self.position + annotation.chars().count();

                    if end == self.source.len()
                        || self.source[end].is_whitespace()
                    {
                        for _ in annotation.chars() {
                            let c = self.get_char();
                            self.add_char(c);
                        }

                        return Ok(Some(self.current_lexeme.clone()));
                    }
                }
            }

            let error_position = self.position;

            while self.position < self.source.len()
                && !self.source[self.position].is_whitespace()
            {
                let c = self.get_char();
                self.add_char(c);
            }

            return Err(format!(
                "Lexical error: invalid annotation '{}' at character position {}",
                self.current_lexeme, error_position
            ));
        }

        while self.position < self.source.len()
            && self.source[self.position] != '#'
        {
            let c = self.get_char();
            self.add_char(c);
        }

        Ok(Some(self.current_lexeme.clone()))
    }

    pub fn next_token(&mut self) -> String {
        match self.next_token_result() {
            Ok(Some(token)) => token,
            Ok(None) => String::new(),
            Err(message) => {
                eprintln!("{}", message);
                String::new()
            }
        }
    }

    pub fn lookup_annotation(&self, token: &str) -> bool {
        self.lookup(token)
    }
}

impl LexicalAnalyzer for Lexer {
    fn get_char(&mut self) -> char {
        if self.position < self.source.len() {
            let character = self.source[self.position];
            self.position += 1;
            character
        } else {
            '\0'
        }
    }

    fn add_char(&mut self, c: char) {
        self.current_lexeme.push(c);
    }

    fn lookup(&self, token: &str) -> bool {
        let annotations = [
            "#HAI",
            "#KBYE",
            "#OBTW",
            "#TLDR",
            "#MAEK HEAD",
            "#GIMMEH TITLE",
            "#MKAY",
            "#OIC",
            "#MAEK PARAGRAF",
            "#GIMMEH BOLD",
            "#MAEK LIST",
            "#GIMMEH ITEM",
            "#NEWLINE",
            "#GIMMEH LINX",
            "#IHAZ",
            "#ITIZ",
            "#GIMMEH VAR",
        ];

        annotations
            .iter()
            .any(|valid| valid.eq_ignore_ascii_case(token))
    }
}
