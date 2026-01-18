# mtable

A fast and efficient Rust library for pattern matching based on trie data structures. Ideal for building lexers, tokenizers, and command parsers.

## Features

- 🚀 **Fast**: Built on an optimized trie structure with O(n) lookup time
- 🎯 **Pattern Matching**: Supports literal patterns, character classes `[abc]`, and repetitions `+`
- 🔤 **Customizable Alphabet**: Define your own set of valid characters
- 🎨 **Generic**: Works with any type `T: Clone + Debug` (enums, integers, structs)
- ✅ **Type-safe**: Robust error handling with `Result`
- 📦 **Zero Dependencies**: Only Rust standard library

## Quick Start

```rust
use mtable::Table;

#[derive(Debug, Clone, PartialEq)]
enum TokenType {
    Keyword,
    Operator,
    Identifier,
}

let mut lexer = Table::new("abcdefghijklmnopqrstuvwxyz".to_string());

// Map keywords to token types
lexer.add("if", TokenType::Keyword).unwrap();
lexer.add("while", TokenType::Keyword).unwrap();
lexer.add("return", TokenType::Keyword).unwrap();

// Check tokens
assert_eq!(lexer.get("if").unwrap(), Some(&TokenType::Keyword));
assert_eq!(lexer.get("while").unwrap(), Some(&TokenType::Keyword));
assert_eq!(lexer.get("other").unwrap(), None);
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

## Real-World Examples

### Lexer for Programming Language

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
}

let mut lexer = Table::new(
    "abcdefghijklmnopqrstuvwxyz0123456789+-=".to_string()
);

// Keywords
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

// Usage
assert_eq!(lexer.get("if").unwrap(), Some(&TokenType::If));
assert_eq!(lexer.get("var123").unwrap(), Some(&TokenType::Identifier));
assert_eq!(lexer.get("42").unwrap(), Some(&TokenType::Number));
assert_eq!(lexer.get("+").unwrap(), Some(&TokenType::Plus));
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

## Performance

The implementation uses a trie (prefix tree) data structure which provides:

- **Insertion**: O(m) where m is the pattern length
- **Lookup**: O(n) where n is the query string length
- **Memory**: Efficient prefix sharing between patterns

## Limitations

- **ASCII Only**: Patterns and queries must contain only ASCII characters
- **Limited Operators**: Supports only `[...]` and `+`, not `*`, `?`, or other regex features
- **No Negation**: Character classes don't support negation (e.g., `[^abc]`)
- **Fixed Alphabet**: The alphabet must be defined at creation and cannot be modified
- **No Ranges**: Character classes must list characters explicitly (no `[a-z]` syntax)

## Roadmap

Features planned for future releases:

- 🔜 **Substring Matching**: Find patterns anywhere in the input string, not just exact matches
- 🔜 **Multiple Match Results**: Return all matching patterns in a string
- 💭 **Optional Operator `?`**: Match zero or one occurrence
- 💭 **Kleene Star `*`**: Match zero or more occurrences
- 💭 **Range Syntax**: Support `[a-z]` and `[0-9]` notation in character classes
- 💭 **Match Position Info**: Return start/end positions of matches

## Use Cases

`mtable` is ideal for:

- ✅ **Lexical analysis**: Tokenizing source code or input
- ✅ **Command parsing**: Matching commands in CLI tools or protocols
- ✅ **Configuration parsing**: Recognizing configuration keys
- ✅ **Protocol parsers**: Matching protocol keywords or message types
- ✅ **Simple routing**: Mapping strings to handlers or IDs
- ✅ **Domain-specific languages**: Pattern matching for custom syntax

**Current version is best for:**
- Exact pattern matching from beginning to end of string
- Single-pass tokenization
- Fixed vocabulary recognition

**Not suitable for:**
- ❌ Complex regular expressions (use [regex](https://crates.io/crates/regex) crate)
- ❌ Unicode text processing

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
