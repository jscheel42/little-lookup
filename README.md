# Little Lookup

Little Lookup is a simple, fast, and lightweight HTTP-based Key/Value store for strings built with Rust and Actix-web. It features namespace support, PSK authentication, and full version history tracking.

## Architecture

Little Lookup is built on a modern Rust stack designed for performance and reliability:

### Tech Stack
- **Language**: Rust (2021 edition)
- **Web Framework**: Actix-web 4.11.0 - High-performance async HTTP server
- **Database ORM**: Diesel 2.2.12 - Type-safe SQL query builder
- **Database**: PostgreSQL - Reliable relational database with JSONB support
- **Connection Pool**: r2d2 - Efficient connection pooling for database access
- **Async Runtime**: Actix-rt - Lightweight async executor based on Tokio

### Project Structure

```
little-lookup/
├── src/
│   ├── main.rs              # Server setup, routes, initialization
│   ├── db_connection.rs     # Database connection pool management
│   ├── schema.rs            # Diesel schema definitions
│   ├── util.rs              # Utility functions (PSK, namespace parsing)
│   ├── handlers/
│   │   ├── mod.rs           # Handler module
│   │   └── items.rs         # Route handlers (get, update, delete, list, history)
│   └── models/
│       ├── mod.rs           # Model module
│       └── item.rs          # Item model and database operations
├── migrations/              # Diesel SQL migrations
├── Cargo.toml               # Rust dependencies and metadata
└── docker-entrypoint.sh     # Docker startup script
```

### Key Features

**Namespace Isolation**: Each key can exist independently in multiple namespaces, providing logical data separation without separate databases. Perfect for multi-tenant scenarios or organizing related data.

**Pre-Shared Key (PSK) Authentication**: Optional read and write PSK protection via environment variables. Separate PSKs for read and write operations provide granular access control.

**Version History**: Every update creates a new database record while preserving all historical values. Query the full history of any key at any time, with timestamps for auditing.

**Type Safety**: Leverages Rust's type system and Diesel ORM for compile-time SQL safety and prevention of common database errors.

**Connection Pooling**: Configurable connection pool with per-worker settings for optimal performance under load.

## CD

Online until my AlloyDB free trial ends:
* https://little-lookup-945130679167.us-central1.run.app

## Docker / Helm

Docker image available at: https://hub.docker.com/r/jscheel42/little-lookup

Helm chart available at: https://github.com/jscheel42/helm-charts/tree/master/little-lookup

## Build and Run

### Compile

Ubuntu 18.04 packages required
```
apt install libpq-dev libssl-dev pkg-config
```

```
### Compile

Ubuntu packages required (including Ubuntu 24.04)
```
apt install libpq-dev libssl-dev pkg-config build-essential
```

For Rust installation, consider using rustup:
```
curl --proto "=https" --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

```
cargo build --release
```
```

#### PSK
Set the read and/or write PSK if wanted
```
export LITTLE_LOOKUP_PSK_READ="read-psk-here"
export LITTLE_LOOKUP_PSK_WRITE="write-psk-here"
```
You will need to include the PSK in your requests, e.g.
```
?psk=read-psk-here
localhost:8088/get/foo?psk=read-psk-here
localhost:8088/update/foo/bar?psk=write-psk-here
```

### Namespace
Namespace support is available for all commands via "ns" or "namespace" settings
```
?ns=bar
?namespace=bar
localhost:8088/get/foo?ns=bar
localhost:8088/get/foo?namespace=bar
```

### Run

```
./target/release/little-lookup
```

## Usage

### Set value

Set key (foo) to value (bar)
```
localhost:8088/update/foo/bar
```

With namespace:
```
localhost:8088/update/foo/bar?ns=production
```

With PSK authentication:
```
localhost:8088/update/foo/bar?psk=your-write-psk
```

### Get value(s)

Retrieve current value for key (foo)
```
localhost:8088/get/foo
```

Returns the most recent value set for the key. Returns "Undefined" (404) if key doesn't exist.

With namespace:
```
localhost:8088/get/foo?ns=production
```

With read PSK:
```
localhost:8088/get/foo?psk=your-read-psk
```

Retrieve complete history of values for key (foo)
```
localhost:8088/history/foo
```

Shows all values ever set for the key, ordered by most recent first. Useful for auditing changes and understanding how a value has evolved.

With namespace:
```
localhost:8088/history/database_password?ns=production&psk=your-read-psk
```

Retrieve values for all keys
```
localhost:8088/list
```

Returns all current key-value pairs in the default namespace as `key value` pairs.

Retrieve values for all keys with custom delimiter
```
localhost:8088/list?delim=|
```

Customize the separator between key and value. Default is space. Useful for parsing in scripts.

With namespace:
```
localhost:8088/list?ns=staging&delim=:
```

### Delete value(s)

Delete key (foo) and all its history
```
localhost:8088/delete/foo
```

Removes all versions of the key from the specified namespace.

With namespace:
```
localhost:8088/delete/foo?ns=production
```

With PSK:
```
localhost:8088/delete/foo?psk=your-write-psk&ns=staging
```

### Generate bash script

Create bash script to export values for all keys
```
localhost:8088/script
```

Returns a ready-to-use bash script that exports all keys as environment variables:
```bash
#!/bin/bash
export API_KEY='value1'
export DATABASE_URL='postgres://...'
export DEBUG='false'
```

With namespace:
```
localhost:8088/script?ns=production
```

Save script to file and source:
```bash
curl -s "localhost:8088/script?ns=production&psk=read-psk" > config.sh
source config.sh
```

### Error Handling

Common error responses:

**401 Unauthorized** - PSK required or incorrect
```
PSK required
Incorrect PSK
```

**404 Not Found** - Key doesn't exist
```
Undefined
```

**500 Internal Server Error** - Database connection failed
```
Database connection failed
Failed to update item
```

## Testing

Little Lookup includes a comprehensive test suite covering all major features and edge cases.

### Running Tests

Run all tests:
```
cargo test
```

Run tests with output (useful for debugging):
```
cargo test -- --nocapture
```

Run a specific test:
```
cargo test test_index
```

Run tests for a specific module:
```
cargo test handlers::items
```

### Test Coverage

The test suite includes **57 passing tests** covering:

**Core Operations** (8 tests)
- Index page rendering
- Get, update, delete, and list operations
- Script generation
- Query parameter parsing

**Namespace Isolation** (4 tests)
- Separate values per namespace
- Distinct key handling
- List operations per namespace
- History isolation

**Authentication** (8 tests)
- PSK validation for read and write operations
- Missing PSK detection
- Incorrect PSK rejection
- Read/write separation
- PSK-optional mode

**Version History** (3 tests)
- History retrieval and ordering
- Multiple versions per key
- Full CRUD workflow with history

**Data Integrity** (4 tests)
- SQL injection prevention
- Unicode key and value support
- Special character handling
- Malformed query parameter handling

**Database Operations** (16 tests)
- Connection pooling
- Migration verification
- Item CRUD operations
- Pool reuse and multiple connections

**Error Handling** (6 tests)
- Not found responses
- Database connection failures
- Empty namespace handling

### Test Requirements

Tests require:
- PostgreSQL database running and accessible
- `LITTLE_LOOKUP_DATABASE` environment variable set (or defaults to `postgres://localhost/little_lookup_test`)
- Proper database permissions to create tables and run migrations

