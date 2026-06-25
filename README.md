# solowork
A lightweight, interpreted scripting language with syntax inspired by Rust and C. Written in Rust.

## new people
Those who are using windows or new(or unaware) to linux or this kind of reading, you don't need to look down. Just search AI "how to download .zip of a github repo" and do that same thing for this project. Extract the .zip file.Just go to **examples** folder present inside it. There you will find many files. Open *docs.html* to learn my language and later watch *solowork.mp4* video which demonstrate how to use it. There are some example programs. Run it as shown in the video. Edit it if you want. **You don't need to read everything present in docs.html as it may be pointless to you. Such pointless and difficult things are only for advance users.** That's all. **I strongly recommed that you learn a language like python before reading docs.html as it may make this docs easier to read and understand.** Don't scroll down more as you may not understand what is written down.

```
let a = 5;
let b = 6;
let mut i = 0;

while(i == 0) {
    let c = a + b;
    print "a + b = " c;
    i = 1;
}
```

## Features

- **Immutable-by-default variables** — `let` bindings are immutable unless marked `mut`
- **Three value types** — `integer` (i64), `string`, and `bool`
- **Full arithmetic** — `+`, `-`, `*`, `/`, `%` with correct precedence and parenthesization
- **Comparison & logical operators** — `==`, `!=`, `<`, `>`, `!`
- **Control flow** — `if` / `elif` / `else` chains and `while` loops
- **String concatenation** — `+` works on strings too
- **Helpful error messages** — `file:line:col` with the offending source line printed

## Getting Started

### Prerequisites

- [Rust](https://rustup.rs/) 1.70 or later

### Build

```sh
git clone https://github.com/krishna123-prpg/solowork
cd solowork
cargo build --release
```

The binary is at `target/release/solowork`.

### Run a file

```sh
./target/release/solowork hello.rl
# or during development:
cargo run -- hello.rl
```

### Help

```sh
solowork --help
```

## Language Reference

Refer 'docs/docs.html' if you want a more detailed reference

### Variables

```rust
let x = 10;           // immutable
let mut count = 0;    // mutable — can be reassigned
count = count + 1;    // reassignment (only valid with mut)
```

Declaring the same name twice replaces the previous binding. Assigning to an immutable variable is a runtime error.

### Types

| Type      | Description              | Examples              |
|-----------|--------------------------|-----------------------|
| `integer` | 64-bit signed integer    | `0`, `42`, `-7`       |
| `string`  | UTF-8 text, double-quoted | `"hello"`, `"world"` |
| `bool`    | Result of a comparison   | produced by `==`, `<` |

Types are inferred — there is no type annotation syntax.

### Expressions & Operators

Operator precedence (highest to lowest):

| Level      | Operators              |
|------------|------------------------|
| primary    | literals, identifiers, `(expr)` |
| unary      | `-x`, `!b`             |
| multiply   | `*`, `/`, `%`          |
| add        | `+`, `-`               |
| comparison | `==`, `!=`, `<`, `>`   |

```rust
let a = 2 + 3 * 4;       // 14
let b = (2 + 3) * 4;     // 20
let ok = a == 14;         // true
let s = "hello" + " world"; // "hello world"
```

Division and modulo truncate toward zero. Division or modulo by zero is a runtime error.

### print

Evaluates one or more expressions and prints them to stdout on a single line, followed by a newline. Commas between values are optional.

```rust
print "hello world";
print "n = " n;
print 1 + 2;
print "a=" 1, " b=" 2;   // commas optional
```

### if / elif / else

```rust
if(score > 89) {
    print "A";
} elif(score > 74) {
    print "B";
} elif(score > 59) {
    print "C";
} else {
    print "F";
}
```

Conditions must be `bool` or `integer` (0 is false, anything else is true). Parentheses are required.

### while

```rust
let mut i = 5;
while(i > 0) {
    print i;
    i = i - 1;
}
// prints: 5  4  3  2  1
```

The condition is re-evaluated before every iteration. There is no `break` yet — exit the loop by making the condition false.

### Error format

```
[error] file.rl:4:12 -> undefined variable 'x'
        let y = x + 1;
```

The interpreter exits with code 1 on the first error.

## Project Structure

```
src/
├── main.rs          # CLI entry point — argument parsing, file loading
├── compiler.rs      # File registry and source-line lookup for errors
├── tokenize.rs      # Lexer — converts source text to a token stream
└── interpret.rs     # Tree-walking interpreter — evaluates the token stream directly
```

The interpreter works in two passes: the **lexer** converts the entire source file to a flat `Vec<Token>`, then the **interpreter** walks that vector and evaluates statements directly — no AST is built.

