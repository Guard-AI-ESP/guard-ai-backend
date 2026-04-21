use crate::db::timestamp::{iso_in_seconds, now_iso};
use crate::models::command::{Command, CommandStatus, CommandType};
use serde_json::Value;
use sqlx::SqlitePool;
use uuid::Uuid;

/// Queue persistante des commandes envoyées aux hubs.
#[derive(Clone)]
pub struct CommandRepository {
    pool: SqlitePool,
}

impl CommandRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Enqueue une nouvelle commande (status=pending).
    pub async fn insert(
        &self,
        hub_id: &str,
        command_type: CommandType,
        payload: &Value,
        ttl_seconds: Option<i64>,
    ) -> Result<Command, sqlx::Error> {
        let id = Uuid::new_v4();
        let created_at = now_iso();
        let expires_at = ttl_seconds.map(iso_in_seconds);
        let payload_json = serde_json::to_string(payload).unwrap_or_else(|_| "{}".to_string());

        sqlx::query(
            "INSERT INTO commands (id, hub_id, type, payload, status, created_at, expires_at)
             VALUES (?, ?, ?, ?, 'pending', ?, ?)",
        )
        .bind(id.to_string())
        .bind(hub_id)
        .bind(command_type.as_str())
        .bind(&payload_json)
        .bind(&created_at)
        .bind(&expires_at)
        .execute(&self.pool)
        .await?;

        Ok(Command {
            id,
            hub_id: hub_id.to_string(),
            command_type,
            payload: payload.clone(),
            status: CommandStatus::Pending,
            created_at,
            dispatched_at: None,
            completed_at: None,
            expires_at,
            result: None,
            error: None,
        })
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<Command>, sqlx::Error> {
        let row = sqlx::query_as::<_, CommandRow>(
            "SELECT id, hub_id, type, payload, status, result, error,
                    created_at, dispatched_at, completed_at, expires_at
             FROM commands WHERE id = ?",
        )
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.and_then(|r| r.try_into().ok()))
    }

    pub async fn find_by_hub(
        &self,
        hub_id: &str,
        status_filter: Option<CommandStatus>,
    ) -> Result<Vec<Command>, sqlx::Error> {
        let rows = match status_filter {
            Some(status) => {
                sqlx::query_as::<_, CommandRow>(
                    "SELECT id, hub_id, type, payload, status, result, error,
                            created_at, dispatched_at, completed_at, expires_at
                     FROM commands WHERE hub_id = ? AND status = ?
                     ORDER BY created_at DESC",
                )
                .bind(hub_id)
                .bind(status.as_str())
                .fetch_all(&self.pool)
                .await?
            }
            None => {
                sqlx::query_as::<_, CommandRow>(
                    "SELECT id, hub_id, type, payload, status, result, error,
                            created_at, dispatched_at, completed_at, expires_at
                     FROM commands WHERE hub_id = ?
                     ORDER BY created_at DESC",
                )
                .bind(hub_id)
                .fetch_all(&self.pool)
                .await?
            }
        };

        Ok(rows.into_iter().filter_map(|r| r.try_into().ok()).collect())
    }

    /// Passe une commande en `dispatched` (appelé quand le WS Hub a reçu le payload).
    pub async fn mark_dispatched(&self, id: Uuid) -> Result<(), sqlx::Error> {
        let ts = now_iso();
        sqlx::query(
            "UPDATE commands SET status = 'dispatched', dispatched_at = ?
             WHERE id = ? AND status = 'pending'",
        )
        .bind(&ts)
        .bind(id.to_string())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Mise à jour finale du cycle de vie par le hub (running → succeeded / failed).
    pub async fn update_status(
        &self,
        id: Uuid,
        status: CommandStatus,
        result: Option<&Value>,
        error: Option<&str>,
    ) -> Result<Option<Command>, sqlx::Error> {
        let result_json = result.map(|v| serde_json::to_string(v).unwrap_or_default());
        let completed_at =
            matches!(status, CommandStatus::Succeeded | CommandStatus::Failed).then(now_iso);

        sqlx::query(
            "UPDATE commands
               SET status = ?,
                   result = COALESCE(?, result),
                   error  = COALESCE(?, error),
                   completed_at = COALESCE(?, completed_at)
             WHERE id = ?",
        )
        .bind(status.as_str())
        .bind(&result_json)
        .bind(error)
        .bind(&completed_at)
        .bind(id.to_string())
        .execute(&self.pool)
        .await?;

        self.find_by_id(id).await
    }
}

#[derive(sqlx::FromRow)]
struct CommandRow {
    id: String,
    hub_id: String,
    #[sqlx(rename = "type")]
    kind: String,
    payload: String,
    status: String,
    result: Option<String>,
    error: Option<String>,
    created_at: String,
    dispatched_at: Option<String>,
    completed_at: Option<String>,
    expires_at: Option<String>,
}

impl TryFrom<CommandRow> for Command {
    type Error = ();

    fn try_from(r: CommandRow) -> Result<Self, Self::Error> {
        let id = Uuid::parse_str(&r.id).map_err(|_| ())?;
        let command_type = CommandType::parse(&r.kind).ok_or(())?;
        let status = CommandStatus::parse(&r.status).ok_or(())?;
        let payload: Value = serde_json::from_str(&r.payload).unwrap_or(Value::Null);
        let result: Option<Value> = r
            .result
            .as_deref()
            .and_then(|s| serde_json::from_str(s).ok());

        Ok(Command {
            id,
            hub_id: r.hub_id,
            command_type,
            payload,
            status,
            created_at: r.created_at,
            dispatched_at: r.dispatched_at,
            completed_at: r.completed_at,
            expires_at: r.expires_at,
            result,
            error: r.error,
        })
    }
}
