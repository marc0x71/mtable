# mtable

A fast and efficient Rust library for pattern matching and lexical analysis based on trie data structures. Build high-performance lexers and tokenizers with ease.

## Features

- 🚀 **Fast**: Built on an optimized trie structure with O(n) lookup time
- 🔍 **Full Tokenization**: Built-in lexer with longest-match (maximal munch) strategy
- 🎯 **Pattern Matching**: Supports literal patterns, character classes `[abc]`, and repetitions `+`
- 🔤 **Customizable Alphabet**: Define your own set of valid characters
- 🎨 **Generic**: Works with any type `T: Clone + Debug` (enums, integers, structs)
- ✅ **Type-safe**: Robust error handling with `Result`
- 📦 **Zero Dependencies**: Only Rust standard library

## Quick Start

### Pattern Matching

```rust
use mtable::Table;

#[derive(Debug, Clone, PartialEq)]
enum TokenType {
    Keyword,
    Identifier,
}

let mut table = Table::new("abcdefghijklmnopqrstuvwxyz".to_string());

table.add("if", TokenType::Keyword).unwrap();
table.add("while", TokenType::Keyword).unwrap();

// Single pattern matching
assert_eq!(table.get("if").unwrap(), Some(&TokenType::Keyword));
assert_eq!(table.get("other").unwrap(), None);
```

### Tokenization (Lexer)

```rust
use mtable::Table;

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Number,
    Plus,
    Multiply,
}

let mut lexer = Table::new("0123456789+*".to_string());

lexer.add("[0123456789]+", Token::Number).unwrap();
lexer.add("+", Token::Plus).unwrap();
lexer.add("*", Token::Multiply).unwrap();

// Tokenize entire expression
let tokens: Vec<_> = lexer.lexer("123+456*789")
    .unwrap()
    .collect::<Result<_, _>>()
    .unwrap();

assert_eq!(tokens.len(), 5);
assert_eq!(tokens[0], (&Token::Number, "123"));
assert_eq!(tokens[1], (&Token::Plus, "+"));
assert_eq!(tokens[2], (&Token::Number, "456"));
assert_eq!(tokens[3], (&Token::Multiply, "*"));
assert_eq!(tokens[4], (&Token::Number, "789"));
```

## Pattern Syntax

### Literal Patterns

Match exact character sequences:

```rust
#[derive(Debug, Clone, PartialEq)]
enum Operator {
    Plus,
    Minus,
    Multiply,
}

let mut table = Table::new("+-*".to_string());
table.add("+", Operator::Plus).unwrap();
table.add("-", Operator::Minus).unwrap();
table.add("*", Operator::Multiply).unwrap();

assert_eq!(table.get("+").unwrap(), Some(&Operator::Plus));
```

### Character Classes `[...]`

Match any one of the specified characters:

```rust
let mut table = Table::new("abcdefghijklmnopqrstuvwxyz".to_string());

// Match "cat", "cot", or "cut"
table.add("c[aou]t", 1).unwrap();

assert_eq!(table.get("cat").unwrap(), Some(&1));
assert_eq!(table.get("cot").unwrap(), Some(&1));
assert_eq!(table.get("cut").unwrap(), Some(&1));
assert_eq!(table.get("cit").unwrap(), None);
```

### Repetition Operator `+`

Match one or more occurrences of the preceding character or class:

```rust
#[derive(Debug, Clone, PartialEq)]
enum TokenType {
    Number,
}

let mut table = Table::new("0123456789".to_string());

// Match one or more digits
table.add("[0123456789]+", TokenType::Number).unwrap();

assert_eq!(table.get("0").unwrap(), Some(&TokenType::Number));
assert_eq!(table.get("42").unwrap(), Some(&TokenType::Number));
assert_eq!(table.get("123456").unwrap(), Some(&TokenType::Number));
```

## Lexer / Tokenizer

The `lexer()` method creates an iterator that tokenizes an entire input string using the **longest match** (maximal munch) strategy.

### Basic Usage

```rust
#[derive(Debug, Clone, PartialEq)]
enum Token {
    Num,
    Add,
    Sub,
}

let mut table = Table::new("0123456789+-".to_string());
table.add("[0123456789]+", Token::Num).unwrap();
table.add("+", Token::Add).unwrap();
table.add("-", Token::Sub).unwrap();

// Create lexer iterator
for result in table.lexer("12+34-56").unwrap() {
    let (token, content) = result.unwrap();
    println!("{:?}: {}", token, content);
}

// Or collect all tokens at once
let tokens: Vec<_> = table.lexer("12+34")
    .unwrap()
    .collect::<Result<_, _>>()
    .unwrap();
```

### Longest Match (Maximal Munch)

