# Little Lookup

Little Lookup is a simple Key/Value store for strings.

## Docker / Helm

Docker image available at: https://hub.docker.com/r/jscheel42/little-lookup

Helm chart available at: https://github.com/jscheel42/helm-charts/tree/master/little-lookup

## Build and Run

### Compile

```
cargo build --release
```

### Configure

Set the location for the sqlite database
```
export LITTLE_LOOKUP_DATABASE=/your/chosen/path.db
```

### Run

```
./target/release/little-lookup
```

## Usage

### Set value

Set key (foo) to value (bar)
```
localhost:8000/item/foo/bar
```

### Get value(s)

Retrieve value for key (foo)
```
localhost:8000/item/foo
```

Retrieve values for all keys
```
localhost:8000/list
```

Retrieve values for all keys that match filter (sql like '%X%')
```
localhost:8000/list?filter=<x>
```

Retrieve values for all keys with custom delimiter <y>
```
localhost:8000/list?delim=<y>
```
