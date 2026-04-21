use serde::{Deserialize, Serialize};

/// Hub réseau (Raspberry Pi exécutant hostapd + dnsmasq + suricata + iptables)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hub {
    pub id: String,
    pub site_id: String,
    pub name: String,
    /// Timestamp ISO-8601 UTC du dernier heartbeat (None si jamais connecté)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_seen_at: Option<String>,
    pub created_at: String,
}

/// Corps de POST /v1/hubs — enregistre un nouveau hub
#[derive(Debug, Clone, Deserialize)]
pub struct CreateHubRequest {
    pub id: String,
    pub site_id: String,
    pub name: String,
    /// Clé API M2M en clair, hashée côté serveur avant persistance
    pub api_key: String,
}

/// Réponse de POST /v1/hubs
#[derive(Debug, Clone, Serialize)]
pub struct HubResponse {
    pub hub: Hub,
}

/// Réponse de GET /v1/hubs
#[derive(Debug, Clone, Serialize)]
pub struct HubsListResponse {
    pub hubs: Vec<Hub>,
    pub count: usize,
}

/// Corps de POST /v1/hubs/:id/heartbeat (optionnel, le hub peut n'envoyer que le timestamp)
#[derive(Debug, Clone, Deserialize, Default)]
pub struct HeartbeatRequest {
    /// Infos diagnostiques facultatives (version agent, uptime, etc.)
    #[serde(default)]
    pub diagnostics: Option<serde_json::Value>,
}
