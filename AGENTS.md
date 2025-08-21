# AGENTS.md

## Build, Lint, and Test Commands

- **Build**: `cargo build --release`
- **Test all**: `cargo test`
- **Test single**: `cargo test <test_name>`
- **Lint**: `cargo clippy` (if available)

## Code Style Guidelines

- **Imports**: Group standard library, external crates, then local modules
- **Formatting**: 4 spaces for indentation
- **Types**: Explicit annotations for function parameters/returns
- **Naming**: `snake_case` for functions/variables, `CamelCase` for types/structs
- **Error handling**: Prefer `Result<T, Error>` over unwrap/expect where possible
- **Database**: Use Diesel ORM with proper schema definitions
- **Environment**: Configuration through environment variables (LITTLE_LOOKUP_*)
- **Testing**: Write unit tests with `#[cfg(test)]` and integration tests for handlers
- **Comments**: Prefer self-documenting code with minimal necessary comments
- **Security**: Be cautious with user input in SQL queries to prevent injections

## Cursor Rules

None found.

## Copilot Instructions

None found.