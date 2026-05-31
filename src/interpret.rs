use crate::compiler::{self, Compiler};
use crate::tokenize::{Token, TokenKind};
use std::collections::HashMap;
use std::io::{self, Write};
use std::process::exit;
enum VariableKind {
    Integer(i64),
    String(String),
}

struct Variable {
    kind: VariableKind,
    is_mut: bool,
}

pub struct Interpret<'a> {
    compiler: &'a Compiler,
    tokens: Vec<Token>,
    position: usize, // index of tokens member; starts at 0
    variables: HashMap<String, Variable>,
}

trait VariableTrait {
    fn is_variable(&self, name: &str) -> bool;
    fn add_or_overwrite(&mut self, name: &str, var: Variable);
    fn delete(&mut self, name: &str);
}
impl VariableTrait for HashMap<String, Variable> {
    fn is_variable(&self, name: &str) -> bool {
        if let None = self.get(name) {
            return false;
        }
        return true;
    }
    fn add_or_overwrite(&mut self, name: &str, var: Variable) {
        self.insert(name.to_string(), var);
    }
    fn delete(&mut self, name: &str) {
        self.remove(name);
    }
}
impl<'a> Interpret<'a> {
    pub fn new(compiler: &'a Compiler, tokens: Vec<Token>) -> Interpret {
        Interpret {
            compiler: &compiler,
            tokens: tokens,
            position: 0,
            variables: HashMap::new(),
        }
    }

    fn current(&self) -> Option<Token> {
        self.tokens.get(self.position).clone().cloned()
    }

    fn advance(&mut self) {
        self.position += 1;
    }
    fn error_tok(&self, tok: &Token, msg: &str) {
        let filename = if tok.isfile == true {
            self.compiler.get_filename(tok.file_id).unwrap()
        } else {
            String::from("[No File]")
        };
        eprintln!("[error] {filename}:{}:{} -> {msg}", tok.line_no, tok.col_no);
        eprintln!("\t{}", compiler::get_line(&filename, tok.line_no).unwrap());
        exit(1);
    }
    fn expect(&mut self, tokenkind: TokenKind){ 
        if let Some(tok) = self.current() {
            if tok.kind == tokenkind {
                self.advance();
                return;
            }
            self.error_tok(
                &tok,
                format!("expected \"{tokenkind}\" but found \"{}\"", tok.kind).as_str(),
            );
        }
        panic!("expected \"{tokenkind}\" but found nothing");
    }
    // fn identifier_token_to_string(tok:
    fn interpret_print_statement(&mut self) {
        self.expect(TokenKind::Print);
        loop {
            if let Some(tok) = self.current() {
                match &tok.kind {
                    TokenKind::EOF => {
                        self.error_tok(&tok, "Semicolon must be given to end print statement");
                    }
                    TokenKind::Semicolon => break,
                    TokenKind::Integer(num) => print!("{num}"),
                    TokenKind::String(s) => print!("{s}"),
                    TokenKind::Identifier(s) => match self.variables.is_variable(s) {
                        true => {
                            let variable = self.variables.get(s).unwrap();
                            match &variable.kind {
                                VariableKind::Integer(num) => print!("{num}"),
                                VariableKind::String(s) => print!("{s}"),
                            }
                        }
                        false => {
                            self.error_tok(&tok, "There is no such variable as {s}");
                        }
                    },
                    _ => self.error_tok(&tok, "An invalid token is given to print statement"),
                }
                println!("");
                self.advance();
                
            } else {
                break;
            }
        }
    }
    pub fn run(&mut self) {
        if let Some(tok) = self.current() {
            match &tok.kind {
                TokenKind::Print => self.interpret_print_statement(),
                _ => {}
            }
        }
    }
}
