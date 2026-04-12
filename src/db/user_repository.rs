use crate::models::user::User;
use sqlx::SqlitePool;

/// Repository pour les opérations sur les utilisateurs
#[derive(Clone)]
pub struct UserRepository {
    pool: SqlitePool,
}

impl UserRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Crée un nouvel utilisateur — retourne l'ID inséré
    pub async fn create(&self, email: &str, password_hash: &str) -> Result<i64, sqlx::Error> {
        let result = sqlx::query(
            "INSERT INTO users (email, password_hash) VALUES (?, ?)",
        )
        .bind(email)
        .bind(password_hash)
        .execute(&self.pool)
        .await?;

        Ok(result.last_insert_rowid())
    }

    /// Cherche un utilisateur par email — `None` si introuvable
    pub async fn find_by_email(&self, email: &str) -> Result<Option<User>, sqlx::Error> {
        let row = sqlx::query_as::<_, UserRow>(
            "SELECT id, email, password_hash FROM users WHERE email = ?",
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(User::from))
    }

    /// Vérifie si un email est déjà pris
    pub async fn email_exists(&self, email: &str) -> Result<bool, sqlx::Error> {
        let (count,): (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM users WHERE email = ?")
                .bind(email)
                .fetch_one(&self.pool)
                .await?;

        Ok(count > 0)
    }
}

#[derive(sqlx::FromRow)]
struct UserRow {
    id: i64,
    email: String,
    password_hash: String,
}

impl From<UserRow> for User {
    fn from(row: UserRow) -> Self {
        Self {
            id: row.id,
            email: row.email,
            password_hash: row.password_hash,
        }
    }
}
