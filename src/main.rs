
mod lexer;

use lexer::Lexer;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

// Professor's Syntax Analyzer Trait - Option 2.
pub trait SyntaxAnalyzer {
    fn parse_lolcode(&mut self) -> Result<(), String>;
    fn parse_head(&mut self) -> Result<(), String>;
    fn parse_title(&mut self) -> Result<(), String>;
    fn parse_comment(&mut self) -> Result<(), String>;
    fn parse_body(&mut self) -> Result<(), String>;
    fn parse_paragraph(&mut self) -> Result<(), String>;
    fn parse_inner_paragraph(&mut self) -> Result<(), String>;
    fn parse_inner_text(&mut self) -> Result<(), String>;
    fn parse_variable_define(&mut self) -> Result<(), String>;
    fn parse_variable_use(&mut self) -> Result<(), String>;
    fn parse_bold(&mut self) -> Result<(), String>;
    fn parse_list(&mut self) -> Result<(), String>;
    fn parse_list_items(&mut self) -> Result<(), String>;
    fn parse_inner_list(&mut self) -> Result<(), String>;
    fn parse_link(&mut self) -> Result<(), String>;
    fn parse_newline(&mut self) -> Result<(), String>;
    fn parse_text(&mut self) -> Result<(), String>;
}

// Nodes represent the parsed contents of the source program.
#[derive(Debug)]
enum Node {
    Text(String),
    Comment(String),
    Paragraph(Vec<Node>),
    Bold(String),
    List(Vec<Node>),
    Item(Vec<Node>),
    Link(String),
    Newline,
}

struct Parser {
    tokens: Vec<String>,
    position: usize,
    scopes: Vec<HashMap<String, String>>,
    nodes: Vec<Node>,
}

impl Parser {
    fn new(tokens: Vec<String>) -> Self {
        Self {
            tokens,
            position: 0,
            scopes: vec![HashMap::new()],
            nodes: Vec::new(),
        }
    }

    fn current(&self) -> &str {
        self.tokens
            .get(self.position)
            .map(|token| token.as_str())
            .unwrap_or("")
    }

    fn is(&self, expected: &str) -> bool {
        self.current().eq_ignore_ascii_case(expected)
    }

    fn consume(&mut self) {
        if self.position < self.tokens.len() {
            self.position += 1;
        }
    }

    // Ignore whitespace-only tokens between annotations.
    fn skip_whitespace(&mut self) {
        while self.position < self.tokens.len()
            && self.current().trim().is_empty()
        {
            self.consume();
        }
    }

    fn expect(&mut self, expected: &str) -> Result<(), String> {
        self.skip_whitespace();

        if self.is(expected) {
            self.consume();
            Ok(())
        } else {
            Err(format!(
                "Syntax error: expected {}, found '{}'",
                expected,
                self.current().trim()
            ))
        }
    }

    fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn pop_scope(&mut self) {
        if self.scopes.len() > 1 {
            self.scopes.pop();
        }
    }

