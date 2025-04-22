# Étape 1 : Builder
FROM rust:1.86.0 AS builder

WORKDIR /usr/src/app

# Installer les dépendances système nécessaires
RUN apt-get update && apt-get install -y --no-install-recommends \
    curl \
    git \
    build-essential \
    clang \
    lld \
    libcurl4-openssl-dev \
    pkg-config \
 && apt-get clean \
 && rm -rf /var/lib/apt/lists/*

# Copier uniquement Cargo.toml et Cargo.lock pour installer les dépendances en cache
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {println!(\"Build dependencies...\");}" > src/main.rs
RUN cargo build --release
RUN rm -rf src

# Copier le reste du code
COPY . .

# Compiler le binaire final
RUN cargo build --release

# Étape 2 : Image finale légère
FROM debian:bullseye-slim

# Installer les dépendances runtime si besoin (souvent libcurl pour reqwest)
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libcurl4 \
 && apt-get clean \
 && rm -rf /var/lib/apt/lists/*

# Copier le binaire compilé
COPY --from=builder /usr/src/app/target/release/rust-downloader /usr/local/bin/rust-downloader

# Définir le point d'entrée
ENTRYPOINT ["rust-downloader"]
