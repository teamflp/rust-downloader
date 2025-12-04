# Rust Media Downloader - Web Application

Application web professionnelle pour télécharger des vidéos et audios depuis diverses plateformes.

## 🏗️ Architecture

Le projet est organisé en workspace Rust avec 3 packages :

- **`backend/`** : API REST avec Axum (port 8080)
- **`frontend/`** : Interface React moderne (port 5173 en dev, 3000 en prod)
- **`shared/`** : Bibliothèque partagée avec la logique métier
- **`cli/`** : Version ligne de commande (toujours fonctionnelle)

## 🚀 Démarrage Rapide

### Prérequis

- **Rust** 1.75+ ([installer](https://rustup.rs/))
- **Node.js** 20+ ([installer](https://nodejs.org/))
- **yt-dlp** ([installer](https://github.com/yt-dlp/yt-dlp))
- **ffmpeg** ([installer](https://ffmpeg.org/))
- **Spleeter** (optionnel, pour extraction instrumentale)

### Développement

#### 1. Démarrer le Backend

```bash
cd backend
cargo run
```

Le serveur API démarre sur `http://localhost:8080`

#### 2. Démarrer le Frontend

Dans un nouveau terminal :

```bash
cd frontend
npm install
npm run dev
```

L'interface web s'ouvre sur `http://localhost:5173`

### Production avec Docker

La méthode la plus simple pour déployer l'application complète :

```bash
docker-compose up --build
```

Accédez à l'application sur `http://localhost:3000`

## ✨ Fonctionnalités

### Interface Web

- 🎨 **Design moderne** avec glassmorphism et animations fluides
- 🌓 **Mode sombre/clair** avec transition douce
- 📱 **Responsive** : fonctionne sur mobile, tablette et desktop
- ⚡ **Temps réel** : suivi de progression en direct
- 🎯 **Filtres intelligents** : tous, en cours, terminés, échoués

### Téléchargements

- 🎥 **Vidéo** : MP4, WebM, MKV avec choix de qualité
- 🎵 **Audio** : MP3, WAV, M4A, FLAC
- 🎹 **Instrumental** : extraction IA avec Spleeter
- 🍪 **Cookies** : support pour contenu restreint

## 📡 API Endpoints

### Downloads

```
POST   /api/downloads          # Créer un téléchargement
GET    /api/downloads          # Liste tous les téléchargements
GET    /api/downloads/:id      # Détails d'un téléchargement
DELETE /api/downloads/:id      # Supprimer un téléchargement
GET    /health                 # Health check
```

### Exemple de requête

```bash
curl -X POST http://localhost:8080/api/downloads \
  -H "Content-Type: application/json" \
  -d '{
    "url": "https://youtube.com/watch?v=...",
    "type": "video",
    "format": "mp4"
  }'
```

## 🎨 Design System

Le frontend utilise un design system complet avec :

- **Palette de couleurs** : thème sombre/clair avec accents vibrants
- **Typographie** : Inter font avec hiérarchie claire
- **Composants** : boutons, cartes, formulaires, badges
- **Animations** : Framer Motion pour transitions fluides
- **Effets** : glassmorphism, gradients, ombres portées

## 📁 Structure du Projet

```
rust-downloader-cli/
├── backend/              # API Rust (Axum)
│   ├── src/
│   │   ├── main.rs      # Serveur principal
│   │   ├── api/         # Routes API
│   │   ├── models.rs    # Structures de données
│   │   └── state.rs     # Gestion d'état
│   └── Cargo.toml
├── frontend/             # Interface React
│   ├── src/
│   │   ├── components/  # Composants React
│   │   ├── hooks/       # Hooks personnalisés
│   │   ├── api/         # Client API
│   │   ├── styles/      # CSS global
│   │   ├── App.jsx      # Composant principal
│   │   └── main.jsx     # Point d'entrée
│   ├── index.html
│   ├── package.json
│   └── vite.config.js
├── shared/               # Bibliothèque partagée
│   ├── src/
│   │   ├── lib.rs       # Exports
│   │   ├── downloader.rs
│   │   ├── spleeter.rs
│   │   └── ...
│   └── Cargo.toml
├── cli/                  # CLI (version originale)
│   ├── src/
│   │   └── main.rs
│   └── Cargo.toml
├── Dockerfile.backend    # Build backend
├── Dockerfile.frontend   # Build frontend
├── docker-compose.yml    # Orchestration
└── Cargo.toml           # Workspace
```

## 🔧 Configuration

### Variables d'Environnement

#### Backend

```bash
RUST_LOG=debug              # Niveau de log
```

#### Frontend

```bash
VITE_API_URL=http://localhost:8080  # URL de l'API
```

## 🧪 Tests

### Backend

```bash
cd backend
cargo test
```

### Frontend

```bash
cd frontend
npm test
```

## 📦 Build de Production

### Backend

```bash
cd backend
cargo build --release
./target/release/rust-media-downloader-backend
```

### Frontend

```bash
cd frontend
npm run build
npm run preview
```

## 🐳 Déploiement Docker

### Build des images

```bash
# Backend
docker build -f Dockerfile.backend -t rmd-backend .

# Frontend
docker build -f Dockerfile.frontend -t rmd-frontend .
```

### Lancer avec Docker Compose

```bash
docker-compose up -d
```

L'application sera accessible sur :
- Frontend : `http://localhost:3000`
- Backend API : `http://localhost:8080`

## 🛠️ Développement

### Ajouter une nouvelle fonctionnalité

1. **Backend** : Ajouter un endpoint dans `backend/src/api/`
2. **Frontend** : Créer un composant dans `frontend/src/components/`
3. **Shared** : Ajouter la logique métier dans `shared/src/`

### Conventions de code

- **Rust** : `cargo fmt` et `cargo clippy`
- **JavaScript** : ESLint et Prettier (à configurer)
- **Commits** : Messages clairs et descriptifs

## 🤝 Contribution

Les contributions sont bienvenues ! Consultez le [README principal](../README.md) pour les guidelines.

## 📄 Licence

MIT - Voir le fichier LICENSE

## 👤 Auteur

[Paterne G. G](https://github.com/teamflp)

---

**Note** : La version CLI reste disponible et fonctionnelle dans le dossier `cli/`.