    fn define_variable(&mut self, name: &str, value: &str) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.to_string(), value.to_string());
        }
    }

    fn get_variable(&self, name: &str) -> Result<String, String> {
        for scope in self.scopes.iter().rev() {
            if let Some(value) = scope.get(name) {
                return Ok(value.clone());
            }
        }

        Err(format!(
            "Static semantic error: variable '{}' was used before it was defined",
            name
        ))
    }

    // Read one text token from the lexer.
    fn take_text(&mut self) -> Result<String, String> {
        self.skip_whitespace();

        if self.position >= self.tokens.len() {
            return Err(
                "Syntax error: expected text but reached end of file".to_string()
            );
        }

        if self.current().starts_with('#') {
            return Err(format!(
                "Syntax error: expected text but found '{}'",
                self.current()
            ));
        }

        let value = self.current().trim().to_string();
        self.consume();

        if value.is_empty() {
            return Err("Syntax error: expected non-empty text".to_string());
        }

        Ok(value)
    }

    fn parse_definition(&mut self) -> Result<(), String> {
        self.expect("#IHAZ")?;

        let name = self.take_text()?;

        if name.split_whitespace().count() != 1 {
            return Err(
                "Syntax error: variable name must be one word".to_string()
            );
        }

        self.expect("#ITIZ")?;

        let value = self.take_text()?;

        self.expect("#MKAY")?;

        self.define_variable(&name, &value);

        Ok(())
    }

    fn parse_use(&mut self) -> Result<String, String> {
        self.expect("#GIMMEH VAR")?;

        let name = self.take_text()?;

        if name.split_whitespace().count() != 1 {
            return Err(
                "Syntax error: variable name must be one word".to_string()
            );
        }

        self.expect("#OIC")?;

        self.get_variable(&name)
    }

    fn parse_comment_node(&mut self) -> Result<Node, String> {
        self.expect("#OBTW")?;

        let mut comment = String::new();

        while !self.is("#TLDR") {
            if self.position >= self.tokens.len() {
                return Err(
                    "Syntax error: comment is missing #TLDR".to_string()
                );
            }

            if self.current().starts_with('#') {
                return Err(
                    "Syntax error: annotation found inside comment".to_string()
                );
            }

            comment.push_str(self.current());
            self.consume();
        }

        self.expect("#TLDR")?;

        Ok(Node::Comment(comment))
    }

    fn parse_title_node(&mut self) -> Result<String, String> {
        self.expect("#GIMMEH TITLE")?;

        let title = self.take_text()?;

        self.expect("#OIC")?;

        Ok(title)
    }

    fn parse_head_node(&mut self) -> Result<(), String> {
        self.expect("#MAEK HEAD")?;

        self.push_scope();

        self.skip_whitespace();

        while self.is("#OBTW") {
            let _comment = self.parse_comment_node()?;
            self.skip_whitespace();
        }

        if self.is("#GIMMEH TITLE") {
            let title = self.parse_title_node()?;

            self.nodes.push(Node::Text(format!(
                "<head><title>{}</title></head>",
                title
            )));
        } else {
            self.nodes.push(Node::Text("<head></head>".to_string()));
        }

        self.expect("#MKAY")?;

        self.pop_scope();

        Ok(())
    }

    fn parse_bold_node(&mut self) -> Result<Node, String> {
        self.expect("#GIMMEH BOLD")?;

        let text = self.take_text()?;

        self.expect("#OIC")?;

        Ok(Node::Bold(text))
    }

    fn parse_link_node(&mut self) -> Result<Node, String> {
        self.expect("#GIMMEH LINX")?;

        let address = self.take_text()?;

        self.expect("#OIC")?;

        Ok(Node::Link(address))
    }

    fn parse_item_node(&mut self) -> Result<Node, String> {
        self.expect("#GIMMEH ITEM")?;

        self.push_scope();

        let mut item_nodes = Vec::new();

        self.skip_whitespace();

        // A variable definition may be the first annotation in an item.
        if self.is("#IHAZ") {
            self.parse_definition()?;
        }

        loop {
            self.skip_whitespace();

            if self.is("#OIC") {
                break;
            }

            if self.position >= self.tokens.len() {
                return Err(
                    "Syntax error: list item is missing #OIC".to_string()
                );
            }

            if self.is("#GIMMEH BOLD") {
                let bold = self.parse_bold_node()?;
                item_nodes.push(bold);
            } else if self.is("#GIMMEH VAR") {
                let value = self.parse_use()?;
                item_nodes.push(Node::Text(value));
            } else if self.is("#OBTW") {
                let comment = self.parse_comment_node()?;
                item_nodes.push(comment);
            } else if self.current().starts_with('#') {
                return Err(format!(
                    "Syntax error: unexpected annotation '{}' inside list item",
                    self.current()
                ));
            } else {
                item_nodes.push(Node::Text(self.current().to_string()));
                self.consume();
            }
        }

        self.expect("#OIC")?;

        self.pop_scope();

        Ok(Node::Item(item_nodes))
    }

    fn parse_list_node(&mut self) -> Result<Node, String> {
        self.expect("#MAEK LIST")?;

        self.push_scope();

        let mut items = Vec::new();

        self.skip_whitespace();

        if !self.is("#GIMMEH ITEM") {
            return Err(
                "Syntax error: #MAEK LIST must be followed by #GIMMEH ITEM"
                    .to_string()
            );
        }

        while self.is("#GIMMEH ITEM") {
            let item = self.parse_item_node()?;
            items.push(item);
            self.skip_whitespace();
        }

        self.expect("#MKAY")?;

        self.pop_scope();

        Ok(Node::List(items))
    }

    fn parse_paragraph_node(&mut self) -> Result<Node, String> {
        self.expect("#MAEK PARAGRAF")?;

        self.push_scope();

        let mut paragraph_nodes = Vec::new();

        self.skip_whitespace();

        // A variable definition must be the first annotation in the paragraph.
        if self.is("#IHAZ") {
            self.parse_definition()?;
        }

        loop {
            self.skip_whitespace();

            if self.is("#MKAY") {
                break;
            }

            if self.position >= self.tokens.len() {
                return Err(
                    "Syntax error: paragraph is missing #MKAY".to_string()
                );
            }

            if self.is("#MAEK PARAGRAF") {
                return Err(
                    "Syntax error: paragraphs cannot be nested".to_string()
                );
            } else if self.is("#GIMMEH BOLD") {
                let bold = self.parse_bold_node()?;
                paragraph_nodes.push(bold);
            } else if self.is("#MAEK LIST") {
                let list = self.parse_list_node()?;
                paragraph_nodes.push(list);
            } else if self.is("#NEWLINE") {
                self.consume();
                paragraph_nodes.push(Node::Newline);
            } else if self.is("#GIMMEH LINX") {
                let link = self.parse_link_node()?;
                paragraph_nodes.push(link);
            } else if self.is("#GIMMEH VAR") {
                let value = self.parse_use()?;
                paragraph_nodes.push(Node::Text(value));
            } else if self.is("#OBTW") {
                let comment = self.parse_comment_node()?;
                paragraph_nodes.push(comment);
            } else if self.current().starts_with('#') {
                return Err(format!(
                    "Syntax error: unexpected annotation '{}' inside paragraph",
                    self.current()
                ));
            } else {
                paragraph_nodes.push(Node::Text(self.current().to_string()));
                self.consume();
            }
        }

        self.expect("#MKAY")?;

        self.pop_scope();

        Ok(Node::Paragraph(paragraph_nodes))
    }

    fn parse_body_nodes(&mut self) -> Result<(), String> {
        loop {
            self.skip_whitespace();

            if self.position >= self.tokens.len() || self.is("#KBYE") {
                break;
            }

            if self.is("#OBTW") {
                let comment = self.parse_comment_node()?;
                self.nodes.push(comment);
            } else if self.is("#MAEK PARAGRAF") {
                let paragraph = self.parse_paragraph_node()?;
                self.nodes.push(paragraph);
            } else if self.is("#GIMMEH BOLD") {
                let bold = self.parse_bold_node()?;
                self.nodes.push(bold);
            } else if self.is("#MAEK LIST") {
                let list = self.parse_list_node()?;
                self.nodes.push(list);
            } else if self.is("#NEWLINE") {
                self.consume();
                self.nodes.push(Node::Newline);
            } else if self.is("#GIMMEH LINX") {
                let link = self.parse_link_node()?;
                self.nodes.push(link);
            } else if self.is("#GIMMEH VAR") {
                let value = self.parse_use()?;
                self.nodes.push(Node::Text(value));
            } else if self.is("#IHAZ") {
                self.parse_definition()?;
            } else if self.current().starts_with('#') {
                return Err(format!(
                    "Syntax error: unexpected annotation '{}'",
                    self.current()
                ));
            } else {
                self.nodes.push(Node::Text(self.current().to_string()));
                self.consume();
            }
        }

        Ok(())
    }

    fn render_nodes(nodes: &[Node]) -> String {
        let mut html = String::new();

        for node in nodes {
            match node {
                Node::Text(text) => html.push_str(text),

                Node::Comment(text) => {
                    html.push_str("<!--");
                    html.push_str(text);
                    html.push_str("-->");
                }

                Node::Paragraph(children) => {
                    html.push_str("<p>");
                    html.push_str(&Self::render_nodes(children));
                    html.push_str("</p>");
                }

                Node::Bold(text) => {
                    html.push_str("<b>");
                    html.push_str(text);
                    html.push_str("</b>");
                }

                Node::List(items) => {
                    html.push_str("<ul>");
                    html.push_str(&Self::render_nodes(items));
                    html.push_str("</ul>");
                }

                Node::Item(children) => {
                    html.push_str("<li>");
                    html.push_str(&Self::render_nodes(children));
                    html.push_str("</li>");
                }

                Node::Link(address) => {
                    html.push_str("<a href=\"");
                    html.push_str(address);
                    html.push_str("\">");
                    html.push_str(address);
                    html.push_str("</a>");
                }

                Node::Newline => html.push_str("<br>"),
            }
        }

        html
    }

    fn generate_html(&self) -> String {
        let mut html = String::from("<html>");

        html.push_str(&Self::render_nodes(&self.nodes));

        html.push_str("</html>");

        html
    }
}

