# Guard-AI Backend - API Reference

Version: v1
Base URL: `http://localhost:8080/v1`

---

## Authentification

Tous les endpoints (sauf `/v1/health`) nécessitent une authentification par clé API.

**Header requis:**
```
X-API-Key: votre-cle-api
```

**Configuration:**
Définissez la variable d'environnement `API_KEY` lors du démarrage du serveur:
```bash
API_KEY="votre-cle-api-securisee" cargo run
```

**Codes d'erreur d'authentification:**

| Code | Description |
|------|-------------|
| 401 | Clé API manquante ou invalide |

**Note:** Si `API_KEY` n'est pas configurée, l'authentification est désactivée (mode développement uniquement).

---

## Table des matieres

- [Health Check](#health-check)
- [Events](#events)
  - [Ingestion batch](#post-v1events---ingestion-batch)
  - [Liste avec filtres](#get-v1events---liste-avec-filtres)
  - [Detail par ID](#get-v1eventsid---detail-par-id)
- [Statistics](#get-v1stats---statistiques)
- [Simulation](#post-v1simulate---simulation)
- [WebSocket](#ws-v1eventsstream---websocket-streaming)
- [Schemas](#schemas)

---

## Health Check

### GET /v1/health

Verifie que le serveur est operationnel.

**Response**

```json
{
  "status": "ok"
}
```

**Codes de reponse**

| Code | Description |
|------|-------------|
| 200 | Serveur operationnel |

**Exemple**

```bash
curl http://localhost:8080/v1/health
```

---

## Events

### POST /v1/events - Ingestion batch

Ingere un ou plusieurs evenements de securite.

**Headers**

| Header | Valeur |
|--------|--------|
| Content-Type | application/json |
| X-API-Key | votre-cle-api |

**Request Body**

```json
{
  "events": [
    {
      "event_id": "550e8400-e29b-41d4-a716-446655440001",
      "site_id": "550e8400-e29b-41d4-a716-446655440002",
      "hub_id": "hub-001",
      "source": "camera",
      "type": "motion_detected",
      "severity": "warning",
      "timestamp": "2025-01-29T10:30:00Z",
      "payload": {
        "zone": "entrance",
        "confidence": 0.95
      },
      "media_ref": "s3://bucket/video.mp4",
      "tags": ["outdoor", "night"],
      "schema_version": "v1"
    }
  ]
}
```

**Champs requis**

| Champ | Type | Description |
|-------|------|-------------|
| event_id | UUID | Identifiant unique de l'evenement |
| site_id | UUID | Identifiant du site |
| source | enum | `camera`, `sensor`, `network`, `system` |
| type | string | Type d'evenement (libre) |
| severity | enum | `info`, `warning`, `critical` |
| timestamp | ISO 8601 | Date/heure de l'evenement |
| payload | object | Donnees specifiques (libre) |
| schema_version | string | Doit etre `"v1"` |

**Champs optionnels**

| Champ | Type | Description |
|-------|------|-------------|
| hub_id | string | Identifiant du hub source |
| media_ref | string | Reference vers media associe |
| tags | array[string] | Tags pour categorisation |

**Response**

```json
{
  "accepted": 1,
  "rejected": 0
}
```

**Codes de reponse**

| Code | Description |
|------|-------------|
| 200 | Requete traitee (voir accepted/rejected) |

**Regles de validation**

- `schema_version` doit etre `"v1"`
- `type` ne peut pas etre vide

**Exemple**

```bash
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
```

---

### GET /v1/events - Liste avec filtres

Recupere la liste des evenements avec filtrage optionnel.

**Query Parameters**

| Parametre | Type | Description |
|-----------|------|-------------|
| site_id | UUID | Filtrer par site |
| source | enum | Filtrer par source (`camera`, `sensor`, `network`, `system`) |
| severity | enum | Filtrer par severite (`info`, `warning`, `critical`) |
| from | ISO 8601 | Date de debut |
| to | ISO 8601 | Date de fin |
| limit | integer | Nombre max de resultats (default: 100) |
| offset | integer | Pagination offset |

**Response**

```json
{
  "events": [
    {
      "event_id": "550e8400-e29b-41d4-a716-446655440001",
      "site_id": "550e8400-e29b-41d4-a716-446655440002",
      "hub_id": null,
      "source": "camera",
      "type": "motion_detected",
      "severity": "warning",
      "timestamp": "2025-01-29T10:30:00Z",
      "payload": {"zone": "entrance"},
      "media_ref": null,
      "tags": [],
      "schema_version": "v1"
    }
  ],
  "count": 1
}
```

**Codes de reponse**

| Code | Description |
|------|-------------|
| 200 | Succes |
| 500 | Erreur serveur |

**Exemples**

```bash
# Tous les evenements
curl -H "X-API-Key: votre-cle-api" http://localhost:8080/v1/events

# Filtrer par source
curl -H "X-API-Key: votre-cle-api" "http://localhost:8080/v1/events?source=camera"

# Filtrer par severite
curl -H "X-API-Key: votre-cle-api" "http://localhost:8080/v1/events?severity=critical"

# Combiner les filtres
curl -H "X-API-Key: votre-cle-api" "http://localhost:8080/v1/events?source=camera&severity=warning&limit=50"

# Pagination
curl -H "X-API-Key: votre-cle-api" "http://localhost:8080/v1/events?limit=10&offset=20"
```

---

### GET /v1/events/:id - Detail par ID

Recupere un evenement specifique par son UUID.

**Path Parameters**

| Parametre | Type | Description |
|-----------|------|-------------|
| id | UUID | Identifiant de l'evenement |

**Response**

```json
{
  "event": {
    "event_id": "550e8400-e29b-41d4-a716-446655440001",
    "site_id": "550e8400-e29b-41d4-a716-446655440002",
    "hub_id": null,
    "source": "camera",
    "type": "motion_detected",
    "severity": "warning",
    "timestamp": "2025-01-29T10:30:00Z",
    "payload": {"zone": "entrance"},
    "media_ref": null,
    "tags": [],
    "schema_version": "v1"
  }
}
```

**Codes de reponse**

| Code | Description |
|------|-------------|
| 200 | Evenement trouve |
| 404 | Evenement non trouve |
| 500 | Erreur serveur |

**Exemple**

```bash
curl -H "X-API-Key: votre-cle-api" http://localhost:8080/v1/events/550e8400-e29b-41d4-a716-446655440001
```

---

## GET /v1/stats - Statistiques

Recupere les statistiques agregees des evenements.

**Response**

```json
{
  "total_events": 1234,
  "by_severity": {
    "info": 800,
    "warning": 300,
    "critical": 134
  },
  "by_source": {
    "camera": 500,
    "sensor": 400,
    "network": 234,
    "system": 100
  },
  "last_24h": 156
}
```

**Codes de reponse**

| Code | Description |
|------|-------------|
| 200 | Succes |
| 500 | Erreur serveur |

**Exemple**

```bash
curl -H "X-API-Key: votre-cle-api" http://localhost:8080/v1/stats
```

---

## POST /v1/simulate - Simulation

Genere des evenements de test aleatoires.

**Headers**

| Header | Valeur |
|--------|--------|
| Content-Type | application/json |
| X-API-Key | votre-cle-api |

**Request Body**

```json
{
  "count": 10,
  "site_id": "550e8400-e29b-41d4-a716-446655440002"
}
```

| Champ | Type | Default | Description |
|-------|------|---------|-------------|
| count | integer | 1 | Nombre d'evenements (max: 100) |
| site_id | UUID | random | Site ID pour tous les evenements |

**Response**

```json
{
  "generated": 10,
  "persisted": 10,
  "broadcast": 10
}
```

**Codes de reponse**

| Code | Description |
|------|-------------|
| 200 | Succes |
| 500 | Erreur serveur |

**Exemple**

```bash
# Generer 5 evenements aleatoires
curl -X POST http://localhost:8080/v1/simulate \
  -H "Content-Type: application/json" \
  -H "X-API-Key: votre-cle-api" \
  -d '{"count": 5}'

# Generer pour un site specifique
curl -X POST http://localhost:8080/v1/simulate \
  -H "Content-Type: application/json" \
  -H "X-API-Key: votre-cle-api" \
  -d '{"count": 10, "site_id": "550e8400-e29b-41d4-a716-446655440002"}'
```

---

## WS /v1/events/stream - WebSocket Streaming

Connexion WebSocket pour recevoir les evenements en temps reel.

**URL**

```
ws://localhost:8080/v1/events/stream
```

**Authentification**

Le header `X-API-Key` doit être fourni lors de la connexion WebSocket.

**Comportement**

1. Client se connecte avec le header X-API-Key
2. Serveur envoie chaque nouvel evenement en JSON
3. Connexion reste ouverte jusqu'a deconnexion client

**Message format**

Chaque message est un evenement JSON complet :

```json
{
  "event_id": "f679d5de-a3b4-45cc-8c9a-542fa12d8885",
  "site_id": "8b831e72-1ad8-489b-b66a-152bac777fff",
  "hub_id": "hub-1",
  "source": "camera",
  "type": "motion_detected",
  "severity": "warning",
  "timestamp": "2025-01-29T15:21:41.349+00:00",
  "payload": {"zone": "entrance", "confidence": 82},
  "media_ref": null,
  "tags": ["outdoor"],
  "schema_version": "v1"
}
```

**Exemple avec websocat**

```bash
# Terminal 1 - Ecouter les evenements
websocat ws://localhost:8080/v1/events/stream --header="X-API-Key: votre-cle-api"

# Terminal 2 - Envoyer des evenements
curl -X POST http://localhost:8080/v1/simulate \
  -H "Content-Type: application/json" \
  -H "X-API-Key: votre-cle-api" \
  -d '{"count": 3}'
```

**Exemple JavaScript (Node.js avec ws library)**

```javascript
const WebSocket = require('ws');

const ws = new WebSocket('ws://localhost:8080/v1/events/stream', {
  headers: {
    'X-API-Key': 'votre-cle-api'
  }
});

ws.on('open', () => console.log('Connected'));
ws.on('message', (data) => {
  const event = JSON.parse(data);
  console.log('Event received:', event);
});
ws.on('close', () => console.log('Disconnected'));
```

**Note**: Les navigateurs web ne supportent pas les headers personnalisés pour les WebSockets. Pour une authentification depuis un navigateur, vous devrez soit:
- Passer l'API key en paramètre de requête: `ws://localhost:8080/v1/events/stream?api_key=votre-cle-api`
- Utiliser le header `Sec-WebSocket-Protocol` pour transmettre le token
- Implémenter une authentification basée sur les cookies/sessions

---

## Schemas

### EventV1

```json
{
  "event_id": "UUID",
  "site_id": "UUID",
  "hub_id": "string | null",
  "source": "camera | sensor | network | system",
  "type": "string",
  "severity": "info | warning | critical",
  "timestamp": "ISO 8601 datetime",
  "payload": "object",
  "media_ref": "string | null",
  "tags": "array[string]",
  "schema_version": "v1"
}
```

### EventSource (enum)

- `camera` - Evenement camera/video
- `sensor` - Capteur physique
- `network` - Evenement reseau
- `system` - Evenement systeme

### Severity (enum)

- `info` - Informatif
- `warning` - Avertissement
- `critical` - Critique, action requise
