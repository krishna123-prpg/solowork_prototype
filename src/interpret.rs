use crate::compiler::{self, Compiler};
use crate::tokenize::{Token, TokenKind};
use std::collections::HashMap;
use std::process::exit;

// ---------------------------------------------------------------------------
// Value types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
enum Value {
    Integer(i64),
    String(String),
    Bool(bool),
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Integer(n) => write!(f, "{n}"),
            Value::String(s) => write!(f, "{s}"),
            Value::Bool(b) => write!(f, "{b}"),
        }
    }
}

// ---------------------------------------------------------------------------
// Variable storage
// ---------------------------------------------------------------------------

struct Variable {
    value: Value,
    is_mut: bool,
}

// ---------------------------------------------------------------------------
// Interpreter
// ---------------------------------------------------------------------------

pub struct Interpret<'a> {
    compiler: &'a Compiler,
    tokens: Vec<Token>,
    position: usize,
    variables: HashMap<String, Variable>,
}

impl<'a> Interpret<'a> {
    pub fn new(compiler: &'a Compiler, tokens: Vec<Token>) -> Self {
        Self {
            compiler,
            tokens,
            position: 0,
            variables: HashMap::new(),
        }
    }

    // -----------------------------------------------------------------------
    // Token navigation helpers
    // -----------------------------------------------------------------------

    fn current(&self) -> Option<&Token> {
        self.tokens.get(self.position)
    }

    fn current_kind(&self) -> Option<&TokenKind> {
        self.current().map(|t| &t.kind)
    }

    fn advance(&mut self) {
        self.position += 1;
    }

    /// Clone the current token (needed when we need ownership before advancing).
    fn current_cloned(&self) -> Option<Token> {
        self.tokens.get(self.position).cloned()
    }

    // -----------------------------------------------------------------------
    // Error helpers
    // -----------------------------------------------------------------------

    fn error_tok(&self, tok: &Token, msg: &str) -> ! {
        let filename = if tok.isfile {
            self.compiler.get_filename(tok.file_id).unwrap_or_else(|| String::from("[No File]"))
        } else {
            String::from("[No File]")
        };
        println!(); // flush stdout
        eprintln!("[error] {filename}:{}:{} -> {msg}", tok.line_no, tok.col_no);
        if tok.isfile {
            if let Some(line) = compiler::get_line(&filename, tok.line_no) {
                eprintln!("\t{line}");
            }
        }
        exit(1);
    }

    fn error_at_current(&self, msg: &str) -> ! {
        if let Some(tok) = self.current_cloned() {
            self.error_tok(&tok, msg);
        }
        eprintln!("[error] {msg}");
        exit(1);
    }

    // -----------------------------------------------------------------------
    // Expect helpers
    // -----------------------------------------------------------------------

    fn expect(&mut self, kind: TokenKind) {
        if let Some(tok) = self.current_cloned() {
            if tok.kind == kind {
                self.advance();
                return;
            }
            self.error_tok(
                &tok,
                &format!("expected \"{kind}\" but found \"{}\"", tok.kind),
            );
        }
        eprintln!("[error] expected \"{kind}\" but found nothing");
        exit(1);
    }

    fn expect_identifier(&mut self) -> String {
        if let Some(tok) = self.current_cloned() {
            if let TokenKind::Identifier(name) = &tok.kind {
                let name = name.clone();
                self.advance();
                return name;
            }
            self.error_tok(&tok, &format!("expected an identifier but found \"{}\"", tok.kind));
        }
        eprintln!("[error] expected an identifier but found nothing");
        exit(1);
    }

    // -----------------------------------------------------------------------
    // Expression evaluation
    //
    // Grammar (precedence, lowest → highest):
    //   expr        → comparison
    //   comparison  → addition ( ( "==" | "!=" | "<" | ">" ) addition )*
    //   addition    → multiplication ( ( "+" | "-" ) multiplication )*
    //   multiplication → unary ( ( "*" | "/" | "%" ) unary )*
    //   unary       → ( "!" | "-" ) unary | primary
    //   primary     → INTEGER | STRING | IDENTIFIER | "(" expr ")"
    // -----------------------------------------------------------------------

    fn parse_expression(&mut self) -> Value {
        self.parse_comparison()
    }

