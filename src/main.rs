pub mod compiler;
pub mod interpret;
pub mod tokenize;
use crate::compiler::Compiler;
use crate::interpret::Interpret;
use crate::tokenize::Lexer;
fn main() {
    let _source = r#"
    let a=5;
    let b=6;
    let mut i = 0;

    while(i==0){
    let c=a+b;
    print c;
    }
    "#;
    let source = r#"
    print "hello world";
    "#;
    let mut compiler = Compiler::new();
    let mut lexer = Lexer::new(&compiler, source);
    /*
     let mut tok = lexer.next_token();
    while tok.kind != TokenKind::EOF {
        println!("{0:#?}", tok.kind);
        tok = lexer.next_token();
    }
    println!("{0:#?}", tok.kind);
    */
    let mut interpreter = Interpret::new(&compiler, lexer.collect());
    interpreter.run();
}
