use crate::compiler::Compiler;
use std::fmt;
use std::fs;
#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Core keywords
    Let,
    Mut,
    Equal,
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    LParen,
    RParen,
    If,
    Else,
    Elif,
    LBrace,
    RBrace,
    Less,
    Greater,
    EqualEqual,
    NotEqual,
    While,
    Semicolon,
    Comma,
    Not,
    DQuote,
    Print,
    Identifier(String),
    Integer(i64),
    DataType(String),
    String(String),
    ILLEGAL,
    EOF,
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let  temp_string:String; 
        let s = match self {
            TokenKind::Let => "let",
            TokenKind::Mut => "mut",
            TokenKind::If => "if",
            TokenKind::Elif => "elif",
            TokenKind::Else => "else",
            TokenKind::While => "while",
            TokenKind::Print => "print",
            TokenKind::LParen => "(",
            TokenKind::RParen => ")",
            TokenKind::LBrace => "{",
            TokenKind::RBrace => "}",
            TokenKind::Semicolon => ";",
            TokenKind::Comma => ",",
            TokenKind::DQuote => "\"",
            TokenKind::Not => "!",
            TokenKind::Equal => "=",
            TokenKind::Plus => "+",
            TokenKind::Minus => "-",
            TokenKind::Star => "*",
            TokenKind::Slash => "/",
            TokenKind::Percent => "%",
            TokenKind::EqualEqual => "==",
            TokenKind::NotEqual => "!=",
            TokenKind::Less => "<",
            TokenKind::Greater => ">",
            TokenKind::Identifier(ident) => {
                temp_string = format!("identifier {ident}");
                temp_string.as_str()
            },
            TokenKind::Integer(num) => {
                temp_string = format!("{num}");
                temp_string.as_str()
            },
            TokenKind::String(s) => s.as_str(),
            TokenKind::DataType(datatype) => datatype.as_str(),
            TokenKind::ILLEGAL => "[ILLEGAL CHARACTER]",
            TokenKind::EOF => "[END OF FILE]",
        };
        write!(f, "{s}")
    }
}
pub struct Lexer<'a> {
    compiler: &'a Compiler,
    input: Vec<char>,
    position: usize, // Index of the current char in the input
    line_no: usize,  // Current line; starts at 1; Only 0 when the input is empty
    col_no: usize,   // Current position in line; starts at 1; Only 0 when the input is empty or
    // when last line(when reached by line_no) is empty
    isfile: bool, // True if the input is taken from a file
    file_id: u16, // Used only if the isfile is true, Otherwise 0
    reached_eof: bool, // For iterator
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub line_no: usize,
    pub col_no: usize,
    pub isfile: bool,
    pub file_id: u16,
}

impl<'a> Lexer<'a> {
    pub fn new(compiler: &'a Compiler, input: &str) -> Self {
        let line_no = if input.is_empty() { 0 } else { 1 };
        Self {
            compiler: &compiler,
            input: input.to_string().chars().collect(),
            position: 0,
            line_no: line_no,
            col_no: line_no, // same rule apply to line_no and col_no (only at beginning)
            isfile: false,
            file_id: 0,
            reached_eof: false,
        }
    }

    pub fn from_file(compiler: &'a Compiler, file_id: u16) -> Option<Self> {
        if compiler.is_file_present(file_id) == false {
            return None;
        }
        match fs::read_to_string(compiler.get_filename(file_id)?.as_str()).ok() {
            Some(input) => {
                let line_no = if input.is_empty() { 0 } else { 1 };
                Some(Self {
                    compiler: &compiler,
                    input: input.chars().collect(),
                    position: 0,
                    line_no: line_no,
                    col_no: line_no, // same rule apply to line_no and col_no (only at beginning)
                    isfile: true,
                    file_id: file_id,
                    reached_eof: false,
                })
            }
            None => None,
        }
    }

    fn current(&self) -> Option<char> {
        self.input.get(self.position).copied()
    }

    fn peek(&self) -> Option<char> {
        self.input.get(self.position + 1).copied()
    }

