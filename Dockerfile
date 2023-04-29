FROM rust:slim-bullseye

RUN apt-get update && \
    apt-get upgrade -y && \
    apt-get install -y musl-dev pkg-config libssl-dev

WORKDIR /usr/src/myapp
COPY . .

RUN cargo install --path .

ENTRYPOINT ["/usr/local/cargo/bin/nostcat"]

CMD ["--help"]
