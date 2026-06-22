use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::SqlitePool;
use std::fs;
use std::path::PathBuf;

/// Ermittelt den App-Datenordner (plattformspezifisch über `dirs`)
/// und legt ihn an, falls er noch nicht existiert.
fn app_data_dir() -> PathBuf {
    let mut dir = dirs::data_dir().expect("Kein Daten-Verzeichnis gefunden");
    dir.push("social-graph");
    fs::create_dir_all(&dir).expect("Konnte App-Datenordner nicht anlegen");
    dir
}

pub async fn init_pool() -> SqlitePool {
    let mut db_path = app_data_dir();
    db_path.push("social-graph.db");

    let options = SqliteConnectOptions::new()
        .filename(&db_path)
        .create_if_missing(true)
        .foreign_keys(true);

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await
        .expect("Konnte keine Verbindung zur Datenbank aufbauen");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Migration fehlgeschlagen");

    pool
}
