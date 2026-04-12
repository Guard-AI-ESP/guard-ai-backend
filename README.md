# Guard AI — Backend

API REST + WebSocket pour le système de surveillance Guard AI.

**Stack** : Rust · Axum 0.7 · SQLx · SQLite · JWT · WebSocket

---

## Démarrage rapide

```bash
git clone https://github.com/Guard-AI-ESP/guard-ai-backend.git
cd guard-ai-backend
git checkout staging

cargo build
cargo run
# → http://localhost:8080
```

> Les migrations SQLite s'exécutent automatiquement au démarrage.

---

## Variables d'environnement

| Variable | Défaut | Description |
|---|---|---|
| `DATABASE_URL` | `sqlite://guard-ai.db` | Chemin de la base SQLite |
| `PORT` | `8080` | Port d'écoute |
| `JWT_SECRET` | `dev-insecure-secret-...` | Clé de signature JWT — **changer en prod** |
| `API_KEY` | _(désactivé)_ | Clé M2M pour accès IoT/services sans JWT |

```bash
# Exemple .env de développement
DATABASE_URL=sqlite://guard-ai.db
JWT_SECRET=mon-secret-local
```

---

## Endpoints

### Publics (sans auth)

| Méthode | Route | Description |
|---|---|---|
| `GET` | `/health` | Santé du serveur |
| `POST` | `/v1/auth/register` | Créer un compte |
| `POST` | `/v1/auth/login` | Obtenir un JWT |

### Protégés (JWT requis)

Ajouter `Authorization: Bearer <token>` à chaque requête.

| Méthode | Route | Description |
|---|---|---|
| `GET` | `/v1/events` | Liste des événements (filtres disponibles) |
| `POST` | `/v1/events` | Ingérer des événements |
| `GET` | `/v1/events/:id` | Détail d'un événement |
| `GET` | `/v1/stats` | Statistiques globales |
| `POST` | `/v1/simulate` | Générer des événements fictifs (max 100) |
| `GET` | `/v1/persons` | Liste des personnes connues (avec embeddings) |
| `POST` | `/v1/persons` | Enregistrer une personne |
| `DELETE` | `/v1/persons/:id` | Supprimer une personne |

### WebSocket

| Route | Description |
|---|---|
| `ws://localhost:8080/ws?token=<jwt>` | Stream temps réel des nouveaux événements |

---

## Schéma de la base de données

### Table `events`

| Colonne | Type | Description |
|---|---|---|
| `event_id` | TEXT (UUID) | Identifiant unique |
| `site_id` | TEXT (UUID) | Site de provenance |
| `source` | TEXT | `camera` · `sensor` · `network` · `system` |
| `event_type` | TEXT | Ex: `face_recognized`, `door_opened` |
| `severity` | TEXT | `info` · `warning` · `critical` |
| `timestamp` | TEXT | ISO-8601 UTC |
| `camera_id` | TEXT? | Identifiant caméra (events caméra uniquement) |
| `face_id` | TEXT? | UUID personne reconnue |
| `person_name` | TEXT? | Nom dénormalisé |
| `confidence` | REAL? | Score 0.0–1.0 |
| `is_known` | INTEGER? | 1 = connu, 0 = inconnu/intrus |
| `bounding_box` | TEXT? | JSON `{x, y, w, h}` normalisé 0.0–1.0 |

### Table `persons`

| Colonne | Type | Description |
|---|---|---|
| `id` | TEXT (UUID) | Identifiant unique |
| `name` | TEXT | Nom complet |
| `embedding` | TEXT | JSON array 512 floats (FaceNet) |
| `photo_url` | TEXT? | URL ou chemin local |
| `created_at` | TEXT | ISO-8601 UTC |

### Table `users`

| Colonne | Type | Description |
|---|---|---|
| `id` | TEXT (UUID) | Identifiant |
| `email` | TEXT (unique) | Email de connexion |
| `password_hash` | TEXT | Bcrypt |

---

## Exemple d'utilisation rapide

```bash
# 1. Créer un compte
curl -X POST http://localhost:8080/v1/auth/register \
  -H "Content-Type: application/json" \
  -d '{"email": "admin@guard-ai.com", "password": "secret123"}'

# 2. Se connecter → récupérer le token
TOKEN=$(curl -s -X POST http://localhost:8080/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email": "admin@guard-ai.com", "password": "secret123"}' \
  | jq -r '.token')

# 3. Générer des events de démo
curl -X POST http://localhost:8080/v1/simulate \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"count": 10}'

# 4. Lister les events
curl http://localhost:8080/v1/events \
  -H "Authorization: Bearer $TOKEN"
```

---

## Schéma d'un événement (POST /v1/events)

```json
{
  "events": [{
    "event_id": "<uuid-v4>",
    "site_id": "00000000-0000-0000-0000-000000000001",
    "hub_id": "hub-iot-01",
    "source": "sensor",
    "type": "door_opened",
    "severity": "info",
    "timestamp": "2026-04-12T10:00:00Z",
    "payload": {},
    "tags": ["iot"],
    "schema_version": "v1"
  }]
}
```

Champs optionnels pour les events caméra :
```json
{
  "camera_id": "cam-entree-01",
  "is_known": false,
  "confidence": 0.92,
  "bounding_box": { "x": 0.2, "y": 0.1, "w": 0.15, "h": 0.2 }
}
```

---

## Tests

```bash
cargo test                    # tous les tests
cargo test persons            # tests du module persons uniquement
cargo test -- --nocapture     # avec les logs
```

36 tests d'intégration couvrent tous les handlers HTTP (happy path + cas d'erreur).

---

## Branches actives

| Branche | Rôle |
|---|---|
| `staging` | Intégration — état stable, base de travail |
| `main` | Production — ne pas modifier directement |
| `feat/backend/*` | Fonctionnalités en cours |

> Toujours créer sa branche depuis `staging`, PR vers `staging`.
