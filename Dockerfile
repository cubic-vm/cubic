FROM rust:1.86.0
WORKDIR /usr/local/app

COPY . .

ENV DEBIAN_FRONTEND=noninteractive
ENV XDG_RUNTIME_DIR=/tmp
RUN rustup component add clippy rustfmt && \
    cargo install --locked cargo-audit@0.21.1 &&\
    cargo fetch
