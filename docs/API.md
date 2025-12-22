# Little Lookup API Reference

Little Lookup is an HTTP-based Key/Value store for strings. This document describes all available API endpoints, their parameters, authentication, and usage examples.

## Base URL

```
http://localhost:8088
```

The server runs on port **8088** by default.

## Table of Contents

- [Authentication](#authentication)
- [Namespaces](#namespaces)
- [Endpoints](#endpoints)
  - [Index](#index)
  - [Get](#get)
  - [Update](#update)
  - [History](#history)
  - [List](#list)
  - [Script](#script)
  - [Delete](#delete)

## Authentication

Little Lookup supports optional Pre-Shared Key (PSK) authentication for read and write operations.

### Setting PSK Keys

Configure PSKs using environment variables:

```bash
export LITTLE_LOOKUP_PSK_READ="your-read-key"
export LITTLE_LOOKUP_PSK_WRITE="your-write-key"
```

### PSK Types

- **Read PSK** (`LITTLE_LOOKUP_PSK_READ`): Required for `/get`, `/history`, `/list`, and `/script` endpoints
- **Write PSK** (`LITTLE_LOOKUP_PSK_WRITE`): Required for `/update` and `/delete` endpoints

### Authentication Errors

When PSK authentication is configured:

- **Missing PSK**: Returns `401 Unauthorized` with response `"PSK required"`
- **Incorrect PSK**: Returns `401 Unauthorized` with response `"Incorrect PSK"`

### Example: Using PSK

```bash
# With read PSK
curl "http://localhost:8088/get/mykey?psk=your-read-key"

# With write PSK
curl "http://localhost:8088/update/mykey/myvalue?psk=your-write-key"
```

## Namespaces

Namespaces allow you to organize and isolate keys within separate logical containers. All endpoints support namespace isolation.

### Default Namespace

If no namespace is specified, Little Lookup uses the **`default`** namespace.

### Specifying a Namespace

Use either `ns` or `namespace` query parameter (both work identically):

```bash
# Using 'ns' parameter
?ns=production

# Using 'namespace' parameter
?namespace=production

# Both are equivalent
```

### Namespace Behavior

- Keys are completely isolated by namespace
- The same key can exist with different values in different namespaces
- Namespace names are case-sensitive
- Operations only affect keys in the specified namespace

### Example: Multiple Namespaces

```bash
# Set key in 'production' namespace
curl "http://localhost:8088/update/db_host/prod.example.com?ns=production"

# Set same key in 'staging' namespace
curl "http://localhost:8088/update/db_host/staging.example.com?ns=staging"

# Retrieve from production
curl "http://localhost:8088/get/db_host?ns=production"
# Returns: prod.example.com

# Retrieve from staging
curl "http://localhost:8088/get/db_host?ns=staging"
# Returns: staging.example.com
```

## Endpoints

### Index

Returns an HTML page listing available endpoints.

#### Request

```
GET /
```

#### Query Parameters

None

#### Response

- **Status**: `200 OK`
- **Content-Type**: `text/html`
- **Body**: HTML page with routes documentation

#### Examples

```bash
curl http://localhost:8088/
```

Response (HTML):
```html
<p>Testing CD</p>
<p>Routes:</p>
<ul>
<li>/get/$KEY : Get val for $KEY</li>
<li>/history/$KEY : Get history for $KEY</li>
<li>/update/$KEY/$VAL : Update $VAL for $KEY</li>
<li>/list?delim=$FOO : List all keys, optional custom delimiter $BAR (defaults to space)</li>
<li>/script : Get bash script to export all keys</li>
<li>/delete/$KEY : Delete $VAL for $KEY</li>
</ul>
```

### Get

Retrieves the current value for a specific key.

#### Request

```
GET /get/{key}
```

#### Path Parameters

| Parameter | Description |
|-----------|-------------|
| `key` | The key to retrieve (required) |

#### Query Parameters

| Parameter | Description | Required |
|-----------|-------------|----------|
| `psk` | Pre-Shared Key for authentication | Only if `LITTLE_LOOKUP_PSK_READ` is set |
| `ns` / `namespace` | Namespace for the key (default: `default`) | No |

#### Response

- **Status**: `200 OK` if key exists
- **Status**: `404 Not Found` if key doesn't exist
- **Content-Type**: `text/plain`
- **Body**: The value associated with the key

#### Examples

```bash
# Basic get
curl http://localhost:8088/get/mykey

# Get with namespace
curl http://localhost:8088/get/mykey?ns=production

# Get with PSK authentication
curl http://localhost:8088/get/mykey?psk=my-read-key

# Combine namespace and PSK
curl http://localhost:8088/get/mykey?ns=production&psk=my-read-key
```

#### Response Examples

Success:
```
Status: 200 OK
Body: myvalue
```

Not found:
```
Status: 404 Not Found
Body: Undefined
```

### Update

Creates or updates a key with a new value. Maintains full version history.

#### Request

```
GET /update/{key}/{value}
```

**Note**: Despite using HTTP GET, this is a write operation and requires the write PSK.

#### Path Parameters

| Parameter | Description |
|-----------|-------------|
| `key` | The key to update (required) |
| `value` | The new value to set (required) |

#### Query Parameters

| Parameter | Description | Required |
|-----------|-------------|----------|
| `psk` | Pre-Shared Key for authentication | Only if `LITTLE_LOOKUP_PSK_WRITE` is set |
| `ns` / `namespace` | Namespace for the key (default: `default`) | No |

#### Response

- **Status**: `200 OK` on success
- **Status**: `401 Unauthorized` if PSK authentication fails
- **Status**: `500 Internal Server Error` on database error
- **Content-Type**: `text/plain`
- **Body**: The value that was set

#### Behavior

- Creates a new key if it doesn't exist
- Updates an existing key with a new value
- Each update is timestamped and stored in history
- The most recent value is returned on GET

#### Examples

```bash
# Basic update
curl http://localhost:8088/update/mykey/myvalue

# Update with namespace
curl http://localhost:8088/update/mykey/myvalue?ns=production

# Update with PSK authentication
curl http://localhost:8088/update/mykey/myvalue?psk=my-write-key

# Update with namespace and PSK
curl http://localhost:8088/update/mykey/myvalue?ns=production&psk=my-write-key

# Update with special characters (URL-encoded)
curl "http://localhost:8088/update/db_url/$(echo 'postgres://user:pass@localhost/db' | jq -sRr @uri)"
```

#### Response Examples

Success:
```
Status: 200 OK
Body: myvalue
```

Authentication required:
```
Status: 401 Unauthorized
Body: PSK required
```

### History

Retrieves all historical values for a key, ordered newest to oldest.

#### Request

```
GET /history/{key}
```

#### Path Parameters

| Parameter | Description |
|-----------|-------------|
| `key` | The key to retrieve history for (required) |

#### Query Parameters

| Parameter | Description | Required |
|-----------|-------------|----------|
| `psk` | Pre-Shared Key for authentication | Only if `LITTLE_LOOKUP_PSK_READ` is set |
| `ns` / `namespace` | Namespace for the key (default: `default`) | No |

#### Response

- **Status**: `200 OK` if key exists
- **Status**: `404 Not Found` if key doesn't exist
- **Content-Type**: `text/html`
- **Body**: HTML-formatted list of values (one per line) in a `<pre>` tag

#### Behavior

- Returns all values ever set for a key, in reverse chronological order (newest first)
- Each value is on a separate line
- Returns in HTML format for browser viewing

#### Examples

```bash
# Get full history
curl http://localhost:8088/history/mykey

# History for specific namespace
curl http://localhost:8088/history/mykey?ns=production

# History with authentication
curl http://localhost:8088/history/mykey?psk=my-read-key
```

#### Response Examples

Success (with multiple versions):
```
Status: 200 OK
Content-Type: text/html

<pre>
value_v3
value_v2
value_v1
</pre>
```

Not found:
```
Status: 404 Not Found
Body: Undefined
```

### List

Lists all keys and their current values in a namespace.

#### Request

```
GET /list
```

#### Query Parameters

| Parameter | Description | Required |
|-----------|-------------|----------|
| `delim` | Delimiter between key and value (default: space) | No |
| `psk` | Pre-Shared Key for authentication | Only if `LITTLE_LOOKUP_PSK_READ` is set |
| `ns` / `namespace` | Namespace to list (default: `default`) | No |

#### Response

- **Status**: `200 OK`
- **Content-Type**: `text/html`
- **Body**: HTML-formatted list in `<pre>` tag with format: `{key}{delimiter}{value}` per line

#### Behavior

- Lists all unique keys in the namespace
- Each key appears only once with its current value
- Default delimiter is a space character
- Custom delimiters are useful for parsing and scripting
- Output is HTML-formatted for browser viewing

#### Examples

```bash
# List all keys in default namespace
curl http://localhost:8088/list

# List with custom delimiter (pipe)
curl "http://localhost:8088/list?delim=|"

# List specific namespace
curl http://localhost:8088/list?ns=production

# List with namespace and custom delimiter
curl "http://localhost:8088/list?ns=production&delim=,"

# List with authentication
curl "http://localhost:8088/list?psk=my-read-key"
```

#### Response Examples

Success:
```
Status: 200 OK
Content-Type: text/html

<pre>
key1 value1
key2 value2
key3 value3
</pre>
```

With custom delimiter (pipe):
```
Status: 200 OK
Content-Type: text/html

<pre>
key1|value1
key2|value2
key3|value3
</pre>
```

Empty namespace:
```
Status: 200 OK
Content-Type: text/html

<pre>
</pre>
```

### Script

Generates a bash shell script that exports all keys as environment variables.

#### Request

```
GET /script
```

#### Query Parameters

| Parameter | Description | Required |
|-----------|-------------|----------|
| `psk` | Pre-Shared Key for authentication | Only if `LITTLE_LOOKUP_PSK_READ` is set |
| `ns` / `namespace` | Namespace to export (default: `default`) | No |

#### Response

- **Status**: `200 OK`
- **Content-Type**: `text/html`
- **Body**: HTML-formatted bash script in `<pre>` tag

#### Behavior

- Generates a valid bash script with `export` statements
- Each key becomes a shell variable
- Values are properly quoted
- Script can be sourced or executed directly
- Useful for loading configuration into shell environments

#### Examples

```bash
# Get export script for default namespace
curl http://localhost:8088/script > config.sh
source config.sh

# Get export script for specific namespace
curl "http://localhost:8088/script?ns=production" > prod-config.sh
source prod-config.sh

# Get script with authentication
curl "http://localhost:8088/script?psk=my-read-key" > config.sh

# Execute directly in shell
eval "$(curl -s http://localhost:8088/script)"
```

#### Response Examples

Success:
```
Status: 200 OK
Content-Type: text/html

<pre>
#!/bin/bash
export key1='value1'
export key2='value2'
export key3='value3'
</pre>
```

#### Usage Example

```bash
# Save and source the script
curl http://localhost:8088/script > env.sh
source env.sh
echo $key1  # Prints: value1

# Or execute directly
eval "$(curl -s http://localhost:8088/script)"
echo $key2  # Prints: value2
```

### Delete

Deletes all versions of a key from a namespace.

#### Request

```
GET /delete/{key}
```

**Note**: Despite using HTTP GET, this is a destructive operation requiring the write PSK.

#### Path Parameters

| Parameter | Description |
|-----------|-------------|
| `key` | The key to delete (required) |

#### Query Parameters

| Parameter | Description | Required |
|-----------|-------------|----------|
| `psk` | Pre-Shared Key for authentication | Only if `LITTLE_LOOKUP_PSK_WRITE` is set |
| `ns` / `namespace` | Namespace for the key (default: `default`) | No |

#### Response

- **Status**: `200 OK` on success
- **Status**: `401 Unauthorized` if PSK authentication fails
- **Status**: `500 Internal Server Error` on database error
- **Content-Type**: `text/plain`
- **Body**: `"{count} items deleted"`

#### Behavior

- Deletes ALL versions of a key from history
- Operation is permanent and cannot be undone
- Returns count of deleted records
- If key doesn't exist, returns `0 items deleted`
- Only affects the specified namespace

#### Examples

```bash
# Delete key from default namespace
curl http://localhost:8088/delete/mykey

# Delete from specific namespace
curl http://localhost:8088/delete/mykey?ns=production

# Delete with authentication
curl http://localhost:8088/delete/mykey?psk=my-write-key

# Delete with namespace and authentication
curl http://localhost:8088/delete/mykey?ns=production&psk=my-write-key
```

#### Response Examples

Success (key existed with 2 versions):
```
Status: 200 OK
Body: 2 items deleted
```

Success (key didn't exist):
```
Status: 200 OK
Body: 0 items deleted
```

Authentication error:
```
Status: 401 Unauthorized
Body: PSK required
```

## Common Usage Patterns

### Configuration Management

Store environment-specific configuration:

```bash
# Set configuration values
curl http://localhost:8088/update/DATABASE_URL/postgres://prod-db:5432/app?ns=production
curl http://localhost:8088/update/API_KEY/secret-key-here?ns=production
curl http://localhost:8088/update/LOG_LEVEL/info?ns=production

# Retrieve and use
source <(curl -s http://localhost:8088/script?ns=production)
psql $DATABASE_URL
```

### Feature Flags

Manage feature flags across environments:

```bash
# Enable feature
curl http://localhost:8088/update/FEATURE_NEW_UI/true?ns=production

# Check if enabled
curl http://localhost:8088/get/FEATURE_NEW_UI?ns=production
```

### Version Tracking

View when values changed:

```bash
# See full history
curl http://localhost:8088/history/DATABASE_URL?ns=production
```

### Bulk Configuration

Parse the list endpoint for automation:

```bash
# Get all config as key=value pairs
curl -s http://localhost:8088/list?delim==  | grep -v '^$'

# Load into environment
eval "$(curl -s http://localhost:8088/list?delim== | sed 's/^/export /')"
```

## Error Handling

### HTTP Status Codes

| Status | Meaning | Example |
|--------|---------|---------|
| `200 OK` | Request successful | Get/Update/Delete successful |
| `401 Unauthorized` | PSK authentication failed | Wrong or missing PSK |
| `404 Not Found` | Key doesn't exist | Get/History on non-existent key |
| `500 Internal Server Error` | Database or server error | Connection failure |

### Common Error Responses

```bash
# PSK required
curl http://localhost:8088/get/key?psk=wrong
# Status: 401 Unauthorized
# Body: Incorrect PSK

# Key not found
curl http://localhost:8088/get/nonexistent
# Status: 404 Not Found
# Body: Undefined

# Database connection error
# Status: 500 Internal Server Error
# Body: Database connection failed
```

## Security Considerations

### PSK Security

- PSKs are passed as query parameters (visible in URLs and logs)
- Use HTTPS in production to prevent PSK exposure
- Rotate PSKs regularly
- Use different PSKs for read and write operations
- Consider network-level access controls in addition to PSKs

### Data Security

- Values are stored unencrypted in the database
- Access control relies on PSK authentication
- Namespace isolation is logical, not cryptographic
- Treat the database as a sensitive resource

### SQL Injection Prevention

Little Lookup uses Diesel ORM with parameterized queries, preventing SQL injection attacks. All user input is safely escaped.

## Rate Limiting

Little Lookup does not implement rate limiting. Deploy behind a reverse proxy (nginx, HAProxy) to add rate limiting if needed.

## Deployment Notes

### Environment Variables

- `LITTLE_LOOKUP_PSK_READ`: Pre-shared key for read operations
- `LITTLE_LOOKUP_PSK_WRITE`: Pre-shared key for write operations
- `LITTLE_LOOKUP_DATABASE`: PostgreSQL connection string (default: `postgres://docker:docker@localhost:15432/little-lookup`)
- `LITTLE_LOOKUP_POOL_SIZE_PER_WORKER`: Database connection pool size (default: `5`)
- `LITTLE_LOOKUP_WORKER_NUM`: Number of HTTP worker threads (default: `2`)

### Database

- PostgreSQL 9.6 or later required
- Automatic migrations run on startup
- Full version history maintained for all keys
- Namespace and key-based indexing for performance

## Examples

### Complete Workflow

```bash
# 1. Set up configuration for production
curl http://localhost:8088/update/app_version/1.2.3?ns=production
curl http://localhost:8088/update/log_level/info?ns=production
curl http://localhost:8088/update/database_host/prod.db.local?ns=production

# 2. List all configuration
curl http://localhost:8088/list?ns=production

# 3. Get specific value
curl http://localhost:8088/get/app_version?ns=production

# 4. View change history
curl http://localhost:8088/history/app_version?ns=production

# 5. Update a value (creates new history entry)
curl http://localhost:8088/update/app_version/1.2.4?ns=production

# 6. Export as shell script
curl http://localhost:8088/script?ns=production | source /dev/stdin

# 7. Delete old configuration
curl http://localhost:8088/delete/database_host?ns=production
```

### With Authentication

```bash
# Set PSKs
export LITTLE_LOOKUP_PSK_READ="read-secret-123"
export LITTLE_LOOKUP_PSK_WRITE="write-secret-456"

# Start server
./little-lookup

# Now all requests require PSK
curl http://localhost:8088/get/key  # 401: PSK required
curl http://localhost:8088/get/key?psk=read-secret-123  # Works
curl http://localhost:8088/update/key/value?psk=write-secret-456  # Works
```

### Multiple Namespaces

```bash
# Development environment
curl http://localhost:8088/update/db_host/localhost?ns=dev
curl http://localhost:8088/update/db_port/5432?ns=dev

# Staging environment
curl http://localhost:8088/update/db_host/staging-db.internal?ns=staging
curl http://localhost:8088/update/db_port/5432?ns=staging

# Production environment
curl http://localhost:8088/update/db_host/prod-db.internal?ns=production
curl http://localhost:8088/update/db_port/5432?ns=production

# Verify isolation
curl http://localhost:8088/list?ns=dev
curl http://localhost:8088/list?ns=staging
curl http://localhost:8088/list?ns=production
```