Test database automatically uses migrations to create the required schema on startup.

## Development

### Building

Compile for release:
```
cargo build --release
```

Compile for debug (faster builds, slower execution):
```
cargo build
```

Build output will be in `target/release/little-lookup` or `target/debug/little-lookup`.

### Linting and Formatting

Check code with Clippy (Rust linter):
```
cargo clippy
```

Auto-format code to match project style:
```
cargo fmt
```

Always run `cargo fmt` before committing.

### Code Style Guidelines

From AGENTS.md:
- **Imports**: Group `std`, external crates, then local modules (one per line)
- **Formatting**: 4-space indentation; run `cargo fmt` before commits
- **Types**: Explicit return types; use type aliases for complex types
- **Naming**: `snake_case` for functions/variables, `CamelCase` for types/enums
- **Error handling**: Return `Result<T, diesel::result::Error>`; avoid `unwrap()`, use `?` or `match`
- **Database**: Use Diesel ORM exclusively; models derive `Queryable`/`Insertable`
- **Security**: Validate input; avoid raw SQL interpolation (Diesel handles this)

### Environment Variables

Configuration via environment variables (all prefixed with `LITTLE_LOOKUP_`):

```
LITTLE_LOOKUP_DATABASE              # PostgreSQL connection string
                                     # Default: postgres://localhost/little_lookup_test
                                     # Example: postgres://user:pass@host:5432/dbname

LITTLE_LOOKUP_PSK_READ              # Pre-shared key for read operations (optional)
                                     # If set, all read operations require this PSK
                                     # Example: LITTLE_LOOKUP_PSK_READ="secret-read-key"

LITTLE_LOOKUP_PSK_WRITE             # Pre-shared key for write operations (optional)
                                     # If set, all write operations require this PSK
                                     # Example: LITTLE_LOOKUP_PSK_WRITE="secret-write-key"

LITTLE_LOOKUP_POOL_SIZE_PER_WORKER # Database connection pool size per worker
                                     # Default: 10
                                     # Example: LITTLE_LOOKUP_POOL_SIZE_PER_WORKER=20

LITTLE_LOOKUP_WORKER_NUM            # Number of worker threads
                                     # Default: auto-detected from CPU count
                                     # Example: LITTLE_LOOKUP_WORKER_NUM=4
```

### Database Setup

Little Lookup uses Diesel migrations to manage the database schema. On startup, the application automatically:
1. Connects to PostgreSQL using `LITTLE_LOOKUP_DATABASE`
2. Runs pending migrations from the `migrations/` directory
3. Creates the `items` table if it doesn't exist

Migrations are embedded in the binary, so no separate migration files are needed at runtime.

To set up PostgreSQL locally:
```bash
# Create a test database
createdb little_lookup_test

# Run the application (migrations run automatically)
./target/release/little-lookup

# Or set custom database URL
export LITTLE_LOOKUP_DATABASE="postgres://user:password@localhost:5432/my_lookup"
./target/release/little-lookup
```

### Local Development Workflow

1. **Make changes** to source code
2. **Run format**: `cargo fmt`
3. **Check linter**: `cargo clippy`
4. **Run tests**: `cargo test`
5. **Build**: `cargo build --release`
6. **Test manually**: `./target/release/little-lookup`

Example development session:
```bash
# Start PostgreSQL (if not running)
brew services start postgresql  # macOS
# or
systemctl start postgresql       # Linux

# Make code changes
vim src/handlers/items.rs

# Format, lint, test
cargo fmt && cargo clippy && cargo test

# Build and run
cargo build --release
./target/release/little-lookup
```