    fn advance(&mut self) {
        if let Some(c) = self.current() {
            if c == '\n' {
                self.line_no += 1;
                if let Some(_n) = self.peek() {
                    self.col_no = 1;
                } else {
                    self.col_no = 0;
                }
            } else {
                self.col_no += 1;
            }
        }
        self.position += 1;
    }
    fn expect_char(&mut self, ept_char: char) -> bool {
        if let Some(c) = self.current() {
            if c == ept_char {
                self.advance();
                return true;
            }
        }
        return false;
    }
    fn expect_str(&mut self, ept_str: &str) -> bool {
        if &self.input.len() - self.position < ept_str.len() {
            /*
             * Many programmers usually(at a glance) think that in the above if statement, self.position should
             * be  (self.position+1) as self.position starts at 0. But that assumption is not true.
             * To clear this out, take an example: there are 3 char in self.input. self.position is
             * 1 ( which means pointing to 2nd char) and ept_str len is 2
             * Solve this and find out with and without your assumption and see what you get
             */
            return false;
        }
        let s: String = self.input[self.position..self.position + ept_str.len()]
            .iter()
            .collect();
        if s.contains(ept_str) {
            for _ in 0..s.len() {
                self.advance();
            }
            return true;
        }
        return false;
    }
    fn skip_whitespace(&mut self) {
        while let Some(character) = self.current() {
            if character.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }
    fn read_number(&mut self) -> Option<i64> {
        let start = self.position;
        while let Some(character) = self.current() {
            if !character.is_ascii_digit() {
                if character.is_alphabetic() {
                    return None;
                }
                break;
            }
            self.advance();
        }
        let end = self.position;
        let s: String = (&self.input[start..end]).iter().collect();
        s.parse().ok()
    }

    fn read_string(&mut self) -> Option<String> {
        if self.expect_char('"') == false {
            return None;
        }
        let start = self.position;
        let mut prev_character = '"';
        let mut found_ending_quote = false;
        while let Some(character) = self.current() {
            if prev_character != '\\' && character == '"' {
                found_ending_quote = true;
                break;
            }
            self.advance();
            prev_character = character;
        }
        if found_ending_quote == false {
            return None;
        }
        let end = self.position;
        self.advance(); // remove ending '"'

        let s: String = (&self.input[start..end]).iter().collect();
        Some(s)
    }

    fn read_identifier(&mut self) -> Option<String> {
        let start = self.position;
        if let Some(character) = self.current() {
            if !character.is_alphabetic() {
                return None;
            }
            self.advance();
        }

        while let Some(character) = self.current() {
            if !character.is_alphanumeric() {
                break;
            }
            self.advance();
        }
        let end = self.position;
        let s: String = (&self.input[start..end]).iter().collect();
        Some(s)
    }
    fn identifier_to_token(identifier: &str) -> TokenKind {
        match identifier {
            "let" => TokenKind::Let,
            "mut" => TokenKind::Mut,
            "if" => TokenKind::If,
            "else" => TokenKind::Else,
            "elif" => TokenKind::Elif,
            "while" => TokenKind::While,
            "print" => TokenKind::Print,
            _ => TokenKind::Identifier(identifier.to_string()),
        }
    }

    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace();
        let mut tok = Token {
            kind: TokenKind::EOF,
            line_no: self.line_no,
            col_no: self.col_no,
            isfile: self.isfile,
            file_id: self.file_id,
        };

        let c = match self.current() {
            None => {
                self.reached_eof=true;
                return tok;
            }
            Some(character) => character,
        };
        let mut is_advance_required = true;
        match c {
            '+' => tok.kind = TokenKind::Plus,
            '-' => tok.kind = TokenKind::Minus,
            '*' => tok.kind = TokenKind::Star,
            '/' => tok.kind = TokenKind::Slash,
            '%' => tok.kind = TokenKind::Percent,
            '(' => tok.kind = TokenKind::LParen,
            ')' => tok.kind = TokenKind::RParen,
            '{' => tok.kind = TokenKind::LBrace,
            '}' => tok.kind = TokenKind::RBrace,
            '<' => tok.kind = TokenKind::Less,
            '>' => tok.kind = TokenKind::Greater,
            ';' => tok.kind = TokenKind::Semicolon,
            ',' => tok.kind = TokenKind::Comma,
            '=' => match self.peek() {
                Some(n) => {
                    if n == '=' {
                        tok.kind = TokenKind::EqualEqual;
                        self.advance(); // skip first =,the second will be skipped after the match()
                    } else {
                        tok.kind = TokenKind::Equal;
                    }
                }
                None => tok.kind = TokenKind::Equal,
            },
            '!' => match self.peek() {
                Some(n) => {
                    if n == '=' {
                        tok.kind = TokenKind::NotEqual;
                        self.advance(); // skip first =,the second will be skipped after the match()
                    } else {
                        tok.kind = TokenKind::Not;
                    }
                }
                None => tok.kind = TokenKind::Equal,
            },
            other => {
                is_advance_required = false;
                match other {
                    '"' => match self.read_string() {
                        Some(string) => tok.kind = TokenKind::String(string),
                        None => tok.kind = TokenKind::ILLEGAL,
                    },
                    '0'..='9' => match self.read_number() {
                        Some(num) => tok.kind = TokenKind::Integer(num),
                        None => tok.kind = TokenKind::ILLEGAL,
                    },
                    'A'..='Z' | 'a'..='z' => match self.read_identifier() {
                        Some(ident) => tok.kind = Lexer::identifier_to_token(&ident),
                        None => tok.kind = TokenKind::ILLEGAL,
                    },
                    _ => tok.kind = TokenKind::ILLEGAL,
                }
            }
        }
        if tok.kind == TokenKind::ILLEGAL {
            is_advance_required = false;
        }
        if is_advance_required == true {
            self.advance();
        }
        tok.line_no = self.line_no;
        tok.col_no = self.col_no;
        return tok;
    }
    pub fn peek_token(&mut self) -> Token {
        let position = self.position;
        let line_no = self.line_no;
        let col_no = self.col_no;
        let reached_eof = self.reached_eof;
        let tok = self.next_token();
        self.position = position;
        self.line_no = line_no;
        self.col_no = col_no;
        self.reached_eof = reached_eof;
        return tok;
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Token;
     fn next(&mut self) -> Option<Self::Item> {
        if self.reached_eof == true{
            return None;
        }
         Some(self.next_token()) // this can return EOF token one time and iterator stops
    }
}
