use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Types de commandes acceptés par un hub (voir guard-ai-contracts/commands/v1)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CommandType {
    ScanNetwork,
    BlockDevice,
    KickDevice,
    UnblockDevice,
}

impl CommandType {
    pub fn as_str(&self) -> &'static str {
        match self {
            CommandType::ScanNetwork => "scan_network",
            CommandType::BlockDevice => "block_device",
            CommandType::KickDevice => "kick_device",
            CommandType::UnblockDevice => "unblock_device",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "scan_network" => Some(CommandType::ScanNetwork),
            "block_device" => Some(CommandType::BlockDevice),
            "kick_device" => Some(CommandType::KickDevice),
            "unblock_device" => Some(CommandType::UnblockDevice),
            _ => None,
        }
    }
}

/// Cycle de vie d'une commande
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CommandStatus {
    Pending,
    Dispatched,
    Running,
    Succeeded,
    Failed,
}

impl CommandStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            CommandStatus::Pending => "pending",
            CommandStatus::Dispatched => "dispatched",
            CommandStatus::Running => "running",
            CommandStatus::Succeeded => "succeeded",
            CommandStatus::Failed => "failed",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "pending" => Some(CommandStatus::Pending),
            "dispatched" => Some(CommandStatus::Dispatched),
            "running" => Some(CommandStatus::Running),
            "succeeded" => Some(CommandStatus::Succeeded),
            "failed" => Some(CommandStatus::Failed),
            _ => None,
        }
    }
}

/// Commande envoyée à un hub
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Command {
    pub id: Uuid,
    pub hub_id: String,
    #[serde(rename = "type")]
    pub command_type: CommandType,
    pub payload: serde_json::Value,
    pub status: CommandStatus,
    pub created_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dispatched_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Corps de POST /v1/hubs/:id/commands
#[derive(Debug, Clone, Deserialize)]
pub struct CreateCommandRequest {
    #[serde(rename = "type")]
    pub command_type: CommandType,
    #[serde(default)]
    pub payload: serde_json::Value,
    /// TTL en secondes — si absent, pas d'expiration (commandes persistantes type block_device permanent)
    #[serde(default)]
    pub ttl_seconds: Option<i64>,
}

/// Corps de PATCH /v1/hubs/:id/commands/:cmd_id — le hub met à jour l'état d'exécution
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateCommandRequest {
    pub status: CommandStatus,
    #[serde(default)]
    pub result: Option<serde_json::Value>,
    #[serde(default)]
    pub error: Option<String>,
}

/// Réponse POST ou GET /v1/hubs/:id/commands/:cmd_id
#[derive(Debug, Clone, Serialize)]
pub struct CommandResponse {
    pub command: Command,
}

/// Réponse GET /v1/hubs/:id/commands
#[derive(Debug, Clone, Serialize)]
pub struct CommandsListResponse {
    pub commands: Vec<Command>,
    pub count: usize,
}
