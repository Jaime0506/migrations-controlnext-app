use tauri::{AppHandle, Manager};
use rusqlite::Connection;

pub fn get_db_path(app: &AppHandle) -> Result<String, String> {
    const DB_NAME: &str = "tenant-forge.db";

    // DEV: En desarrollo se sitúa en src-tauri/tenant-forge.db
    if cfg!(debug_assertions) {
        return Ok(DB_NAME.to_string());
    }

    // PROD (Windows, macOS, Linux): Siempre en AppData / Application Support
    let app_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?;

    std::fs::create_dir_all(&app_dir).map_err(|e| e.to_string())?;

    let db_path = app_dir.join(DB_NAME);
    Ok(db_path.to_string_lossy().to_string())
}

/// Abre la conexión SQLite y garantiza que las tablas base existan
pub fn get_db_connection(app: &AppHandle) -> Result<Connection, String> {
    let db_path = get_db_path(app)?;
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;

    // Crear tabla de proyectos si no existe aún
    conn.execute(
        "CREATE TABLE IF NOT EXISTS projects (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            description TEXT,
            tags TEXT,
            connections TEXT,
            scripts TEXT
        )",
        [],
    ).map_err(|e| format!("Error inicializando tabla projects: {}", e))?;

    // Asegurar columna scripts en migraciones
    let _ = conn.execute("ALTER TABLE projects ADD COLUMN scripts TEXT", []);

    Ok(conn)
}
