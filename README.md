# NFA Regex Engine in Rust

A toy regex engine implementation using NFA (Non-deterministic Finite Automaton)

## Dependency

- `regex-syntax`: Parse the regex pattern into an AST using regex-syntax
- `anyhow`

## Supported Syntax

- Literal: `1` / `2` / `3` ...
- Concat
- Look: `^` / `$`
- Alternation: `|`
- Repetition: `*`
- Class: `[ ]` | `[^ ]`
- Capture Group: `()`

## Usage

```rust
use rsgex::Engine;

let e = Engine::try_from("^[1-9][0-9]{11}$").unwrap();

assert!(e.test("17700012450"));
// exec method returns a HashMap
assert_eq!(e.exec("17700012450").unwrap().get(&0.to_string()).unwrap().clone(), "17700012450");
```
