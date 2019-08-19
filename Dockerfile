FROM rustlang/rust:nightly-slim

RUN mkdir /app /data
WORKDIR /app

ENV LITTLE_LOOKUP_DATABASE /data/default.db

# ADD . .

ADD src src
ADD Cargo.lock .
ADD Cargo.toml .

RUN ls /app

RUN cargo install --path .

ENTRYPOINT [ "/usr/local/cargo/bin/little-lookup" ]