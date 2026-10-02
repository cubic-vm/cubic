FROM rust:1.92-slim
WORKDIR /usr/local/app

COPY . .

ENV DEBIAN_FRONTEND=noninteractive
ENV XDG_DATA_HOME=/tmp/data
ENV XDG_CACHE_HOME=/tmp/cache
RUN apt update && \
    apt install -y \
        qemu-utils \
        qemu-system-x86 \
        qemu-system-arm \
        python3-sphinx \
        python3-sphinx-rtd-theme \
        git \
        vim \
        yamllint \
        shellcheck &&\
    git config --global --add safe.directory /usr/local/app
RUN rustup component add clippy rustfmt && \
    cargo install --locked cargo-audit &&\
    echo 'alias cubic="cargo run"' >> ~/.bashrc
