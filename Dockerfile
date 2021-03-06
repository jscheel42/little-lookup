# steps based on https://gist.github.com/jamesproud/4022da405709a633ba7f021a36d7b462

##
## BUILD IMAGE
##

FROM rust:1-slim-buster as cargo-build

WORKDIR /usr/local/src

RUN apt-get update &&\
        apt-get install -y \
            libpq-dev \
            libssl-dev \
            pkg-config &&\
        rm -rf /var/lib/apt/lists/*

ADD src ./src
ADD migrations ./migrations
ADD Cargo.toml ./
ADD Cargo.lock ./
ADD diesel.toml ./
ADD docker-entrypoint.sh ./

RUN cargo build --release
RUN chown 1000:1000 /usr/local/src/target/release/little-lookup

##
## FINAL IMAGE
##

FROM debian:buster-slim

RUN apt-get update &&\
        apt-get install -y \
            ca-certificates \
            libpq5 &&\
        rm -rf /var/lib/apt/lists/*

RUN addgroup -gid 1000 app
RUN adduser --disabled-password --shell /bin/bash --uid 1000 --ingroup app app
RUN mkdir /data
RUN chown app:app /data

ENV LITTLE_LOOKUP_DATABASE postgres://docker:docker@localhost:5432/little-lookup

WORKDIR /home/app

COPY --from=cargo-build /usr/local/src/target/release/little-lookup .
USER app
ADD docker-entrypoint.sh .

EXPOSE 8088

ENTRYPOINT ["/home/app/docker-entrypoint.sh"]
