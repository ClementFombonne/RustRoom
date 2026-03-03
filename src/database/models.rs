use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct ImageRecord {
    pub id: i64,
    pub original_path: String,
    pub preview_path: String,
    pub imported_at: DateTime<Utc>,
    pub metadata_json: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditAction {
    pub operation: String,
    pub value: f32,
}

#[derive(Debug, Clone)]
pub struct HistoryRecord {
    pub id: i64,
    pub image_id: i64,
    pub action: EditAction,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct AlbumRecord {
    pub id: i64,
    pub name: String,
}
