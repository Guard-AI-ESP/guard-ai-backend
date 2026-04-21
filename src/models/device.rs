use serde::{Deserialize, Serialize};

/// Appareil connecté au hub réseau, vu via hostapd / dnsmasq
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Device {
    pub mac_address: String,
    pub hub_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ip_address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hostname: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rssi: Option<i32>,
    pub first_seen_at: String,
    pub last_seen_at: String,
    pub connected: bool,
}

/// Entrée d'un snapshot device poussé par le hub via PUT /v1/devices
#[derive(Debug, Clone, Deserialize)]
pub struct DeviceSnapshotEntry {
    pub mac_address: String,
    #[serde(default)]
    pub ip_address: Option<String>,
    #[serde(default)]
    pub hostname: Option<String>,
    #[serde(default)]
    pub rssi: Option<i32>,
}

/// Corps de PUT /v1/devices — le hub remplace intégralement l'état "connecté" pour son `hub_id`
#[derive(Debug, Clone, Deserialize)]
pub struct DeviceSnapshotRequest {
    pub hub_id: String,
    pub devices: Vec<DeviceSnapshotEntry>,
}

/// Réponse de PUT /v1/devices
#[derive(Debug, Clone, Serialize)]
pub struct DeviceSnapshotResponse {
    /// Nombre de devices actuellement marqués connectés après le snapshot
    pub connected_count: usize,
}

/// Réponse de GET /v1/devices
#[derive(Debug, Clone, Serialize)]
pub struct DevicesListResponse {
    pub devices: Vec<Device>,
    pub count: usize,
}
