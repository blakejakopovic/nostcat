FROM rust:1 AS chef
ENV CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse
# We only pay the installation cost once,
# it will be cached from the second build onwards
RUN cargo install cargo-chef
WORKDIR app

FROM chef AS planner
COPY . .
RUN cargo chef prepare  --recipe-path recipe.json

FROM chef AS builder
ENV CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json
# Build application
COPY . .
# We need to make openssl use vendored ssl to work for alpine
RUN echo 'openssl = { version = "0.10", features = ["vendored"] }' >> /app/Cargo.toml
RUN cargo build --release
RUN strip /app/target/release/nostcat

# We do not need the Rust toolchain to run the binary!
FROM alpine:3.14 AS runtime
RUN apk add openssl openssl-dev
# We need to instal glibc or we cannot run the executable
ENV GLIBC_REPO=https://github.com/sgerrand/alpine-pkg-glibc
ENV GLIBC_VERSION=2.30-r0
RUN set -ex && \
    apk --update add libstdc++ curl ca-certificates && \
    for pkg in glibc-${GLIBC_VERSION} glibc-bin-${GLIBC_VERSION}; \
        do curl -sSL ${GLIBC_REPO}/releases/download/${GLIBC_VERSION}/${pkg}.apk -o /tmp/${pkg}.apk; done && \
    apk add --allow-untrusted /tmp/*.apk && \
    rm -v /tmp/*.apk && \
    /usr/glibc-compat/sbin/ldconfig /lib /usr/glibc-compat/lib
# Purge some unneeded cache
RUN apk del libstdc++ curl
RUN rm -rf /var/lib/apt/lists/*
WORKDIR app
COPY --from=builder /app/target/release/nostcat /usr/local/bin
ENTRYPOINT ["/usr/local/bin/nostcat"]
