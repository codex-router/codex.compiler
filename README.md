# codex-compiler

Fast grammar-verification compiler for **C**, **C++**, and **Java** source files.
The binary is named `codexc`.

---

## Features

- Lexes and parses C (`.c`), C++ (`.cpp`, `.cc`, `.cxx`, `.h`, `.hpp`, …), and Java (`.java`) files
- Reports errors and warnings with file, line, and column information
- Processes multiple files **in parallel** (powered by [Rayon](https://github.com/rayon-rs/rayon))
- Coloured terminal output (disable with `--no-color`)
- Optional timing information, per-file error limits, and a raw token dump for debugging

---

## Requirements

- [Rust](https://www.rust-lang.org/tools/install) 1.70 or later (`rustup` recommended)

---

## Setting Release Version

Update the crate version in `Cargo.toml` before creating a release.

### Option A: Edit `Cargo.toml` directly

Set the `[package]` version field:

```toml
[package]
name = "codex-compiler"
version = "1.0.0"
```

### Option B: Use `cargo set-version`

Install once:

```bash
cargo install cargo-edit
```

Set the version:

```bash
cargo set-version 1.0.0
```

Optional: create a matching git tag:

```bash
git tag v1.0.0
git push origin v1.0.0
```

---

## Building

### Debug build

```bash
cargo build
```

The binary is written to `target/debug/codexc`.

### Release build (optimised)

```bash
cargo build --release
```

The binary is written to `target/release/codexc`.
The release profile enables LTO and `opt-level = 3` for maximum performance.

---

## Running

### Via Cargo

```bash
cargo run -- <file> [file …] [options]
```

### Direct binary

```bash
# release build
./target/release/codexc samples/hello.c samples/Hello.java
```

---

## CLI Reference

```
Usage: codexc [OPTIONS] <FILES>…

Arguments:
  <FILES>…   Source files to verify (*.c *.cpp *.cc *.h *.hpp *.java)

Options:
  -f, --fast-fail              Stop after the first error in each file
  -e, --error-limit <N>        Maximum errors to report per file (0 = unlimited) [default: 0]
  -t, --timings                Print wall-clock timing after processing
      --dump-tokens            Print the full token stream before parsing (debug)
      --no-color               Disable coloured output
  -h, --help                   Print help
  -V, --version                Print version
```

### Examples

```bash
# Check a single C file
codexc samples/hello.c

# Check multiple files at once
codexc samples/hello.c samples/hello.cpp samples/Hello.java

# Stop at the first error and show timing
codexc -f -t samples/bad.c

# Limit output to 5 errors per file
codexc -e 5 samples/bad.c

# Debug: dump the token stream
codexc --dump-tokens samples/mini.c

# No colour (e.g. for CI logs)
codexc --no-color samples/*.c
```

---

## Sample Files

The `samples/` directory contains ready-to-use test inputs:

| File | Language | Purpose |
|------|----------|---------|
| `hello.c` | C | Valid C program |
| `hello.cpp` | C++ | Valid C++ program |
| `Hello.java` | Java | Valid Java class |
| `bad.c` | C | Intentional grammar errors |
| `Bad.java` | Java | Intentional Java errors |
| `mini.c` | C | Minimal C snippet |
| `test2.c` – `test8.c` | C | Various feature tests |

---

## Testing

Run the Rust unit and integration test suite:

```bash
cargo test
```

Run tests with output printed to stdout (useful for debugging):

```bash
cargo test -- --nocapture
```

Quickly smoke-test the compiler against the provided samples:

```bash
# Expect clean output
cargo run -- samples/hello.c samples/hello.cpp samples/Hello.java

# Expect reported errors
cargo run -- samples/bad.c samples/Bad.java
```

---

## Project Structure

```
Cargo.toml
src/
  main.rs          # CLI entry point, parallel file dispatch
  language.rs      # Language detection from file extension
  lexer.rs         # Tokeniser (C / C++ / Java)
  token.rs         # Token types and span
  error.rs         # Diagnostic types (errors / warnings)
  parser/
    mod.rs         # Shared recursive-descent cursor
    c_parser.rs    # C / C++ grammar rules
    java_parser.rs # Java grammar rules
samples/           # Example source files for manual testing
```

---

## Exit Codes

| Code | Meaning |
|------|---------|
| `0` | All files parsed without errors |
| `1` | One or more files contained grammar errors |

---

## License

Licensed under the **Apache License, Version 2.0**.

You may obtain a copy of the License at:
- http://www.apache.org/licenses/LICENSE-2.0

See [LICENSE](./LICENSE) for the full license text.
