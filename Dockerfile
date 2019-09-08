# steps based on https://gist.github.com/jamesproud/4022da405709a633ba7f021a36d7b462

## BUILD IMAGE
FROM rustlang/rust:nightly-slim as cargo-build

RUN mkdir /app /data
WORKDIR /app
ADD . .

RUN apt-get update
RUN apt-get install -y \
        libsqlite3-dev \
        musl-tools \
        sqlite3
RUN rustup target add x86_64-unknown-linux-musl

RUN ls /app

RUN RUSTFLAGS=-Clinker=musl-gcc cargo build --release --target=x86_64-unknown-linux-musl


## FINAL IMAGE
FROM alpine:latest

RUN apk --no-cache add ca-certificates

RUN addgroup -g 1000 app
RUN adduser -D -s /bin/sh -u 1000 -G app app
RUN mkdir /data
RUN chown app:app /data

ENV LITTLE_LOOKUP_DATABASE /data/default.db

USER app
WORKDIR /home/app

COPY --from=cargo-build /app/target/x86_64-unknown-linux-musl/release/little-lookup .

ENTRYPOINT ["/home/app/little-lookup"]