The lexer always chooses the longest possible match:

```rust
#[derive(Debug, Clone, PartialEq)]
enum Token {
    Eq,      // =
    EqEq,    // ==
}

let mut table = Table::new("=".to_string());
table.add("=", Token::Eq).unwrap();
table.add("==", Token::EqEq).unwrap();

let tokens: Vec<_> = table.lexer("===")
    .unwrap()
    .collect::<Result<_, _>>()
    .unwrap();

// Results in [EqEq("=="), Eq("=")]
// Not [Eq("="), Eq("="), Eq("=")]
assert_eq!(tokens[0], (&Token::EqEq, "=="));
assert_eq!(tokens[1], (&Token::Eq, "="));
```

### Error Handling in Lexer

The lexer can return two types of errors:

```rust
use mtable::LexerError;

#[derive(Debug, Clone, PartialEq)]
enum Token { Number }

let mut table = Table::new("0123456789".to_string());
table.add("[0123456789]+", Token::Number).unwrap();

// Unknown character
match table.lexer("12@34").unwrap().next() {
    Some(Err(LexerError::UnknownChar { char, position })) => {
        println!("Unknown char '{}' at position {}", char, position);
    }
    _ => {}
}

// No pattern matches
match table.lexer("abc").unwrap().next() {
    Some(Err(LexerError::UnexpectedEnd { position })) => {
        println!("No match at position {}", position);
    }
    _ => {}
}
```

## Real-World Examples

### Complete Programming Language Lexer

```rust
#[derive(Debug, Clone, PartialEq)]
enum TokenType {
    // Keywords
    If,
    Else,
    While,
    Return,
    
    // Identifiers and literals
    Identifier,
    Number,
    
    // Operators
    Plus,
    Minus,
    Assign,
    LParen,
    RParen,
}

let mut lexer = Table::new(
    "abcdefghijklmnopqrstuvwxyz0123456789+-=()".to_string()
);

// Keywords (must be added before identifier pattern)
lexer.add("if", TokenType::If).unwrap();
lexer.add("else", TokenType::Else).unwrap();
lexer.add("while", TokenType::While).unwrap();
lexer.add("return", TokenType::Return).unwrap();

// Identifiers (letter followed by letters/digits)
lexer.add(
    "[abcdefghijklmnopqrstuvwxyz][abcdefghijklmnopqrstuvwxyz0123456789]+",
    TokenType::Identifier
).unwrap();

// Numbers
lexer.add("[0123456789]+", TokenType::Number).unwrap();

// Operators
lexer.add("+", TokenType::Plus).unwrap();
lexer.add("-", TokenType::Minus).unwrap();
lexer.add("=", TokenType::Assign).unwrap();
lexer.add("(", TokenType::LParen).unwrap();
lexer.add(")", TokenType::RParen).unwrap();

// Tokenize source code
let source = "if(x=42)return(x+1)";
let tokens: Vec<_> = lexer.lexer(source)
    .unwrap()
    .map(|r| r.unwrap().0.clone())
    .collect();

assert_eq!(tokens, vec![
    TokenType::If,
    TokenType::LParen,
    TokenType::Identifier,
    TokenType::Assign,
    TokenType::Number,
    TokenType::RParen,
    TokenType::Return,
    TokenType::LParen,
    TokenType::Identifier,
    TokenType::Plus,
    TokenType::Number,
    TokenType::RParen,
]);
```

### HTTP Method Router with Numeric IDs

```rust
// Method IDs for fast matching
const GET: u32 = 1;
const POST: u32 = 2;
const PUT: u32 = 3;
const DELETE: u32 = 4;
const PATCH: u32 = 5;

let mut router = Table::new("abcdefghilmnoprstuv".to_string());

router.add("get", GET).unwrap();
router.add("post", POST).unwrap();
router.add("put", PUT).unwrap();
router.add("delete", DELETE).unwrap();
router.add("patch", PATCH).unwrap();

match router.get("post").unwrap() {
    Some(&POST) => println!("Handle POST request"),
    Some(&GET) => println!("Handle GET request"),
    _ => println!("Unknown method"),
}
```

### Arithmetic Expression Lexer

