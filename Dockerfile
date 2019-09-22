# steps based on https://gist.github.com/jamesproud/4022da405709a633ba7f021a36d7b462

## BUILD IMAGE
FROM ekidd/rust-musl-builder:stable as cargo-build

ADD . ./

RUN sudo chown -R rust:rust .

RUN sudo apt-get update
RUN sudo apt-get install -y \
        libsqlite-dev \
        libsqlite3-dev \
        musl-tools \
        sqlite \
        sqlite3
# RUN rustup target add x86_64-unknown-linux-musl

RUN cargo build --release

# Compile command from previous image (rustlang/rust) which had problems compiling diesel w/ musl (alpine linux)
# RUN RUSTFLAGS=-Clinker=musl-gcc cargo build --release --target=x86_64-unknown-linux-musl


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

COPY --from=cargo-build /home/rust/src/target/x86_64-unknown-linux-musl/release/little-lookup .

EXPOSE 8088

ENTRYPOINT ["/home/app/little-lookup"]
