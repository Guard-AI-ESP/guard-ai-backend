use crate::models::person::Person;
use sqlx::SqlitePool;
use uuid::Uuid;

/// Repository CRUD pour la table `persons`
#[derive(Clone)]
pub struct PersonRepository {
    pool: SqlitePool,
}

impl PersonRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Insère une nouvelle personne. Retourne la personne créée.
    pub async fn insert(
        &self,
        name: &str,
        embedding: &[f64],
        photo_url: Option<&str>,
    ) -> Result<Person, sqlx::Error> {
        let id = Uuid::new_v4();
        let id_str = id.to_string();
        let embedding_json = serde_json::to_string(embedding).unwrap_or_else(|_| "[]".to_string());
        let created_at = chrono_now_iso();

        sqlx::query(
            "INSERT INTO persons (id, name, embedding, photo_url, created_at)
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(&id_str)
        .bind(name)
        .bind(&embedding_json)
        .bind(photo_url)
        .bind(&created_at)
        .execute(&self.pool)
        .await?;

        Ok(Person {
            id,
            name: name.to_string(),
            embedding: embedding.to_vec(),
            photo_url: photo_url.map(str::to_string),
            created_at,
        })
    }

    /// Retourne toutes les personnes (embedding inclus — nécessaire pour la reconnaissance).
    pub async fn find_all(&self) -> Result<Vec<Person>, sqlx::Error> {
        let rows = sqlx::query_as::<_, PersonRow>(
            "SELECT id, name, embedding, photo_url, created_at FROM persons ORDER BY created_at DESC",
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().filter_map(|r| r.try_into().ok()).collect())
    }

    /// Retourne une personne par son UUID, ou None si absente.
    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<Person>, sqlx::Error> {
        let row = sqlx::query_as::<_, PersonRow>(
            "SELECT id, name, embedding, photo_url, created_at FROM persons WHERE id = ?",
        )
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.and_then(|r| r.try_into().ok()))
    }

    /// Supprime une personne. Retourne true si une ligne a été supprimée.
    pub async fn delete(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM persons WHERE id = ?")
            .bind(id.to_string())
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }
}

// ── Row intermédiaire SQLite → Person ────────────────────────────────────────

#[derive(sqlx::FromRow)]
struct PersonRow {
    id: String,
    name: String,
    embedding: String, // JSON array
    photo_url: Option<String>,
    created_at: String,
}

impl TryFrom<PersonRow> for Person {
    type Error = ();

    fn try_from(row: PersonRow) -> Result<Self, Self::Error> {
        let id = Uuid::parse_str(&row.id).map_err(|_| ())?;
        let embedding: Vec<f64> = serde_json::from_str(&row.embedding).unwrap_or_default();

        Ok(Person {
            id,
            name: row.name,
            embedding,
            photo_url: row.photo_url,
            created_at: row.created_at,
        })
    }
}

// ── Timestamp ISO-8601 UTC ────────────────────────────────────────────────────

fn chrono_now_iso() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let (y, mo, d, h, mi, s) = epoch_to_parts(secs);
    format!("{y:04}-{mo:02}-{d:02}T{h:02}:{mi:02}:{s:02}Z")
}

fn epoch_to_parts(secs: u64) -> (u64, u64, u64, u64, u64, u64) {
    let s = secs % 60;
    let total_min = secs / 60;
    let mi = total_min % 60;
    let total_hours = total_min / 60;
    let h = total_hours % 24;
    let total_days = total_hours / 24;

    let z = total_days + 719468;
    let era = z / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let mo = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if mo <= 2 { y + 1 } else { y };

    (y, mo, d, h, mi, s)
}
