FROM rust:1.86-alpine AS builder

# Installer curl et outils de build
RUN apk update && apk add --no-cache \
    curl \
    git \
    build-base \
    clang \
    lld \
    musl-dev \
    pkgconfig \
    mingw-w64-gcc \
    mingw-w64-crt \
    mingw-w64-headers \
    mingw-w64-winpthreads \
    cabextract \
    unzip

# Installer Rustup (optionnel si tu veux une version différente de celle de l’image)
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

WORKDIR /workspace
COPY . /workspace

RUN rustup target add x86_64-unknown-linux-gnu \
    && rustup target add x86_64-pc-windows-gnu \
    && rustup target add x86_64-apple-darwin

CMD ["cargo", "build", "--release", "--target", "x86_64-unknown-linux-gnu"]
