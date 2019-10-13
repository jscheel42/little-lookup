# steps based on https://gist.github.com/jamesproud/4022da405709a633ba7f021a36d7b462

##
## BUILD IMAGE
##

FROM ekidd/rust-musl-builder:stable as cargo-build

WORKDIR /usr/local/src

ADD . ./

RUN sudo chown -R rust:rust .

RUN cargo build --release
RUN chown 1000:1000 /usr/local/src/target/x86_64-unknown-linux-musl/release/little-lookup

##
## FINAL IMAGE
##

FROM alpine:latest

RUN apk --no-cache add ca-certificates

RUN addgroup -g 1000 app
RUN adduser -D -s /bin/sh -u 1000 -G app app
RUN mkdir /data
RUN chown app:app /data

ENV LITTLE_LOOKUP_DATABASE postgres://docker:docker@localhost:5432/little-lookup

WORKDIR /home/app

COPY --from=cargo-build /usr/local/src/target/x86_64-unknown-linux-musl/release/little-lookup .
USER app

RUN ln -s /pgpass/.pgpass .pgpass

EXPOSE 8088

ENTRYPOINT ["/home/app/little-lookup"]
