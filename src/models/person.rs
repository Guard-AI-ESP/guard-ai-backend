use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Personne connue stockée en base avec son embedding FaceNet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Person {
    pub id: Uuid,
    pub name: String,
    /// Vecteur d'embedding FaceNet 512-dim, sérialisé en JSON dans SQLite
    pub embedding: Vec<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub photo_url: Option<String>,
    pub created_at: String,
}

/// Corps de POST /v1/persons
#[derive(Debug, Clone, Deserialize)]
pub struct CreatePersonRequest {
    pub name: String,
    /// Embedding FaceNet (512 valeurs attendues)
    pub embedding: Vec<f64>,
    pub photo_url: Option<String>,
}

/// Réponse de POST /v1/persons
#[derive(Debug, Clone, Serialize)]
pub struct PersonResponse {
    pub person: Person,
}

/// Réponse de GET /v1/persons
#[derive(Debug, Clone, Serialize)]
pub struct PersonsListResponse {
    pub persons: Vec<Person>,
    pub count: usize,
}
