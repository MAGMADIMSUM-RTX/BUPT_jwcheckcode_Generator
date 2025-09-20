#[cfg(feature = "server")]
use std::sync::{Arc, Mutex, OnceLock};

#[cfg(feature = "server")]
static DB_POOL: OnceLock<Arc<Mutex<rusqlite::Connection>>> = OnceLock::new();

#[cfg(feature = "server")]
pub fn get_db() -> Arc<Mutex<rusqlite::Connection>> {
    DB_POOL
        .get_or_init(|| {
            let conn = initialize_database().expect("Failed to initialize database");
            Arc::new(Mutex::new(conn))
        })
        .clone()
}

#[cfg(feature = "server")]
pub fn initialize_database() -> Result<rusqlite::Connection, Box<dyn std::error::Error>> {
    if !std::path::Path::new("lesson_data.db").exists() {
        std::fs::File::create("lesson_data.db")?;
        println!("Created new database file: lesson_data.db");
    }
    let conn = rusqlite::Connection::open("lesson_data.db")?;
    
    match conn.prepare("PRAGMA journal_mode=WAL").and_then(|mut stmt| {
        stmt.query_row([], |_| Ok(()))
    }) {
        Ok(_) => println!("WAL mode enabled successfully"),
        Err(e) => println!("Warning: Failed to enable WAL mode: {}", e),
    }
    match conn.execute("PRAGMA foreign_keys=ON", []) {
        Ok(_) => println!("Foreign key constraints enabled"),
        Err(e) => println!("Warning: Failed to enable foreign keys: {}", e),
    }
    match conn.execute("PRAGMA synchronous=NORMAL", []) {
        Ok(_) => println!("Synchronous mode set to NORMAL"),
        Err(e) => println!("Warning: Failed to set  mode: {}", e),
    }

    // Check if we need to migrate from old schema
    let has_old_schema = conn
        .prepare("PRAGMA table_info(class_data)")
        .and_then(|mut stmt| {
            let column_iter = stmt.query_map([], |row| {
                let column_name: String = row.get(1)?;
                Ok(column_name)
            })?;
            
            let mut has_class_lesson_id = false;
            let mut has_last_site_id = false;
            
            for column in column_iter {
                match column {
                    Ok(name) if name == "class_lesson_id" => has_class_lesson_id = true,
                    Ok(name) if name == "last_site_id" => has_last_site_id = true,
                    _ => {}
                }
            }
            
            Ok(has_class_lesson_id && has_last_site_id)
        })
        .unwrap_or(false);

    if has_old_schema {
        println!("Migrating database schema...");
        
        // Create new table with new schema
        conn.execute(
            "CREATE TABLE class_data_new (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                site_id TEXT NOT NULL UNIQUE,
                class_name TEXT NOT NULL,
                classes TEXT NOT NULL,
                last_checkwork_id TEXT,
                last_class_lesson_id TEXT,
                last_created_time DATETIME,
                is_expired BOOLEAN NOT NULL DEFAULT 0,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
            []
        )?;

        // Migrate data: class_lesson_id -> site_id, last_site_id -> last_class_lesson_id
        conn.execute(
            "INSERT INTO class_data_new (site_id, class_name, classes, last_checkwork_id, last_class_lesson_id, last_created_time, is_expired, created_at, updated_at)
             SELECT class_lesson_id, class_name, classes, last_checkwork_id, last_site_id, last_created_time, is_expired, created_at, updated_at
             FROM class_data",
            []
        )?;

        // Drop old table and rename new table
        conn.execute("DROP TABLE class_data", [])?;
        conn.execute("ALTER TABLE class_data_new RENAME TO class_data", [])?;

        println!("Database schema migration completed successfully");
    } else {
        // Create the class_data table if it doesn't already exist
        conn.execute(
            "CREATE TABLE IF NOT EXISTS class_data (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                site_id TEXT NOT NULL UNIQUE,
                class_name TEXT NOT NULL,
                classes TEXT NOT NULL,
                last_checkwork_id TEXT,
                last_class_lesson_id TEXT,
                last_created_time DATETIME,
                is_expired BOOLEAN NOT NULL DEFAULT 0,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
            []
        )?;
    }

    // Create trigger
    conn.execute(
        "CREATE TRIGGER IF NOT EXISTS update_class_data_timestamp 
         AFTER UPDATE ON class_data
         BEGIN
             UPDATE class_data SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
         END",
        []
    )?;

    println!("Database initialized successfully");
    Ok(conn)
}
