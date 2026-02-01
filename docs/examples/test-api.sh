#!/bin/bash

# Guard-AI Backend - Script de test API
# Usage: ./test-api.sh [base_url]
# Environment: API_KEY=your-api-key (optionnel, requis si le serveur a l'authentification activee)

BASE_URL="${1:-http://localhost:8080}"
API_KEY="${API_KEY:-}"

echo "=========================================="
echo "Guard-AI Backend API Test"
echo "Base URL: $BASE_URL"
if [ -n "$API_KEY" ]; then
    echo "Authentication: Enabled"
else
    echo "Authentication: Disabled (development mode)"
fi
echo "=========================================="
echo ""

# Couleurs
GREEN='\033[0;32m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

success() {
    echo -e "${GREEN}[OK]${NC} $1"
}

fail() {
    echo -e "${RED}[FAIL]${NC} $1"
}

info() {
    echo -e "${BLUE}[TEST]${NC} $1"
}

# Fonction pour construire les arguments curl avec authentification si necessaire
build_curl_args() {
    if [ -n "$API_KEY" ]; then
        echo "-H \"X-API-Key: $API_KEY\""
    fi
}

# 1. Health Check
info "Health Check"
RESPONSE=$(curl -s -w "\n%{http_code}" "$BASE_URL/v1/health")
HTTP_CODE=$(echo "$RESPONSE" | tail -n1)
BODY=$(echo "$RESPONSE" | head -n-1)

if [ "$HTTP_CODE" = "200" ]; then
    success "GET /v1/health - $HTTP_CODE"
    echo "  Response: $BODY"
else
    fail "GET /v1/health - $HTTP_CODE"
fi
echo ""

# 2. Ingestion d'evenements
info "Ingestion d'evenements"
EVENT_ID="test-$(date +%s)-001"
SITE_ID="aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa"

PAYLOAD=$(cat <<EOF
{
  "events": [{
    "event_id": "$EVENT_ID",
    "site_id": "$SITE_ID",
    "source": "camera",
    "type": "motion_detected",
    "severity": "warning",
    "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
    "payload": {"zone": "test", "script": true},
    "tags": ["test"],
    "schema_version": "v1"
  }]
}
EOF
)

if [ -n "$API_KEY" ]; then
    RESPONSE=$(curl -s -w "\n%{http_code}" -X POST "$BASE_URL/v1/events" \
      -H "Content-Type: application/json" \
      -H "X-API-Key: $API_KEY" \
      -d "$PAYLOAD")
else
    RESPONSE=$(curl -s -w "\n%{http_code}" -X POST "$BASE_URL/v1/events" \
      -H "Content-Type: application/json" \
      -d "$PAYLOAD")
fi
HTTP_CODE=$(echo "$RESPONSE" | tail -n1)
BODY=$(echo "$RESPONSE" | head -n-1)

if [ "$HTTP_CODE" = "200" ]; then
    success "POST /v1/events - $HTTP_CODE"
    echo "  Response: $BODY"
else
    fail "POST /v1/events - $HTTP_CODE"
fi
echo ""

# 3. Liste des evenements
info "Liste des evenements"
if [ -n "$API_KEY" ]; then
    RESPONSE=$(curl -s -w "\n%{http_code}" -H "X-API-Key: $API_KEY" "$BASE_URL/v1/events?limit=5")
else
    RESPONSE=$(curl -s -w "\n%{http_code}" "$BASE_URL/v1/events?limit=5")
fi
HTTP_CODE=$(echo "$RESPONSE" | tail -n1)
BODY=$(echo "$RESPONSE" | head -n-1)

if [ "$HTTP_CODE" = "200" ]; then
    success "GET /v1/events - $HTTP_CODE"
    COUNT=$(echo "$BODY" | grep -o '"count":[0-9]*' | cut -d: -f2)
    echo "  Count: $COUNT"
else
    fail "GET /v1/events - $HTTP_CODE"
fi
echo ""

