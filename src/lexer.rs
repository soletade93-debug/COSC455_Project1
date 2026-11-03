// These are the functions our lexer needs to implement.
pub trait LexicalAnalyzer {
    fn get_char(&mut self) -> char;
    fn add_char(&mut self, c: char);
    fn lookup(&self, s: &str) -> bool;
}

// This struct keeps track of the source code and where the lexer is reading.
pub struct Lexer {
    source: Vec<char>,
    position: usize,
    current_lexeme: String,
}

impl Lexer {
    // Set up the lexer with the source code we want to read.
    pub fn new(source: &str) -> Self {
        Self {
            // Turn the source code into a list of characters.
            source: source.chars().collect(),
            // Start reading from the first character.
            position: 0,
            // This will hold the token currently being read.
            current_lexeme: String::new(),
        }
    }

    // This function reads the source code and returns the next token.
    pub fn next_token_result(&mut self) -> Result<Option<String>, String> {
        // Clear the previous token before reading the next one.
        self.current_lexeme.clear();

        // Check if we have reached the end of the source code.
        if self.position >= self.source.len() {
            return Ok(None);
        }

        // An annotation always starts with #.
        if self.source[self.position] == '#' {
            // These are the annotations that our lexer accepts.
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

            // Get the part of the source code starting at our current position.
            let remaining: String =
                self.source[self.position..].iter().collect();

            // Check if the remaining source starts with a valid annotation.
            for annotation in annotations {
                // Convert to uppercase so annotations are not case sensitive.
                if remaining
                    .to_ascii_uppercase()
                    .starts_with(annotation)
                {
                    let end = self.position + annotation.chars().count();

                    // Make sure the annotation ends at the end of the file
                    // or is followed by whitespace.
                    if end == self.source.len()
                        || self.source[end].is_whitespace()
                    {
                        // Read each character of the annotation.
                        for _ in annotation.chars() {
                            let c = self.get_char();
                            self.add_char(c);
                        }

                        // Return the annotation we just found.
                        return Ok(Some(self.current_lexeme.clone()));
                    }
                }
            }

            // Save the position where the invalid annotation starts.
            let error_position = self.position;

            // Read the invalid annotation so we can include it in the error.
            while self.position < self.source.len()
                && !self.source[self.position].is_whitespace()
            {
                let c = self.get_char();
                self.add_char(c);
            }

            // Report the invalid annotation and where it was found.
            return Err(format!(
                "Lexical error: invalid annotation '{}' at character position {}",
                self.current_lexeme, error_position
            ));
        }

        // If the character is not #, keep reading normal text.
        // Stop when we reach the next annotation.
        while self.position < self.source.len()
            && self.source[self.position] != '#'
        {
            let c = self.get_char();
            self.add_char(c);
        }

        // Return the text token we just collected.
        Ok(Some(self.current_lexeme.clone()))
    }

    // This is the simpler version that returns a String.
    pub fn next_token(&mut self) -> String {
        // Check what next_token_result returned.
        match self.next_token_result() {
            // Return the token if one was found.
            Ok(Some(token)) => token,

            // Return an empty string when there are no more tokens.
            Ok(None) => String::new(),

            // Print the error if an invalid annotation was found.
            Err(message) => {
                eprintln!("{}", message);
                String::new()
            }
        }
    }

    // Check whether a token is one of the valid annotations.
    pub fn lookup_annotation(&self, token: &str) -> bool {
        self.lookup(token)
    }
}

// Here we implement the LexicalAnalyzer trait from the professor.
impl LexicalAnalyzer for Lexer {

    // Get the next character from the source code.
    fn get_char(&mut self) -> char {
        if self.position < self.source.len() {
            // Save the character at the current position.
            let character = self.source[self.position];

            // Move forward to the next character.
            self.position += 1;

            character
        } else {
            // Return this when we reach the end of the source.
            '\0'
        }
    }

    // Add a character to the token we are currently building.
    fn add_char(&mut self, c: char) {
        self.current_lexeme.push(c);
    }

    // Check whether the given token matches a valid annotation.
    fn lookup(&self, token: &str) -> bool {
        // List of annotations that are accepted by the lexer.
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

        // Compare the token against the list without worrying about capitalization.
        annotations
            .iter()
            .any(|valid| valid.eq_ignore_ascii_case(token))
    }
}