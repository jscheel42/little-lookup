# AGENTS.md

## Build, Lint, and Test Commands
- **Build**: `cargo build --release`
- **Test all**: `cargo test`
- **Test single**: `cargo test <test_name>` (e.g., `cargo test test_index`)
- **Lint**: `cargo clippy`
- **Format**: `cargo fmt`

## Code Style Guidelines
- **Imports**: Group `std`, external crates, then local (`crate::`) modules; one per line.
- **Formatting**: 4‑space indentation; run `cargo fmt` before commits.
- **Types**: Explicit return types; use aliases for complex types (e.g., `Pool`, `PooledConnection`).
- **Naming**: `snake_case` for functions/variables, `CamelCase` for types/enums.
- **Error handling**: Return `Result<T, diesel::result::Error>`; avoid `unwrap()`, use `?` or `match`.
- **Database**: Use Diesel ORM; models derive `Queryable`/`Insertable`.
- **Environment**: Config via `LITTLE_LOOKUP_*` env vars.
- **Testing**: `#[cfg(test)]` modules; async tests use `#[actix_rt::test]`; require Postgres.
- **Security**: Validate input; avoid raw SQL interpolation.

