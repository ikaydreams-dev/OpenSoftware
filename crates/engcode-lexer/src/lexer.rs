use crate::token::{Token, TokenWithPosition};

pub struct Lexer {
    input: Vec<char>,
    position: usize,
    line: usize,
    column: usize,
}

impl Lexer {
    pub fn new(input: String) -> Self {
        Self {
            input: input.chars().collect(),
            position: 0,
            line: 1,
            column: 1,
        }
    }

    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();

        while !self.is_at_end() {
            self.skip_whitespace_except_newlines();

            if self.is_at_end() {
                break;
            }

            if let Some(token) = self.next_token() {
                tokens.push(token);
            }
        }

        tokens.push(Token::EOF);
        tokens
    }

    pub fn tokenize_with_positions(&mut self) -> Vec<TokenWithPosition> {
        let mut tokens = Vec::new();

        while !self.is_at_end() {
            self.skip_whitespace_except_newlines();

            if self.is_at_end() {
                break;
            }

            let start_line = self.line;
            let start_column = self.column;

            if let Some(token) = self.next_token() {
                tokens.push(TokenWithPosition {
                    token,
                    line: start_line,
                    column: start_column,
                });
            }
        }

        tokens.push(TokenWithPosition {
            token: Token::EOF,
            line: self.line,
            column: self.column,
        });
        tokens
    }

    fn next_token(&mut self) -> Option<Token> {
        let ch = self.current_char()?;

        // Handle comments
        if ch == '#' {
            // Skip until end of line
            while !self.is_at_end() && self.current_char() != Some('\n') {
                self.advance();
            }
            // Skip the newline too if present
            if self.current_char() == Some('\n') {
                self.advance();
            }
            return self.next_token(); // Get next token after comment
        }

        match ch {
            '\n' => {
                self.advance();
                Some(Token::Newline)
            }
            ',' => {
                self.advance();
                Some(Token::Comma)
            }
            '=' => {
                self.advance();
                Some(Token::Equals)
            }
            '[' => {
                self.advance();
                Some(Token::LeftBracket)
            }
            ']' => {
                self.advance();
                Some(Token::RightBracket)
            }
            '(' => {
                self.advance();
                Some(Token::LeftParen)
            }
            ')' => {
                self.advance();
                Some(Token::RightParen)
            }
            '.' => {
                self.advance();
                Some(Token::Dot)
            }
            '-' => {
                self.advance();
                Some(Token::Minus)
            }
            '+' => {
                self.advance();
                Some(Token::Plus)
            }
            '*' => {
                self.advance();
                Some(Token::Star)
            }
            '/' => {
                self.advance();
                Some(Token::Slash)
            }
            ':' => {
                self.advance();
                Some(Token::Colon)
            }
            '{' => {
                self.advance();
                Some(Token::LeftBrace)
            }
            '}' => {
                self.advance();
                Some(Token::RightBrace)
            }
            '"' => Some(self.read_string()),
            '0'..='9' => Some(self.read_number()),
            'a'..='z' | 'A'..='Z' | '_' => Some(self.read_word()),
            _ => {
                self.advance();
                None
            }
        }
    }

    fn read_string(&mut self) -> Token {
        self.advance(); // skip opening quote

        let mut value = String::new();

        while !self.is_at_end() && self.current_char() != Some('"') {
            value.push(self.current_char().unwrap());
            self.advance();
        }

        if !self.is_at_end() {
            self.advance(); // skip closing quote
        }

        Token::String(value)
    }

    fn read_number(&mut self) -> Token {
        let mut num = String::new();

        while !self.is_at_end() {
            if let Some(ch) = self.current_char() {
                if ch.is_numeric() || ch == '.' {
                    num.push(ch);
                    self.advance();
                } else {
                    break;
                }
            }
        }

        Token::Number(num.parse().unwrap_or(0.0))
    }

    fn read_word(&mut self) -> Token {
        let mut word = String::new();

        while !self.is_at_end() {
            if let Some(ch) = self.current_char() {
                if ch.is_alphanumeric() || ch == '_' {
                    word.push(ch);
                    self.advance();
                } else {
                    break;
                }
            }
        }

        Token::from_word(&word)
    }

    fn skip_whitespace_except_newlines(&mut self) {
        while !self.is_at_end() {
            if let Some(ch) = self.current_char() {
                if ch == ' ' || ch == '\t' || ch == '\r' {
                    self.advance();
                } else {
                    break;
                }
            }
        }
    }

    fn current_char(&self) -> Option<char> {
        if self.position < self.input.len() {
            Some(self.input[self.position])
        } else {
            None
        }
    }

    fn advance(&mut self) {
        if let Some(ch) = self.current_char() {
            if ch == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
        }
        self.position += 1;
    }

    fn is_at_end(&self) -> bool {
        self.position >= self.input.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_simple_database() {
        let input = r#"create a database called "MyDB""#;
        let mut lexer = Lexer::new(input.to_string());
        let tokens = lexer.tokenize();

        assert_eq!(tokens[0], Token::Create);
        assert_eq!(tokens[1], Token::A);
        assert_eq!(tokens[2], Token::Database);
        assert_eq!(tokens[3], Token::Called);
        assert_eq!(tokens[4], Token::String("MyDB".to_string()));
        assert_eq!(tokens[5], Token::EOF);
    }

    #[test]
    fn test_tokenize_with_newlines() {
        let input = "create\ncollections\nin\nit";
        let mut lexer = Lexer::new(input.to_string());
        let tokens = lexer.tokenize();

        assert_eq!(tokens[0], Token::Create);
        assert_eq!(tokens[1], Token::Newline);
        assert_eq!(tokens[2], Token::Collections);
        assert_eq!(tokens[3], Token::Newline);
        assert_eq!(tokens[4], Token::In);
        assert_eq!(tokens[5], Token::Newline);
        assert_eq!(tokens[6], Token::It);
    }

    #[test]
    fn test_tokenize_identifiers() {
        let input = "may april march";
        let mut lexer = Lexer::new(input.to_string());
        let tokens = lexer.tokenize();

        assert_eq!(tokens[0], Token::Identifier("may".to_string()));
        assert_eq!(tokens[1], Token::Identifier("april".to_string()));
        assert_eq!(tokens[2], Token::Identifier("march".to_string()));
    }

    #[test]
    fn test_tokenize_numbers() {
        let input = "42 3.14 100";
        let mut lexer = Lexer::new(input.to_string());
        let tokens = lexer.tokenize();

        assert_eq!(tokens[0], Token::Number(42.0));
        assert_eq!(tokens[1], Token::Number(3.14));
        assert_eq!(tokens[2], Token::Number(100.0));
    }

    #[test]
    fn test_tokenize_variables() {
        let input = r#"set name to "Alice""#;
        let mut lexer = Lexer::new(input.to_string());
        let tokens = lexer.tokenize();

        assert_eq!(tokens[0], Token::Set);
        assert_eq!(tokens[1], Token::Identifier("name".to_string()));
        assert_eq!(tokens[2], Token::To);
        assert_eq!(tokens[3], Token::String("Alice".to_string()));
    }

    #[test]
    fn test_tokenize_crud() {
        let input = "insert into users";
        let mut lexer = Lexer::new(input.to_string());
        let tokens = lexer.tokenize();

        assert_eq!(tokens[0], Token::Insert);
        assert_eq!(tokens[1], Token::Into);
        assert_eq!(tokens[2], Token::Identifier("users".to_string()));
    }

    #[test]
    fn test_tokenize_booleans() {
        let input = "true false null";
        let mut lexer = Lexer::new(input.to_string());
        let tokens = lexer.tokenize();

        assert_eq!(tokens[0], Token::True);
        assert_eq!(tokens[1], Token::False);
        assert_eq!(tokens[2], Token::Null);
    }

    #[test]
    fn test_tokenize_equals() {
        let input = "let x = 5";
        let mut lexer = Lexer::new(input.to_string());
        let tokens = lexer.tokenize();

        assert_eq!(tokens[0], Token::Let);
        assert_eq!(tokens[1], Token::Identifier("x".to_string()));
        assert_eq!(tokens[2], Token::Equals);
        assert_eq!(tokens[3], Token::Number(5.0));
    }

    #[test]
    fn test_line_tracking() {
        let input = "create\na\ndatabase";
        let mut lexer = Lexer::new(input.to_string());
        let tokens = lexer.tokenize_with_positions();

        assert_eq!(tokens[0].line, 1);
        assert_eq!(tokens[1].line, 1); // newline
        assert_eq!(tokens[2].line, 2);
        assert_eq!(tokens[3].line, 2); // newline
        assert_eq!(tokens[4].line, 3);
    }
}