impl SyntaxAnalyzer for Parser {
    fn parse_lolcode(&mut self) -> Result<(), String> {
        self.expect("#HAI")?;

        self.skip_whitespace();

        while self.is("#OBTW") {
            let comment = self.parse_comment_node()?;
            self.nodes.push(comment);
            self.skip_whitespace();
        }

        if self.is("#MAEK HEAD") {
            self.parse_head_node()?;
        }

        self.parse_body_nodes()?;

        self.expect("#KBYE")?;

        // Ignore spaces and newlines after the final annotation.
        self.skip_whitespace();

        if self.position < self.tokens.len() {
            return Err(format!(
                "Syntax error: unexpected content after #KBYE: '{}'",
                self.current().trim()
            ));
        }

        Ok(())
    }

    fn parse_head(&mut self) -> Result<(), String> {
        self.parse_head_node()
    }

    fn parse_title(&mut self) -> Result<(), String> {
        let title = self.parse_title_node()?;
        self.nodes.push(Node::Text(title));
        Ok(())
    }

    fn parse_comment(&mut self) -> Result<(), String> {
        let comment = self.parse_comment_node()?;
        self.nodes.push(comment);
        Ok(())
    }

    fn parse_body(&mut self) -> Result<(), String> {
        self.parse_body_nodes()
    }