```rust
#[derive(Debug, Clone, PartialEq)]
enum Token {
    Number,
    Plus,
    Minus,
    Multiply,
    Divide,
    LParen,
    RParen,
}

let mut lexer = Table::new("0123456789+-*/()".to_string());

lexer.add("[0123456789]+", Token::Number).unwrap();
lexer.add("+", Token::Plus).unwrap();
lexer.add("-", Token::Minus).unwrap();
lexer.add("*", Token::Multiply).unwrap();
lexer.add("/", Token::Divide).unwrap();
lexer.add("(", Token::LParen).unwrap();
lexer.add(")", Token::RParen).unwrap();

// Parse complex expression
let expr = "(100+50)*2-10/5";
let tokens: Vec<_> = lexer.lexer(expr)
    .unwrap()
    .collect::<Result<_, _>>()
    .unwrap();

// Can be used to build an AST or evaluate directly
for (token, text) in &tokens {
    match token {
        Token::Number => println!("NUM: {}", text),
        Token::Plus => println!("OP: +"),
        Token::Multiply => println!("OP: *"),
        _ => println!("TOKEN: {:?}", token),
    }
}
```

### Command Parser with Detailed Token Info

```rust
#[derive(Debug, Clone, PartialEq)]
struct Token {
    kind: TokenKind,
    precedence: u8,
}

#[derive(Debug, Clone, PartialEq)]
enum TokenKind {
    Add,
    Subtract,
    Multiply,
    Divide,
}

let mut parser = Table::new("+-*/".to_string());

parser.add("+", Token { kind: TokenKind::Add, precedence: 1 }).unwrap();
parser.add("-", Token { kind: TokenKind::Subtract, precedence: 1 }).unwrap();
parser.add("*", Token { kind: TokenKind::Multiply, precedence: 2 }).unwrap();
parser.add("/", Token { kind: TokenKind::Divide, precedence: 2 }).unwrap();

let token = parser.get("*").unwrap().unwrap();
assert_eq!(token.kind, TokenKind::Multiply);
assert_eq!(token.precedence, 2);
```

### Protocol Message Parser

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
enum MessageType {
    Connect = 0x01,
    Disconnect = 0x02,
    Ping = 0x03,
    Pong = 0x04,
    Data = 0x05,
}

let mut protocol = Table::new("abcdefghijklmnopqrstuvwxyz".to_string());

protocol.add("connect", MessageType::Connect).unwrap();
protocol.add("disconnect", MessageType::Disconnect).unwrap();
protocol.add("ping", MessageType::Ping).unwrap();
protocol.add("pong", MessageType::Pong).unwrap();
protocol.add("data", MessageType::Data).unwrap();

if let Some(&msg_type) = protocol.get("ping").unwrap() {
    println!("Message code: 0x{:02x}", msg_type as u8);
}
```

### Configuration File Parser

```rust
#[derive(Debug, Clone, PartialEq)]
enum ConfigKey {
    ServerPort,
    ServerHost,
    DatabaseUrl,
    CacheEnabled,
    LogLevel,
}

let mut config_parser = Table::new(
    "abcdefghijklmnopqrstuvwxyz_".to_string()
);

config_parser.add("server_port", ConfigKey::ServerPort).unwrap();
config_parser.add("server_host", ConfigKey::ServerHost).unwrap();
config_parser.add("database_url", ConfigKey::DatabaseUrl).unwrap();
config_parser.add("cache_enabled", ConfigKey::CacheEnabled).unwrap();
config_parser.add("log_level", ConfigKey::LogLevel).unwrap();

match config_parser.get("server_port").unwrap() {
    Some(&ConfigKey::ServerPort) => println!("Found server port config"),
    _ => println!("Unknown config key"),
}
```

## Error Handling

The library provides detailed error handling:

```rust
use mtable::TableError;

#[derive(Debug, Clone)]
enum Token { Identifier }

let mut table = Table::new("abc".to_string());

// Character not in alphabet
match table.add("hello!", Token::Identifier) {
    Err(TableError::InvalidInput('!')) => println!("Invalid character!"),
    _ => {}
}

// Non-ASCII string
match table.add("héllo", Token::Identifier) {
    Err(TableError::InvalidString(_)) => println!("Only ASCII supported"),
    _ => {}
}

// Duplicate value
table.add("abc", Token::Identifier).unwrap();
match table.add("abc", Token::Identifier) {
    Err(TableError::ValueAlreadyDefined { current, requested }) => {
        println!("Pattern already has a value assigned");
    },
    _ => {}
}

// Invalid character class
match table.add("[abc", Token::Identifier) {
    Err(TableError::InvalidRange) => println!("Unclosed bracket"),
    _ => {}
}
```

## API Reference

### `Table::new(alphabet: String) -> Self`

Creates a new table with the specified alphabet. The alphabet defines which characters are valid in patterns and queries.

```rust
// Lowercase letters only
let table = Table::<TokenType>::new("abcdefghijklmnopqrstuvwxyz".to_string());

// Digits only
let table = Table::<i32>::new("0123456789".to_string());

