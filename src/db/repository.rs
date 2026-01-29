use crate::models::event::{EventSource, EventV1, Severity};
use sqlx::SqlitePool;
use uuid::Uuid;

/// Paramètres de filtrage pour les requêtes GET
#[derive(Debug, Default)]
pub struct EventFilter {
    pub site_id: Option<Uuid>,
    pub source: Option<EventSource>,
    pub severity: Option<Severity>,
    pub from_timestamp: Option<String>,
    pub to_timestamp: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// Repository pour les opérations CRUD sur les événements
#[derive(Clone)]
pub struct EventRepository {
    pool: SqlitePool,
}

impl EventRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Insère plusieurs événements en transaction
    pub async fn insert_batch(&self, events: &[EventV1]) -> Result<usize, sqlx::Error> {
        let mut tx = self.pool.begin().await?;
        let mut inserted = 0;

        for event in events {
            let source_str = serialize_enum(&event.source);
            let severity_str = serialize_enum(&event.severity);
            let payload_json = serde_json::to_string(&event.payload).unwrap_or_default();
            let tags_json = serde_json::to_string(&event.tags).unwrap_or_default();

            let result = sqlx::query(
                r#"
                INSERT INTO events (
                    event_id, site_id, hub_id, source, event_type, severity,
                    timestamp, payload, media_ref, tags, schema_version
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                "#,
            )
            .bind(event.event_id.to_string())
            .bind(event.site_id.to_string())
            .bind(&event.hub_id)
            .bind(&source_str)
            .bind(&event.event_type)
            .bind(&severity_str)
            .bind(&event.timestamp)
            .bind(&payload_json)
            .bind(&event.media_ref)
            .bind(&tags_json)
            .bind(&event.schema_version)
            .execute(&mut *tx)
            .await;

            if result.is_ok() {
                inserted += 1;
            }
        }

        tx.commit().await?;
        Ok(inserted)
    }

    /// Récupère les événements avec filtrage
    pub async fn find(&self, filter: &EventFilter) -> Result<Vec<EventV1>, sqlx::Error> {
        let mut query = String::from(
            "SELECT event_id, site_id, hub_id, source, event_type, severity,
                    timestamp, payload, media_ref, tags, schema_version
             FROM events WHERE 1=1",
        );

        let mut bindings: Vec<String> = Vec::new();

        if let Some(ref site_id) = filter.site_id {
            query.push_str(" AND site_id = ?");
            bindings.push(site_id.to_string());
        }
        if let Some(ref source) = filter.source {
            query.push_str(" AND source = ?");
            bindings.push(serialize_enum(source));
        }
        if let Some(ref severity) = filter.severity {
            query.push_str(" AND severity = ?");
            bindings.push(serialize_enum(severity));
        }
        if let Some(ref from) = filter.from_timestamp {
            query.push_str(" AND timestamp >= ?");
            bindings.push(from.clone());
        }
        if let Some(ref to) = filter.to_timestamp {
            query.push_str(" AND timestamp <= ?");
            bindings.push(to.clone());
        }

        query.push_str(" ORDER BY timestamp DESC");

        let limit = filter.limit.unwrap_or(100);
        query.push_str(&format!(" LIMIT {}", limit));

        if let Some(offset) = filter.offset {
            query.push_str(&format!(" OFFSET {}", offset));
        }

        let mut sqlx_query = sqlx::query_as::<_, EventRow>(&query);
        for binding in bindings {
            sqlx_query = sqlx_query.bind(binding);
        }

        let rows = sqlx_query.fetch_all(&self.pool).await?;
        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    /// Récupère un événement par son ID
    pub async fn find_by_id(&self, event_id: Uuid) -> Result<Option<EventV1>, sqlx::Error> {
        let row = sqlx::query_as::<_, EventRow>(
            "SELECT event_id, site_id, hub_id, source, event_type, severity,
                    timestamp, payload, media_ref, tags, schema_version
             FROM events WHERE event_id = ?",
        )
        .bind(event_id.to_string())
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.into()))
    }

    /// Récupère les statistiques agrégées
    pub async fn get_stats(&self) -> Result<EventStats, sqlx::Error> {
        // Total
        let total: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM events")
            .fetch_one(&self.pool)
            .await?;

        // Par sévérité
        let by_severity_rows: Vec<(String, i64)> =
            sqlx::query_as("SELECT severity, COUNT(*) as count FROM events GROUP BY severity")
                .fetch_all(&self.pool)
                .await?;

        let mut by_severity = std::collections::HashMap::new();
        for (severity, count) in by_severity_rows {
            by_severity.insert(severity, count as usize);
        }

        // Par source
        let by_source_rows: Vec<(String, i64)> =
            sqlx::query_as("SELECT source, COUNT(*) as count FROM events GROUP BY source")
                .fetch_all(&self.pool)
                .await?;

        let mut by_source = std::collections::HashMap::new();
        for (source, count) in by_source_rows {
            by_source.insert(source, count as usize);
        }

        // Dernières 24h
        let last_24h: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM events WHERE datetime(timestamp) >= datetime('now', '-24 hours')",
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(EventStats {
            total_events: total.0 as usize,
            by_severity,
            by_source,
            last_24h: last_24h.0 as usize,
        })
    }
}

/// Statistiques agrégées des événements
#[derive(Debug, Clone, serde::Serialize)]
pub struct EventStats {
    pub total_events: usize,
    pub by_severity: std::collections::HashMap<String, usize>,
    pub by_source: std::collections::HashMap<String, usize>,
    pub last_24h: usize,
}

/// Sérialise un enum en snake_case string
fn serialize_enum<T: serde::Serialize>(value: &T) -> String {
    serde_json::to_string(value)
        .unwrap_or_default()
        .trim_matches('"')
        .to_string()
}

/// Row intermédiaire pour le mapping SQLite -> EventV1
#[derive(sqlx::FromRow)]
struct EventRow {
    event_id: String,
    site_id: String,
    hub_id: Option<String>,
    source: String,
    event_type: String,
    severity: String,
    timestamp: String,
    payload: String,
    media_ref: Option<String>,
    tags: String,
    schema_version: String,
}

impl From<EventRow> for EventV1 {
    fn from(row: EventRow) -> Self {
        EventV1 {
            event_id: Uuid::parse_str(&row.event_id).unwrap_or_default(),
            site_id: Uuid::parse_str(&row.site_id).unwrap_or_default(),
            hub_id: row.hub_id,
            source: serde_json::from_str(&format!("\"{}\"", row.source))
                .unwrap_or(EventSource::System),
            event_type: row.event_type,
            severity: serde_json::from_str(&format!("\"{}\"", row.severity))
                .unwrap_or(Severity::Info),
            timestamp: row.timestamp,
            payload: serde_json::from_str(&row.payload).unwrap_or_default(),
            media_ref: row.media_ref,
            tags: serde_json::from_str(&row.tags).unwrap_or_default(),
            schema_version: row.schema_version,
        }
    }
}
