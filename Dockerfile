FROM rust:1.86.0
WORKDIR /usr/local/app

COPY . .

ENV DEBIAN_FRONTEND=noninteractive
ENV XDG_RUNTIME_DIR=/tmp
RUN apt update && \
    apt install -y qemu-utils cloud-utils qemu-system-x86 qemu-system-arm
RUN rustup component add clippy rustfmt && \
    cargo install --locked cargo-audit@0.21.1 &&\
    cargo fetch
