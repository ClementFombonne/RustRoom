use super::{
    Catalog,
    models::{AlbumRecord, ImageRecord},
};
use chrono::DateTime;
use chrono::Utc;
use rusqlite::Result as SqlResult;

impl Catalog {
    pub fn create_album(&self, name: &str) -> SqlResult<i64> {
        let now = Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT INTO albums (name, created_at) VALUES (?1, ?2)",
            rusqlite::params![name, now],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn get_albums(&self) -> SqlResult<Vec<AlbumRecord>> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, name FROM albums ORDER BY name ASC")?;
        let album_iter = stmt.query_map([], |row| {
            Ok(AlbumRecord {
                id: row.get(0)?,
                name: row.get(1)?,
            })
        })?;

        let mut albums = Vec::new();
        for album in album_iter {
            albums.push(album?);
        }
        Ok(albums)
    }

    pub fn add_image_to_album(&self, image_id: i64, album_id: i64) -> SqlResult<()> {
        self.conn.execute(
            "INSERT OR IGNORE INTO album_images (album_id, image_id) VALUES (?1, ?2)",
            rusqlite::params![album_id, image_id],
        )?;
        Ok(())
    }

    pub fn get_images_for_album(&self, album_id: i64) -> SqlResult<Vec<ImageRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT i.id, i.original_path, i.preview_path, i.imported_at, i.metadata_json 
             FROM images i
             JOIN album_images ai ON i.id = ai.image_id
             WHERE ai.album_id = ?1
             ORDER BY i.imported_at DESC",
        )?;

        let image_iter = stmt.query_map([album_id], |row| {
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
