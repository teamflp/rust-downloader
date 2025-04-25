# Étape 1 : Builder
FROM rust:1.86 AS builder

WORKDIR /usr/src/app

# Installer les dépendances système nécessaires à la compilation
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

# Copier les fichiers Cargo pour mise en cache des dépendances
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {println!(\"Build dependencies...\");}" > src/main.rs
RUN cargo build --release
RUN rm -rf src

# Copier le code source complet
COPY . .

# Compiler le binaire final
RUN cargo build --release

# Étape 2 : Image minimale pour exécution
FROM debian:bullseye-slim

# Installer les dépendances nécessaires à l'exécution
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libcurl4 \
 && apt-get clean \
 && rm -rf /var/lib/apt/lists/*

# Copier le binaire depuis le builder
COPY --from=builder /usr/src/app/target/release/rust-media-downloader /usr/local/bin/rust-media-downloader

# Exposer le point d’entrée
ENTRYPOINT ["rust-media-downloader"]
