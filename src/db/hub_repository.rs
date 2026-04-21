use crate::db::timestamp::now_iso;
use crate::models::hub::Hub;
use bcrypt::{hash, verify, DEFAULT_COST};
use sqlx::SqlitePool;

/// CRUD de la table `hubs`
#[derive(Clone)]
pub struct HubRepository {
    pool: SqlitePool,
}

impl HubRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Enregistre un nouveau hub. La clé API est hashée avec bcrypt avant persistance.
    pub async fn insert(
        &self,
        id: &str,
        site_id: &str,
        name: &str,
        api_key_plain: &str,
    ) -> Result<Hub, sqlx::Error> {
        let api_key_hash = hash(api_key_plain, DEFAULT_COST)
            .map_err(|e| sqlx::Error::Protocol(format!("bcrypt hash: {e}")))?;
        let created_at = now_iso();

        sqlx::query(
            "INSERT INTO hubs (id, site_id, name, api_key_hash, created_at)
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(id)
        .bind(site_id)
        .bind(name)
        .bind(&api_key_hash)
        .bind(&created_at)
        .execute(&self.pool)
        .await?;

        Ok(Hub {
            id: id.to_string(),
            site_id: site_id.to_string(),
            name: name.to_string(),
            last_seen_at: None,
            created_at,
        })
    }

    pub async fn find_all(&self) -> Result<Vec<Hub>, sqlx::Error> {
        let rows = sqlx::query_as::<_, HubRow>(
            "SELECT id, site_id, name, last_seen_at, created_at
             FROM hubs ORDER BY created_at DESC",
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    pub async fn find_by_id(&self, id: &str) -> Result<Option<Hub>, sqlx::Error> {
        let row = sqlx::query_as::<_, HubRow>(
            "SELECT id, site_id, name, last_seen_at, created_at
             FROM hubs WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(Into::into))
    }

    /// Vérifie une clé API en clair contre le hash stocké. Utilisé pour l'auth WS du hub.
    pub async fn verify_api_key(&self, id: &str, api_key_plain: &str) -> Result<bool, sqlx::Error> {
        let row: Option<(String,)> = sqlx::query_as("SELECT api_key_hash FROM hubs WHERE id = ?")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;

        let Some((hash_stored,)) = row else {
            return Ok(false);
        };

        Ok(verify(api_key_plain, &hash_stored).unwrap_or(false))
    }

    /// Met à jour `last_seen_at` au timestamp courant (appelé par heartbeat HTTP et WS connect).
    pub async fn touch(&self, id: &str) -> Result<(), sqlx::Error> {
        let ts = now_iso();
        sqlx::query("UPDATE hubs SET last_seen_at = ? WHERE id = ?")
            .bind(&ts)
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn delete(&self, id: &str) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM hubs WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }
}

#[derive(sqlx::FromRow)]
struct HubRow {
    id: String,
    site_id: String,
    name: String,
    last_seen_at: Option<String>,
    created_at: String,
}

impl From<HubRow> for Hub {
    fn from(r: HubRow) -> Self {
        Hub {
            id: r.id,
            site_id: r.site_id,
            name: r.name,
            last_seen_at: r.last_seen_at,
            created_at: r.created_at,
        }
    }
}
