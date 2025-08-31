use crate::errors::son_error::SoNError;
use std::fmt::{Display, Formatter};

pub struct Lexer {
    pub input: String,
    position: usize,
}

impl Display for Lexer {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.input.get(self.position..) {
            Some(slice) => write!(f, "{}", slice),
            None => write!(f, "<invalid position>"),
        }
    }
}

impl Lexer {
    pub fn from_string(input: String) -> Lexer {
        Lexer { input, position: 0 }
    }

    pub fn from_str(input: &str) -> Lexer {
        Lexer::from_string(String::from(input))
    }

    pub fn position(&self) -> usize {
        self.position
    }

    pub fn dbg_position(&self) -> Option<(usize, usize)> {
        self.line_col_for(self.position())
    }

    pub fn dbg_position_string(&self) -> String {
        if let Some((line, column)) = self.dbg_position() {
            format!("{}:{}", line, column)
        } else {
            "out of bounds".into()
        }
    }

    pub fn line_col_for(&self, position: usize) -> Option<(usize, usize)> {
        let mut line_start = 0;
        for (line_number, line) in self.input.lines().enumerate() {
            let line_len = line.len();
            if position <= line_start + line_len {
                let column = position - line_start;
                return Some((line_number + 1, column + 1)); // 1-based
            }
            // '\n'
            line_start += line_len + 1;
        }
        None
    }


    pub fn is_eof(&self) -> bool { self.position >= self.input.len() }

    pub fn peek(&self) -> Option<char> { self.input.chars().nth(self.position) }

    pub fn next_char(&mut self) -> Option<char> {
        self.peek().inspect(|_| self.position += 1)
    }

    pub fn is_whitespace(&self) -> bool {
        self.peek().map(|c| c.is_whitespace()).unwrap_or(false)
    }

    pub fn skip_whitespace(&mut self) {
        while self.is_whitespace() {
            self.next_char();
        }
    }

    /// Does NOT change self.
    pub fn peek_matsch(&mut self, syntax: &str) -> bool {
        let prev_position = self.position;
        if self.matsch(syntax) {
            self.position = prev_position;
            true
        } else {
            false
        }
    }

    /// Does NOT change self.
    pub fn peek_matschx(&mut self, syntax: &str) -> bool {
        let prev_position = self.position;
        if self.matschx(syntax) {
            self.position = prev_position;
            true
        } else {
            false
        }
    }

    /// Return true, if we find "syntax" after skipping white space; also
    /// then advance the cursor past syntax.
    /// Return false otherwise, and do not advance the cursor.
    pub fn matsch(&mut self, syntax: &str) -> bool {
        self.skip_whitespace();
        if self.input[self.position..].starts_with(syntax) {
            self.position += syntax.len();
            true
        } else {
            false
        }
    }

    pub fn matschx(&mut self, syntax: &str) -> bool {
        if !self.matsch(syntax) {
            return false;
        }
        if self.peek().is_some_and(|ch| Lexer::is_id_letter(&ch)) {
            self.position -= syntax.len();
            return false;
        }
        true
    }


    // Used for errors
    pub fn dbg_get_any_next_token(&mut self) -> String {
        if self.is_eof() {
            return String::new();
        }

        let ch = match self.peek() {
            Some(c) => c,
            None => return "$unexpected EOF$".to_string(),
        };

        if Lexer::is_id_start(&ch) {
            return self.parse_id();
        }
        if Lexer::is_number(&ch) {
            return self.parse_number_string();
        }
        ch.to_string()
    }

    pub fn parse_number(&mut self) -> Result<i64, SoNError> {
        let snum = self.parse_number_string();
        if snum.len() > 1 && snum.chars().nth(0).is_some_and(|c| c.eq(&'0')) {
            return Err(SoNError::NumberCannotStartWith0);
        }
        Ok(snum.parse::<i64>().expect("numbers must start with a digit"))
    }