    fn parse_comparison(&mut self) -> Value {
        let mut left = self.parse_addition();

        loop {
            let op = match self.current_kind() {
                Some(TokenKind::EqualEqual) => TokenKind::EqualEqual,
                Some(TokenKind::NotEqual)   => TokenKind::NotEqual,
                Some(TokenKind::Less)       => TokenKind::Less,
                Some(TokenKind::Greater)    => TokenKind::Greater,
                _ => break,
            };
            self.advance();
            let right = self.parse_addition();
            left = self.apply_comparison(op, left, right);
        }
        left
    }

    fn apply_comparison(&self, op: TokenKind, left: Value, right: Value) -> Value {
        match (left, right) {
            (Value::Integer(l), Value::Integer(r)) => Value::Bool(match op {
                TokenKind::EqualEqual => l == r,
                TokenKind::NotEqual   => l != r,
                TokenKind::Less       => l < r,
                TokenKind::Greater    => l > r,
                _ => unreachable!(),
            }),
            (Value::String(l), Value::String(r)) => Value::Bool(match op {
                TokenKind::EqualEqual => l == r,
                TokenKind::NotEqual   => l != r,
                _ => self.error_at_current("only == and != are supported for string comparison"),
            }),
            (Value::Bool(l), Value::Bool(r)) => Value::Bool(match op {
                TokenKind::EqualEqual => l == r,
                TokenKind::NotEqual   => l != r,
                _ => self.error_at_current("only == and != are supported for bool comparison"),
            }),
            _ => self.error_at_current("type mismatch in comparison"),
        }
    }

    fn parse_addition(&mut self) -> Value {
        let mut left = self.parse_multiplication();

        loop {
            let op = match self.current_kind() {
                Some(TokenKind::Plus)  => TokenKind::Plus,
                Some(TokenKind::Minus) => TokenKind::Minus,
                _ => break,
            };
            self.advance();
            let right = self.parse_multiplication();
            left = match (left, right) {
                (Value::Integer(l), Value::Integer(r)) => match op {
                    TokenKind::Plus  => Value::Integer(l + r),
                    TokenKind::Minus => Value::Integer(l - r),
                    _ => unreachable!(),
                },
                (Value::String(l), Value::String(r)) => match op {
                    TokenKind::Plus  => Value::String(l + &r),
                    _ => self.error_at_current("operator not supported for strings (use + to concat)"),
                },
                _ => self.error_at_current("type mismatch in addition/subtraction"),
            };
        }
        left
    }

    fn parse_multiplication(&mut self) -> Value {
        let mut left = self.parse_unary();

        loop {
            let op = match self.current_kind() {
                Some(TokenKind::Star)    => TokenKind::Star,
                Some(TokenKind::Slash)   => TokenKind::Slash,
                Some(TokenKind::Percent) => TokenKind::Percent,
                _ => break,
            };
            self.advance();
            let right = self.parse_unary();
            left = match (left, right) {
                (Value::Integer(l), Value::Integer(r)) => match op {
                    TokenKind::Star    => Value::Integer(l * r),
                    TokenKind::Slash   => {
                        if r == 0 { self.error_at_current("division by zero"); }
                        Value::Integer(l / r)
                    }
                    TokenKind::Percent => {
                        if r == 0 { self.error_at_current("modulo by zero"); }
                        Value::Integer(l % r)
                    }
                    _ => unreachable!(),
                },
                _ => self.error_at_current("arithmetic requires integer operands"),
            };
        }
        left
    }

    fn parse_unary(&mut self) -> Value {
        if let Some(tok) = self.current_cloned() {
            match &tok.kind {
                TokenKind::Minus => {
                    self.advance();
                    match self.parse_unary() {
                        Value::Integer(n) => return Value::Integer(-n),
                        _ => self.error_tok(&tok, "unary '-' requires an integer operand"),
                    }
                }
                TokenKind::Not => {
                    self.advance();
                    match self.parse_unary() {
                        Value::Bool(b) => return Value::Bool(!b),
                        _ => self.error_tok(&tok, "unary '!' requires a boolean operand"),
                    }
                }
                _ => {}
            }
        }
        self.parse_primary()
    }