    fn parse_paragraph(&mut self) -> Result<(), String> {
        let paragraph = self.parse_paragraph_node()?;
        self.nodes.push(paragraph);
        Ok(())
    }

    fn parse_inner_paragraph(&mut self) -> Result<(), String> {
        self.parse_paragraph()
    }

    fn parse_inner_text(&mut self) -> Result<(), String> {
        self.parse_text()
    }

    fn parse_variable_define(&mut self) -> Result<(), String> {
        self.parse_definition()
    }

    fn parse_variable_use(&mut self) -> Result<(), String> {
        let value = self.parse_use()?;
        self.nodes.push(Node::Text(value));
        Ok(())
    }

    fn parse_bold(&mut self) -> Result<(), String> {
        let bold = self.parse_bold_node()?;
        self.nodes.push(bold);
        Ok(())
    }

    fn parse_list(&mut self) -> Result<(), String> {
        let list = self.parse_list_node()?;
        self.nodes.push(list);
        Ok(())
    }

    fn parse_list_items(&mut self) -> Result<(), String> {
        while self.is("#GIMMEH ITEM") {
            let item = self.parse_item_node()?;
            self.nodes.push(item);
            self.skip_whitespace();
        }

        Ok(())
    }

    fn parse_inner_list(&mut self) -> Result<(), String> {
        self.parse_list_items()
    }

    fn parse_link(&mut self) -> Result<(), String> {
        let link = self.parse_link_node()?;
        self.nodes.push(link);
        Ok(())
    }

    fn parse_newline(&mut self) -> Result<(), String> {
        self.expect("#NEWLINE")?;
        self.nodes.push(Node::Newline);
        Ok(())
    }

    fn parse_text(&mut self) -> Result<(), String> {
        if self.current().starts_with('#') || self.current().is_empty() {
            return Err("Syntax error: expected text".to_string());
        }

        self.nodes.push(Node::Text(self.current().to_string()));
        self.consume();

        Ok(())
    }
}

// Get all tokens from the lexer.
fn tokenize(source: &str) -> Result<Vec<String>, String> {
    let mut lexer = Lexer::new(source);
    let mut tokens = Vec::new();

    loop {
        match lexer.next_token_result()? {
            Some(token) => tokens.push(token),
            None => break,
        }
    }

    Ok(tokens)
}

// Open the generated HTML file in the default browser.
fn open_browser(file_path: &Path) {
    #[cfg(target_os = "windows")]
    {
        let path = file_path.display().to_string();

        let _ = Command::new("cmd")
            .args(["/C", "start", "", &path])
            .spawn();
    }

    #[cfg(target_os = "macos")]
    {
        let _ = Command::new("open")
            .arg(file_path)
            .spawn();
    }

    #[cfg(target_os = "linux")]
    {
        let _ = Command::new("xdg-open")
            .arg(file_path)
            .spawn();
    }
}

fn compile_file(file_name: &str) -> Result<(), String> {
    let input_path = Path::new(file_name);

    if input_path.extension().and_then(|ext| ext.to_str()) != Some("lol") {
        return Err(
            "Usage error: input file must have a .lol extension".to_string()
        );
    }

    let source = fs::read_to_string(input_path)
        .map_err(|error| format!("File error: {}", error))?;

    println!("Starting lexical analysis...");

    let tokens = tokenize(&source)?;

    println!("Lexical analysis completed.");
    println!("Starting syntax and semantic analysis...");

    let mut parser = Parser::new(tokens);

    parser.parse_lolcode()?;

    println!("Syntax and semantic analysis completed.");

    // Create the HTML file only after all analysis succeeds.
    let html = parser.generate_html();

    let output_path = input_path.with_extension("html");

    fs::write(&output_path, html)
        .map_err(|error| format!("Output file error: {}", error))?;

    println!(
        "Compilation successful. HTML created: {}",
        output_path.display()
    );

    open_browser(&output_path);

    Ok(())
}

fn main() {
    // Required project variable.
    let dolphin = "LOLCODE Markdown compiler";

    let arguments: Vec<String> = env::args().collect();

    if arguments.len() != 2 {
        eprintln!("Usage: cargo run -- <input_file.lol>");
        return;
    }

    println!("Running {}", dolphin);

    if let Err(error) = compile_file(&arguments[1]) {
        eprintln!("{}", error);
        eprintln!("Compilation failed.");
    }
}