# 4. Filtrage par source
info "Filtrage par source"
if [ -n "$API_KEY" ]; then
    RESPONSE=$(curl -s -w "\n%{http_code}" -H "X-API-Key: $API_KEY" "$BASE_URL/v1/events?source=camera")
else
    RESPONSE=$(curl -s -w "\n%{http_code}" "$BASE_URL/v1/events?source=camera")
fi
HTTP_CODE=$(echo "$RESPONSE" | tail -n1)

if [ "$HTTP_CODE" = "200" ]; then
    success "GET /v1/events?source=camera - $HTTP_CODE"
else
    fail "GET /v1/events?source=camera - $HTTP_CODE"
fi
echo ""

# 5. Evenement par ID (404 attendu si ID inexistant)
info "Evenement par ID"
if [ -n "$API_KEY" ]; then
    RESPONSE=$(curl -s -w "\n%{http_code}" -H "X-API-Key: $API_KEY" "$BASE_URL/v1/events/00000000-0000-0000-0000-000000000000")
else
    RESPONSE=$(curl -s -w "\n%{http_code}" "$BASE_URL/v1/events/00000000-0000-0000-0000-000000000000")
fi
HTTP_CODE=$(echo "$RESPONSE" | tail -n1)

if [ "$HTTP_CODE" = "404" ]; then
    success "GET /v1/events/:id (not found) - $HTTP_CODE"
else
    fail "GET /v1/events/:id - Expected 404, got $HTTP_CODE"
fi
echo ""

# 6. Statistiques
info "Statistiques"
if [ -n "$API_KEY" ]; then
    RESPONSE=$(curl -s -w "\n%{http_code}" -H "X-API-Key: $API_KEY" "$BASE_URL/v1/stats")
else
    RESPONSE=$(curl -s -w "\n%{http_code}" "$BASE_URL/v1/stats")
fi
HTTP_CODE=$(echo "$RESPONSE" | tail -n1)
BODY=$(echo "$RESPONSE" | head -n-1)

if [ "$HTTP_CODE" = "200" ]; then
    success "GET /v1/stats - $HTTP_CODE"
    TOTAL=$(echo "$BODY" | grep -o '"total_events":[0-9]*' | cut -d: -f2)
    echo "  Total events: $TOTAL"
else
    fail "GET /v1/stats - $HTTP_CODE"
fi
echo ""

# 7. Simulation
info "Simulation d'evenements"
if [ -n "$API_KEY" ]; then
    RESPONSE=$(curl -s -w "\n%{http_code}" -X POST "$BASE_URL/v1/simulate" \
      -H "Content-Type: application/json" \
      -H "X-API-Key: $API_KEY" \
      -d '{"count": 3}')
else
    RESPONSE=$(curl -s -w "\n%{http_code}" -X POST "$BASE_URL/v1/simulate" \
      -H "Content-Type: application/json" \
      -d '{"count": 3}')
fi
HTTP_CODE=$(echo "$RESPONSE" | tail -n1)
BODY=$(echo "$RESPONSE" | head -n-1)

if [ "$HTTP_CODE" = "200" ]; then
    success "POST /v1/simulate - $HTTP_CODE"
    echo "  Response: $BODY"
else
    fail "POST /v1/simulate - $HTTP_CODE"
fi
echo ""

# 8. Verification post-simulation
info "Verification post-simulation"
if [ -n "$API_KEY" ]; then
    RESPONSE=$(curl -s -w "\n%{http_code}" -H "X-API-Key: $API_KEY" "$BASE_URL/v1/stats")
else
    RESPONSE=$(curl -s -w "\n%{http_code}" "$BASE_URL/v1/stats")
fi
HTTP_CODE=$(echo "$RESPONSE" | tail -n1)
BODY=$(echo "$RESPONSE" | head -n-1)

if [ "$HTTP_CODE" = "200" ]; then
    TOTAL=$(echo "$BODY" | grep -o '"total_events":[0-9]*' | cut -d: -f2)
    success "Stats after simulation - Total: $TOTAL"
else
    fail "GET /v1/stats - $HTTP_CODE"
fi
echo ""

echo "=========================================="
echo "Tests termines"
echo "=========================================="
