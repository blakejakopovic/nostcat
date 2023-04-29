FROM rust:alpine

RUN apk update && \
    apk upgrade && \
    apk add --no-cache pkgconfig openssl-dev musl-dev

WORKDIR /usr/src/myapp
COPY . .

RUN cargo install --path .

ENTRYPOINT ["/usr/local/cargo/bin/nostcat"]

CMD ["--help"]
