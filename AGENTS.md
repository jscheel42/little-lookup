# AGENTS.md

## Build, Lint, and Test Commands

- **Build**: `cargo build --release`
- **Test all**: `cargo test`
- **Test single**: `cargo test <test_name>` (e.g., `cargo test test_index`)
- **Lint**: `cargo clippy`
- **Format**: `cargo fmt`

## Code Style Guidelines

- **Imports**: Group std lib, external crates, then local (`crate::`) modules; one per line
- **Formatting**: 4-space indentation, run `cargo fmt` before commits
- **Types**: Explicit return types on functions; use type aliases for complex types (see `Pool`, `PooledConnection`)
- **Naming**: `snake_case` for functions/variables, `CamelCase` for types/structs/enums
- **Error handling**: Return `Result<T, diesel::result::Error>` or similar; avoid `unwrap()` in production code, use `match` or `?`
- **Database**: Use Diesel ORM; define models with `#[derive(Queryable)]` / `#[derive(Insertable)]`
- **Environment**: Config via env vars prefixed `LITTLE_LOOKUP_*`
- **Testing**: Use `#[cfg(test)]` modules; async tests use `#[actix_rt::test]`; tests require running Postgres
- **Security**: Validate user input; avoid raw SQL string interpolation (see `replace_into` for what NOT to do)
