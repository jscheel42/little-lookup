# Little Lookup

Little Lookup is a simple Key/Value store for strings.

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

### Get value(s)

Retrieve value for key (foo)
```
localhost:8088/get/foo
```

Retrieve history of values for key (foo)
```
localhost:8088/history/foo
```

Retrieve values for all keys
```
localhost:8088/list
```

Retrieve values for all keys with custom delimiter <y>
```
localhost:8088/list?delim=<y>
```

Delete value for key (foo)
```
localhost:8088/delete/foo
```

Create bash script to export values for all keys
```
localhost:8088/script
```
