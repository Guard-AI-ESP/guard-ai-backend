# Guard-AI Backend

Backend Rust pour le systeme de securite local Guard-AI. Gere l'ingestion, le stockage et la diffusion en temps reel des evenements de securite.

## Fonctionnalites

- **Ingestion d'evenements** - Reception et validation d'evenements de securite en batch
- **Persistance SQLite** - Stockage local sans serveur externe
- **Streaming temps reel** - WebSocket pour diffusion instantanee aux clients
- **Statistiques** - Metriques agregees par source, severite, periode
- **Simulation** - Generation d'evenements de test pour le developpement

## Prerequisites

- Rust 1.75+
- SQLite 3 (inclus automatiquement via SQLx)

## Installation

```bash
git clone https://github.com/Guard-AI-ESP/guard-ai-backend.git
cd guard-ai-backend
cargo build --release
```

## Lancement

```bash
# Lancement par defaut (base: guard-ai.db, port: 8080)
cargo run

# Avec configuration custom
DATABASE_URL="sqlite://custom.db" PORT=3000 cargo run
```

## Variables d'environnement

| Variable | Default | Description |
|----------|---------|-------------|
| `DATABASE_URL` | `sqlite://guard-ai.db` | Chemin vers la base SQLite |
| `PORT` | `8080` | Port du serveur HTTP |
| `API_KEY` | (none) | Clé d'API pour authentification des endpoints |
| `RUST_LOG` | `info` | Niveau de log (debug, info, warn, error) |

**Note de sécurité**: Il est fortement recommandé de définir `API_KEY` en production pour protéger les endpoints sensibles. Sans cette variable, l'authentification est désactivée (mode développement uniquement).

## API Endpoints

Tous les endpoints (sauf `/v1/health`) requièrent une authentification par clé API via le header `X-API-Key`.

| Methode | Endpoint | Description | Auth requise |
|---------|----------|-------------|--------------|
| GET | `/v1/health` | Health check | Non |
| POST | `/v1/events` | Ingestion batch d'evenements | Oui |
| GET | `/v1/events` | Liste avec filtres (site_id, source, severity, from, to) | Oui |
| GET | `/v1/events/:id` | Detail d'un evenement par UUID | Oui |
| GET | `/v1/stats` | Statistiques agregees | Oui |
| POST | `/v1/simulate` | Generer des evenements de test | Oui |
| WS | `/v1/events/stream` | WebSocket streaming temps reel | Oui |

Voir [docs/API.md](docs/API.md) pour la documentation complete.

## Exemples rapides

```bash
# Health check (pas d'authentification requise)
curl http://localhost:8080/v1/health

# Ingerer un evenement (authentification requise)
curl -X POST http://localhost:8080/v1/events \
  -H "Content-Type: application/json" \
  -H "X-API-Key: votre-cle-api" \
  -d '{
    "events": [{
      "event_id": "550e8400-e29b-41d4-a716-446655440001",
      "site_id": "550e8400-e29b-41d4-a716-446655440002",
      "source": "camera",
      "type": "motion_detected",
      "severity": "warning",
      "timestamp": "2025-01-29T10:30:00Z",
      "payload": {"zone": "entrance"},
      "tags": [],
      "schema_version": "v1"
    }]
  }'

# Lister les evenements
curl -H "X-API-Key: votre-cle-api" http://localhost:8080/v1/events

# Filtrer par source
curl -H "X-API-Key: votre-cle-api" "http://localhost:8080/v1/events?source=camera&severity=warning"

# Statistiques
curl -H "X-API-Key: votre-cle-api" http://localhost:8080/v1/stats

# Simuler 10 evenements
curl -X POST http://localhost:8080/v1/simulate \
  -H "Content-Type: application/json" \
  -H "X-API-Key: votre-cle-api" \
  -d '{"count": 10}'

# WebSocket (necessite websocat)
websocat ws://localhost:8080/v1/events/stream --header="X-API-Key: votre-cle-api"
```

## Tests

```bash
# Lancer tous les tests
cargo test

# Tests avec output verbose
cargo test -- --nocapture
```

## Architecture

```
guard-ai-backend/
├── src/
│   ├── main.rs           # Point d'entree
│   ├── lib.rs            # Exports publics
│   ├── app.rs            # Configuration Axum router
│   ├── state.rs          # AppState partage (DB + broadcast)
│   ├── simulator.rs      # Generateur d'evenements test
│   ├── db/
│   │   ├── pool.rs       # Pool SQLite
│   │   └── repository.rs # CRUD evenements
│   ├── models/
│   │   └── event.rs      # Structures EventV1, etc.
│   └── routes/
│       ├── events.rs     # POST/GET /events
│       ├── health.rs     # GET /health
│       ├── stats.rs      # GET /stats
│       ├── simulate.rs   # POST /simulate
│       └── ws.rs         # WebSocket /events/stream
├── migrations/           # Schemas SQL
└── tests/               # Tests d'integration
```

## Stack technique

- **Framework** : Axum 0.7
- **Runtime** : Tokio
- **Database** : SQLite via SQLx 0.8
- **Serialization** : Serde + serde_json
- **Logging** : tracing + tracing-subscriber

## License

MIT