// Letters, digits, and special chars
let table = Table::<u32>::new("abcdefghijklmnopqrstuvwxyz0123456789_-".to_string());
```

### `Table::add(&mut self, pattern: &str, value: T) -> Result<(), TableError<T>>`

Adds a pattern to the table with an associated value. Returns an error if:
- The pattern contains characters not in the alphabet
- The pattern has invalid syntax (unclosed brackets, empty classes)
- A value is already defined for this pattern

```rust
table.add("keyword", TokenType::Keyword).unwrap();
table.add("[0123456789]+", TokenType::Number).unwrap();
```

### `Table::get(&self, s: &str) -> Result<Option<&T>, TableError<T>>`

Retrieves the value associated with a string, if it matches a pattern. Returns:
- `Ok(Some(&value))` if a match is found
- `Ok(None)` if no match is found
- `Err(...)` if the string contains invalid characters

```rust
match table.get("if").unwrap() {
    Some(&TokenType::Keyword) => println!("Found keyword"),
    None => println!("Not found"),
}
```

### `Table::lexer<'a>(&'a self, s: &'a str) -> Result<TableIterator<'a, T>, LexerError>`

Creates an iterator that tokenizes the entire input string using longest-match strategy. Returns:
- `Ok(iterator)` that yields `Result<(&T, &str), LexerError>` for each token
- `Err(LexerError::InvalidString)` if the input contains non-ASCII characters

```rust
// Iterate through tokens
for result in table.lexer("input string").unwrap() {
    match result {
        Ok((token, text)) => println!("{:?}: {}", token, text),
        Err(e) => eprintln!("Lexer error: {}", e),
    }
}

// Collect all tokens
let tokens: Vec<_> = table.lexer("input")
    .unwrap()
    .collect::<Result<_, _>>()
    .unwrap();
```

**Lexer Errors:**
- `LexerError::UnknownChar { char, position }` - Character not in alphabet
- `LexerError::UnexpectedEnd { position }` - No pattern matches at this position
- `LexerError::InvalidString(String)` - Input contains non-ASCII characters
```

## Performance

The implementation uses a trie (prefix tree) data structure which provides:

- **Pattern insertion**: O(m) where m is the pattern length
- **Single match lookup**: O(n) where n is the query string length
- **Lexer tokenization**: O(n) where n is the input string length
- **Memory**: Efficient prefix sharing between patterns, O(k) where k is total pattern size

The lexer uses **longest-match (maximal munch)** strategy with backtracking:
- Explores paths greedily, storing potential matches
- Backtracks to last valid match when no further transitions exist
- Single-pass tokenization with minimal overhead

## Limitations

- **ASCII Only**: Patterns and queries must contain only ASCII characters
- **Limited Operators**: Supports only `[...]` and `+`, not `*`, `?`, or other regex features
- **No Negation**: Character classes don't support negation (e.g., `[^abc]`)
- **Fixed Alphabet**: The alphabet must be defined at creation and cannot be modified
- **No Ranges**: Character classes must list characters explicitly (no `[a-z]` syntax)

## Roadmap

Features planned for future releases:

- 💭 **Optional Operator `?`**: Match zero or one occurrence
- 💭 **Kleene Star `*`**: Match zero or more occurrences  
- 💭 **Range Syntax**: Support `[a-z]` and `[0-9]` notation in character classes
- 💭 **Negation in Classes**: Support `[^abc]` to match anything except specified characters
- 💭 **Unicode Support**: Extend beyond ASCII to support UTF-8 strings

## Use Cases

`mtable` is ideal for:

- ✅ **Lexical analysis**: Full tokenization of source code with longest-match strategy
- ✅ **Expression parsers**: Tokenize mathematical or logical expressions
- ✅ **Command parsing**: Parse CLI commands and arguments
- ✅ **Configuration parsing**: Tokenize configuration files
- ✅ **Protocol parsers**: Tokenize protocol messages and commands
- ✅ **Data format parsers**: CSV, TSV, and simple structured formats
- ✅ **Simple compilers**: Frontend lexical analysis for DSLs

**Strengths:**
- Fast single-pass tokenization
- Predictable longest-match behavior
- Type-safe token representation
- Zero-copy string references in tokens
- Detailed error reporting with positions

**Not suitable for:**
- ❌ Complex regular expressions (use [regex](https://crates.io/crates/regex) crate)
- ❌ Unicode text processing (ASCII only)
- ❌ Context-sensitive parsing (use a proper parser)
- ❌ Patterns requiring lookahead/lookbehind

## Contributing

Contributions are welcome! Please feel free to submit issues or pull requests.

### Development

```bash
# Run tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Check formatting
cargo fmt --check

# Run clippy
cargo clippy
```

## License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

---

**mtable** - Simple, fast, and type-safe pattern matching for Rust 🦀
