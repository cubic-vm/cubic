FROM rust:1.75.0
WORKDIR /usr/local/app

ENV DEBIAN_FRONTEND=noninteractive
RUN rustup component add clippy rustfmt && \
    cargo install --locked cargo-audit