    fn parse_primary(&mut self) -> Value {
        let tok = match self.current_cloned() {
            Some(t) => t,
            None    => self.error_at_current("unexpected end of input in expression"),
        };

        match tok.kind.clone() {
            TokenKind::Integer(n) => {
                self.advance();
                Value::Integer(n)
            }
            TokenKind::String(s) => {
                self.advance();
                Value::String(s)
            }
            TokenKind::Identifier(name) => {
                self.advance();
                match self.variables.get(&name) {
                    Some(var) => var.value.clone(),
                    None => self.error_tok(&tok, &format!("undefined variable '{name}'")),
                }
            }
            TokenKind::LParen => {
                self.advance(); // consume '('
                let val = self.parse_expression();
                self.expect(TokenKind::RParen);
                val
            }
            other => self.error_tok(&tok, &format!("unexpected token '{other}' in expression")),
        }
    }

    // -----------------------------------------------------------------------
    // Statements
    // -----------------------------------------------------------------------

    /// `let [mut] name = expr ;`
    fn interpret_let_statement(&mut self) {
        self.expect(TokenKind::Let);

        let is_mut = matches!(self.current_kind(), Some(TokenKind::Mut));
        if is_mut {
            self.advance();
        }

        let name = self.expect_identifier();

        self.expect(TokenKind::Equal);

        let value = self.parse_expression();

        self.expect(TokenKind::Semicolon);

        self.variables.insert(name, Variable { value, is_mut });
    }

    /// `name = expr ;`  (assignment to an existing mutable variable)
    fn interpret_assign_statement(&mut self, name: String) {
        // We already consumed the identifier; caller passes it to us.
        let tok = self.current_cloned().unwrap(); // points at '='
        self.expect(TokenKind::Equal);

        let value = self.parse_expression();

        self.expect(TokenKind::Semicolon);

        match self.variables.get_mut(&name) {
            Some(var) if var.is_mut => var.value = value,
            Some(_) => self.error_tok(&tok, &format!("cannot assign to immutable variable '{name}'")),
            None    => self.error_tok(&tok, &format!("undefined variable '{name}'")),
        }
    }

    /// `print expr [, expr]* ;`
    fn interpret_print_statement(&mut self) {
        self.expect(TokenKind::Print);

        let mut first = true;
        loop {
            match self.current_kind() {
                Some(TokenKind::Semicolon) | None => break,
                _ => {}
            }

            if !first {
                // Comma between values is optional but accepted
                if matches!(self.current_kind(), Some(TokenKind::Comma)) {
                    self.advance();
                }
            }
            first = false;

            let val = self.parse_expression();
            print!("{val}");
        }
        println!(); // newline at end of print statement
        self.expect(TokenKind::Semicolon);
    }

    /// `if ( expr ) { block } [elif ( expr ) { block }]* [else { block }]`
    fn interpret_if_statement(&mut self) {
        self.expect(TokenKind::If);
        self.expect(TokenKind::LParen);
        let condition = self.parse_expression();
        self.expect(TokenKind::RParen);

        let taken = match &condition {
            Value::Bool(b) => *b,
            Value::Integer(n) => *n != 0,
            _ => self.error_at_current("if condition must evaluate to a bool or integer"),
        };

        if taken {
            self.interpret_block();
            self.skip_elif_else_chain();
        } else {
            self.skip_block();
            self.interpret_elif_else_chain();
        }
    }

    /// After a taken if/elif branch: skip remaining elif/else blocks.
    fn skip_elif_else_chain(&mut self) {
        loop {
            match self.current_kind() {
                Some(TokenKind::Elif) => {
                    self.advance(); // elif
                    self.expect(TokenKind::LParen);
                    // skip condition tokens until matching ')'
                    let mut depth = 1usize;
                    loop {
                        match self.current_kind() {
                            None => self.error_at_current("unterminated elif condition"),
                            Some(TokenKind::LParen) => { depth += 1; self.advance(); }
                            Some(TokenKind::RParen) => {
                                depth -= 1;
                                self.advance();
                                if depth == 0 { break; }
                            }
                            _ => { self.advance(); }
                        }
                    }
                    self.skip_block();
                }
                Some(TokenKind::Else) => {
                    self.advance();
                    self.skip_block();
                    break;
                }
                _ => break,
            }
        }
    }

