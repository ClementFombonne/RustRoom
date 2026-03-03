pub mod album;
pub mod image;
pub mod models;
pub mod schema;

pub use models::*;

use directories::ProjectDirs;
use rusqlite::Connection;
use std::fs;
use std::path::PathBuf;

pub struct Catalog {
    pub conn: Connection,
    pub preview_dir: PathBuf,
}

impl Catalog {
    /// Initializes the database in the standard OS app data directory
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // Resolve the standard AppData / .config / Application Support path
        let proj_dirs = ProjectDirs::from("com", "RustRoom", "RustRoom")
            .ok_or("Could not determine the home directory to store application data.")?;

        let data_dir = proj_dirs.data_dir();

        let db_path = data_dir.join("catalog.db");
        let preview_dir = data_dir.join("previews");

        // Ensure directories exist
        fs::create_dir_all(&preview_dir)?;

        // Open connection and enforce constraints
        let conn = Connection::open(&db_path)?;
        conn.execute("PRAGMA foreign_keys = ON", [])?;

        let catalog = Self { conn, preview_dir };

        // Initialize tables
        catalog.init_tables()?;

        Ok(catalog)
    }
}
