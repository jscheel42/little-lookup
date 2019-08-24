# Little Lookup

Little Lookup is a simple Key/Value store for strings.

## Docker / Helm

Docker image available at: https://cloud.docker.com/u/jscheel42/repository/docker/jscheel42/little-lookup

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

Retrieve value for key (foo)
```
localhost:8000/item/foo
```
