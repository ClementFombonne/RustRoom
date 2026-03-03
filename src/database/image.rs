use super::{Catalog, models::ImageRecord};
use chrono::{DateTime, Utc};
use image::GenericImageView;
use rusqlite::Result as SqlResult;
use std::path::Path;

impl Catalog {
    pub fn import_photo(&self, original_path: &str) -> Result<i64, Box<dyn std::error::Error>> {
        let path = Path::new(original_path);
        if !path.exists() {
            return Err("File does not exist".into());
        }

        let img = image::open(path)?;
        let (width, height) = img.dimensions();

        let preview = img.thumbnail(1920, 1080);
        let file_stem = path.file_stem().unwrap_or_default().to_string_lossy();
        let timestamp = Utc::now().timestamp_millis();
        let preview_filename = format!("{}_{}.jpg", file_stem, timestamp);
        let preview_path = self.preview_dir.join(preview_filename);
        preview.save(&preview_path)?;

        let mut metadata_map = serde_json::Map::new();
        metadata_map.insert("Width".to_string(), serde_json::json!(width));
        metadata_map.insert("Height".to_string(), serde_json::json!(height));
        if let Some(ext) = path.extension() {
            metadata_map.insert(
                "Format".to_string(),
                serde_json::json!(ext.to_string_lossy().to_uppercase()),
            );
        }

        if let Ok(file) = std::fs::File::open(path) {
            let mut bufreader = std::io::BufReader::new(&file);
            let exifreader = exif::Reader::new();
            if let Ok(exif_data) = exifreader.read_from_container(&mut bufreader) {
                for field in exif_data.fields() {
                    let tag_name = field.tag.to_string();
                    if tag_name == "MakerNote" || tag_name == "UserComment" {
                        continue;
                    }
                    let value = field.display_value().with_unit(&exif_data).to_string();
                    if !value.trim().is_empty() {
                        metadata_map.insert(tag_name, serde_json::json!(value));
                    }
                }
            }
        }

        let metadata_json = serde_json::Value::Object(metadata_map).to_string();
        let now = Utc::now().to_rfc3339();
        let preview_path_str = preview_path.to_string_lossy().to_string();

        self.conn.execute(
            "INSERT INTO images (original_path, preview_path, imported_at, metadata_json)
             VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![original_path, preview_path_str, now, metadata_json],
        )?;

        Ok(self.conn.last_insert_rowid())
    }

    pub fn get_all_images(&self) -> SqlResult<Vec<ImageRecord>> {
        let mut stmt = self.conn.prepare("SELECT id, original_path, preview_path, imported_at, metadata_json FROM images ORDER BY imported_at DESC")?;

        let image_iter = stmt.query_map([], |row| {
            let imported_at_str: String = row.get(3)?;
            let imported_at = DateTime::parse_from_rfc3339(&imported_at_str)
                .unwrap_or_default()
                .with_timezone(&Utc);

            Ok(ImageRecord {
                id: row.get(0)?,
                original_path: row.get(1)?,
                preview_path: row.get(2)?,
                imported_at,
                metadata_json: row.get(4)?,
            })
        })?;

        let mut images = Vec::new();
        for img in image_iter {
            images.push(img?);
        }
        Ok(images)
    }
}