    /// When the if condition was false: try elif/else.
    fn interpret_elif_else_chain(&mut self) {
        loop {
            match self.current_kind() {
                Some(TokenKind::Elif) => {
                    self.advance();
                    self.expect(TokenKind::LParen);
                    let condition = self.parse_expression();
                    self.expect(TokenKind::RParen);
                    let taken = match &condition {
                        Value::Bool(b) => *b,
                        Value::Integer(n) => *n != 0,
                        _ => self.error_at_current("elif condition must evaluate to a bool or integer"),
                    };
                    if taken {
                        self.interpret_block();
                        self.skip_elif_else_chain();
                        return;
                    } else {
                        self.skip_block();
                        // continue loop to look for next elif/else
                    }
                }
                Some(TokenKind::Else) => {
                    self.advance();
                    self.interpret_block();
                    break;
                }
                _ => break,
            }
        }
    }

    /// `while ( expr ) { block }`
    fn interpret_while_statement(&mut self) {
        let condition_start = self.position;
        self.expect(TokenKind::While);
        self.expect(TokenKind::LParen);
        let condition_body_start = self.position; // first token of condition

        loop {
            // Evaluate condition
            self.position = condition_body_start;
            let condition = self.parse_expression();
            self.expect(TokenKind::RParen);

            let keep_going = match &condition {
                Value::Bool(b) => *b,
                Value::Integer(n) => *n != 0,
                _ => self.error_at_current("while condition must evaluate to a bool or integer"),
            };

            if keep_going {
                self.interpret_block();
                // Loop: go back to re-evaluate condition
                let _ = condition_start; // silence lint
            } else {
                self.skip_block();
                break;
            }
        }
    }

    // -----------------------------------------------------------------------
    // Block execution
    // -----------------------------------------------------------------------

    /// Execute tokens inside `{ ... }`.
    fn interpret_block(&mut self) {
        self.expect(TokenKind::LBrace);
        loop {
            match self.current_kind() {
                Some(TokenKind::RBrace) | None => break,
                _ => self.interpret_statement(),
            }
        }
        self.expect(TokenKind::RBrace);
    }

    /// Skip over a `{ ... }` block without executing it.
    fn skip_block(&mut self) {
        self.expect(TokenKind::LBrace);
        let mut depth = 1usize;
        loop {
            match self.current_kind() {
                None => self.error_at_current("unterminated block"),
                Some(TokenKind::LBrace) => { depth += 1; self.advance(); }
                Some(TokenKind::RBrace) => {
                    depth -= 1;
                    self.advance();
                    if depth == 0 { break; }
                }
                _ => { self.advance(); }
            }
        }
    }

    // -----------------------------------------------------------------------
    // Statement dispatcher
    // -----------------------------------------------------------------------

    fn interpret_statement(&mut self) {
        match self.current_kind() {
            Some(TokenKind::Let)   => self.interpret_let_statement(),
            Some(TokenKind::Print) => self.interpret_print_statement(),
            Some(TokenKind::If)    => self.interpret_if_statement(),
            Some(TokenKind::While) => self.interpret_while_statement(),
            Some(TokenKind::Identifier(_)) => {
                // Could be assignment: `name = expr ;`
                let tok = self.current_cloned().unwrap();
                if let TokenKind::Identifier(name) = tok.kind.clone() {
                    self.advance(); // consume identifier
                    match self.current_kind() {
                        Some(TokenKind::Equal) => self.interpret_assign_statement(name),
                        _ => self.error_tok(&tok, "expected '=' after identifier for assignment"),
                    }
                }
            }
            Some(TokenKind::EOF) | None => {}
            Some(_) => {
                let tok = self.current_cloned().unwrap();
                self.error_tok(&tok, &format!("unexpected token '{}' at statement level", tok.kind));
            }
        }
    }

    // -----------------------------------------------------------------------
    // Entry point
    // -----------------------------------------------------------------------

    pub fn run(&mut self) {
        loop {
            match self.current_kind() {
                Some(TokenKind::EOF) | None => break,
                _ => self.interpret_statement(),
            }
        }
    }
}
