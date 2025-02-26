# Little Lookup Development Guide

## Build & Test Commands
- Build: `cargo build --release`
- Run: `./target/release/little-lookup` or `cargo run`
- Dependencies: `apt install libpq-dev libssl-dev pkg-config` (Ubuntu)
- Test all: `cargo test`
- Test single: `cargo test test_name`
- Test module: `cargo test models::item::tests`

## Code Style Guidelines
- Indentation: 4 spaces
- Imports: Group standard library, external crates, then local modules
- Types: Explicit annotations for function parameters/returns
- Naming: `snake_case` for functions/variables, `CamelCase` for types/structs
- Error handling: Prefer `Result<T, Error>` over unwrap/expect where possible
- Database: Use Diesel ORM with proper schema definitions
- Environment: Configuration through environment variables (LITTLE_LOOKUP_*)
- Testing: Write unit tests with `#[cfg(test)]` and integration tests for handlers
- Comments: Prefer self-documenting code with minimal necessary comments
- Security: Be cautious with user input in SQL queries to prevent injections