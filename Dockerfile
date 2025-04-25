# Étape 1 : Builder (multi-stage pour réduire la taille)
FROM rust:1.86 as builder

# Installe les dépendances requises (ajuste si nécessaire)
RUN apt-get update && apt-get install -y pkg-config libssl-dev ffmpeg

# Crée un utilisateur non-root (bonnes pratiques)
RUN useradd -m rustuser
USER rustuser

WORKDIR /home/rustuser/app

# Copie le code
COPY --chown=rustuser:rustuser . .

# Compile en release
RUN cargo build --release

# Étape 2 : Runtime minimal
FROM debian:bullseye-slim

# Installe les libs runtime requises
RUN apt-get update && \
    apt-get install -y ffmpeg libssl1.1 ca-certificates && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*

# Crée un utilisateur sécurisé
RUN useradd -m appuser
USER appuser

# Crée le dossier d'exécution
WORKDIR /home/appuser

# Copie le binaire depuis le builder
COPY --from=builder /home/rustuser/app/target/release/rust-media-downloader .

# Définit le point d’entrée
ENTRYPOINT ["./rust-media-downloader"]