    fn parse_number_string(&mut self) -> String {
        let start = self.position;
        while let Some(c) = self.next_char() {
            if !Lexer::is_number(&c) {
                // Step back one position so we don't consume this non‑ID char
                self.position -= 1;
                break;
            }
        }
        self.input[start..self.position].to_string()
    }

    pub fn parse_id(&mut self) -> String {
        let start = self.position;

        while let Some(c) = self.next_char() {
            if !Lexer::is_id_letter(&c) {
                // Step back one position so we don't consume this non‑ID char
                self.position -= 1;
                break;
            }
        }

        self.input[start..self.position].to_string()
    }

    // All characters of an identifier, e.g. "_x123"
    fn is_id_letter(ch: &char) -> bool {
        ch.is_alphanumeric() || ch.eq(&'_')
    }

    fn is_number(ch: &char) -> bool {
        ch.is_digit(10)
    }

    pub fn peek_is_number(&self) -> bool {
        self.peek().is_some_and(|c| Lexer::is_number(&c))
    }

    // First letter of an identifier
    pub fn is_id_start(ch: &char) -> bool {
        ch.is_alphabetic() || ch.eq(&'_')
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_lex_dbg() {
        // Arrange
        let mut lexer = Lexer::from_str("1230");

        // Act
        let token = lexer.dbg_get_any_next_token();

        // Assert
        assert_eq!("1230", token);
        assert_eq!(4, lexer.position);
    }

    #[test]
    fn should_lex_number() {
        // Arrange
        let mut lexer = Lexer::from_str("1230");

        // Act
        let token = lexer.parse_number();

        // Assert
        assert_eq!(1230, token.unwrap());
        assert_eq!(4, lexer.position);
    }

    #[test]
    fn should_lex_number_but_stop_at_non_number() {
        // Arrange
        let mut lexer = Lexer::from_str("123a");

        // Act
        let token = lexer.parse_number();

        // Assert
        assert_eq!(123, token.unwrap());
        assert_eq!(3, lexer.position);
    }

    #[test]
    #[should_panic(expected = "numbers must start with a digit")]
    fn should_fail_when_parse_number_is_called_without_checking_for_number_first() {
        // Arrange
        let mut lexer = Lexer::from_str("a123");

        // Act
        let _ = lexer.parse_number();
    }

    #[test]
    fn should_match_loosely() {
        // Arrange
        let mut lexer = Lexer::from_str("waitaminute");

        // Act
        let m = lexer.matsch(&"wait".to_string());

        // Assert
        assert!(m);
        assert_eq!(4, lexer.position);
    }

    #[test]
    fn should_match_exactly() {
        // Arrange
        let mut lexer = Lexer::from_str("waitaminute");

        // Act
        let m = lexer.matschx("wait");

        // Assert
        assert!(!m);
        assert_eq!(0, lexer.position);
    }

    #[test]
    fn should_match_still_exactly_for_non_id_letters() {
        // Arrange
        let mut lexer = Lexer::from_str("wait!aminute");

        // Act
        let m = lexer.matschx("wait");

        // Assert
        assert!(m);
        assert_eq!(4, lexer.position);
    }

    #[test]
    fn should_get_line_col() {
        // Arrange
        let mut lexer = Lexer::from_str("01\n34\n6");
        lexer.position = 4;

        // Act
        let result = lexer.dbg_position_string();

        // Assert
        assert_eq!("2:2", result);
    }

    #[test]
    fn should_get_line_col_oob() {
        // Arrange
        let mut lexer = Lexer::from_str("01\n34\n6");
        lexer.position = 123;

        // Act
        let result = lexer.dbg_position_string();

        // Assert
        assert_eq!("out of bounds", result);
    }

    #[test]
    fn should_parse_zero_number() {
        // Arrange
        let mut lexer = Lexer::from_str("0");

        // Act
        let result = lexer.parse_number().unwrap();

        // Assert
        assert_eq!(0, result);
    }
}

