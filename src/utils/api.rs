use crate::models::{ClassData, SigningCode};
#[cfg(feature = "server")]
use crate::utils::db::get_db;
#[cfg(feature = "server")]
use crate::utils::time::get_formatted_time;
use dioxus::prelude::*;

#[server(endpoint = "save_scanned_code_data")]
pub async fn save_scanned_code_data(
    site_id: String,
    class_name: Option<String>,
    classes: Option<String>,
    checkwork_id: Option<String>,
    class_lesson_id: Option<String>,
    created_time: Option<String>,
) -> Result<String, ServerFnError> {
    let current_time = get_formatted_time();
    let db = get_db();
    let conn = db
        .lock()
        .map_err(|e| ServerFnError::new(format!("Database lock error: {}", e)))?;
    let exists = conn
        .query_row(
            "SELECT COUNT(*) FROM class_data WHERE site_id = ?1",
            rusqlite::params![site_id],
            |row| row.get::<_, i64>(0),
        )
        .map_err(|e| ServerFnError::new(format!("Database query error: {}", e)))?;
    if exists > 0 {
        if checkwork_id.is_some() || class_lesson_id.is_some() || created_time.is_some() {
            conn.execute(
                "UPDATE class_data 
                 SET last_checkwork_id = COALESCE(?1, last_checkwork_id), 
                     last_class_lesson_id = COALESCE(?2, last_class_lesson_id), 
                     last_created_time = COALESCE(?3, last_created_time),
                     is_expired = 0,
                     updated_at = ?4
                 WHERE site_id = ?5",
                rusqlite::params![
                    checkwork_id,
                    class_lesson_id,
                    created_time,
                    current_time,
                    site_id
                ],
            )
            .map_err(|e| ServerFnError::new(format!("Database update error: {}", e)))?;
        }
        if class_name.is_some() || classes.is_some() {
            conn.execute(
                "UPDATE class_data 
                 SET class_name = COALESCE(?1, class_name),
                     classes = COALESCE(?2, classes),
                     updated_at = ?3
                 WHERE site_id = ?4",
                rusqlite::params![class_name, classes, current_time, site_id],
            )
            .map_err(|e| ServerFnError::new(format!("Database update error: {}", e)))?;
        }
        Ok(format!("课程签到信息已更新 - {}", current_time))
    } else {
        conn.execute(
            "INSERT INTO class_data 
             (site_id, class_name, classes, last_checkwork_id, last_class_lesson_id, last_created_time, is_expired, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, 0, ?7, ?8)",
            rusqlite::params![
                site_id,
                class_name.unwrap_or_else(|| format!("Course_{}", site_id)),
                classes.unwrap_or_else(|| "Unknown Class".to_string()),
                checkwork_id,
                class_lesson_id,
                created_time,
                current_time,
                current_time
            ]
        ).map_err(|e| ServerFnError::new(format!("Database insert error: {}", e)))?;
        Ok(format!("新课程签到信息已保存 - {}", current_time))
    }
}

#[server(endpoint = "save_signing_code")]
pub async fn save_signing_code(signing_code: SigningCode) -> Result<String, ServerFnError> {
    save_scanned_code_data(
        signing_code.site_id,
        None,
        None,
        Some(signing_code.id),
        Some(signing_code.class_lesson_id),
        Some(signing_code.create_time),
    )
    .await
}

#[server]
pub async fn log_scan_result(data: String) -> Result<(), ServerFnError> {
    use std::fs::OpenOptions;
    use std::io::Write;
    use chrono::Local;

    let now = Local::now();
    let timestamp = now.format("%Y-%m-%d %H:%M:%S");
    let log_entry = format!("[{}] {}\n", timestamp, data);

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("scanlogs.txt")
        .map_err(|e| ServerFnError::new(format!("Failed to open log file: {}", e)))?;

    file.write_all(log_entry.as_bytes())
        .map_err(|e| ServerFnError::new(format!("Failed to write to log file: {}", e)))?;

    Ok(())
}

#[server(endpoint = "update_class_info")]
pub async fn update_class_info(
    site_id: String,
    class_name: Option<String>,
    classes: Option<String>,
) -> Result<String, ServerFnError> {
    save_scanned_code_data(
        site_id,
        class_name,
        classes,
        None,
        None,
        None,
    )
    .await
}

#[server(endpoint = "get_current_time")]
pub async fn get_current_time() -> Result<String, ServerFnError> {
    use chrono::{FixedOffset, Utc};
    let china_tz = FixedOffset::east_opt(8 * 3600).unwrap();
    let current_time = Utc::now()
        .with_timezone(&china_tz)
        .format("%Y-%m-%dT%H:%M:%S%.3f")
        .to_string();
    Ok(current_time)
}

#[server(endpoint = "get_class_data")]
pub async fn get_class_data(site_id: String) -> Result<Option<ClassData>, ServerFnError> {
    let db = get_db();
    let conn = db
        .lock()
        .map_err(|e| ServerFnError::new(format!("Database lock error: {}", e)))?;
    let mut stmt = conn
        .prepare(
            "SELECT id, site_id, class_name, classes, last_checkwork_id, \
                last_class_lesson_id, last_created_time, is_expired
         FROM class_data 
         WHERE site_id = ?1",
        )
        .map_err(|e| ServerFnError::new(format!("Database prepare error: {}", e)))?;
    let class_data = stmt.query_row(rusqlite::params![site_id], |row| {
        Ok(ClassData {
            // id: row.get(0)?,
            site_id: row.get(1)?,
            class_name: row.get(2)?,
            classes: row.get(3)?,
            last_checkwork_id: row.get(4)?,
            last_class_lesson_id: row.get(5)?,
            last_created_time: row.get(6)?,
            is_expired: row.get(7)?,
        })
    });
    match class_data {
        Ok(data) => Ok(Some(data)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(ServerFnError::new(format!("Database query error: {}", e))),
    }
}

#[server(endpoint = "get_class_data_by_id")]
pub async fn get_class_data_by_id(id: i64) -> Result<Option<ClassData>, ServerFnError> {
    let db = get_db();
    let conn = db
        .lock()
        .map_err(|e| ServerFnError::new(format!("Database lock error: {}", e)))?;
    let mut stmt = conn
        .prepare(
            "SELECT id, site_id, class_name, classes, last_checkwork_id, \
                last_class_lesson_id, last_created_time, is_expired
         FROM class_data 
         WHERE id = ?1",
        )
        .map_err(|e| ServerFnError::new(format!("Database prepare error: {}", e)))?;
    let class_data = stmt.query_row(rusqlite::params![id], |row| {
        Ok(ClassData {
            // id: row.get(0)?,
            site_id: row.get(1)?,
            class_name: row.get(2)?,
            classes: row.get(3)?,
            last_checkwork_id: row.get(4)?,
            last_class_lesson_id: row.get(5)?,
            last_created_time: row.get(6)?,
            is_expired: row.get(7)?,
        })
    });
    match class_data {
        Ok(data) => Ok(Some(data)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(ServerFnError::new(format!("Database query error: {}", e))),
    }
}

#[server(endpoint = "get_class_id")]
pub async fn get_class_id(site_id: String) -> Result<Option<i64>, ServerFnError> {
    let db = get_db();
    let conn = db
        .lock()
        .map_err(|e| ServerFnError::new(format!("Database lock error: {}", e)))?;
    let mut stmt = conn
        .prepare("SELECT id FROM class_data WHERE site_id = ?1")
        .map_err(|e| ServerFnError::new(format!("Database prepare error: {}", e)))?;
    let result = stmt.query_row(rusqlite::params![site_id], |row| row.get(0));
    match result {
        Ok(id) => Ok(Some(id)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(ServerFnError::new(format!("Database query error: {}", e))),
    }
}

#[server(endpoint = "list_all_classes")]
pub async fn list_all_classes() -> Result<Vec<ClassData>, ServerFnError> {
    let db = get_db();
    let conn = db
        .lock()
        .map_err(|e| ServerFnError::new(format!("Database lock error: {}", e)))?;
    let mut stmt = conn
        .prepare(
            "SELECT id, site_id, class_name, classes, last_checkwork_id,\
                last_class_lesson_id, last_created_time, is_expired
         FROM class_data 
         ORDER BY updated_at DESC",
        )
        .map_err(|e| ServerFnError::new(format!("Database prepare error: {}", e)))?;
    let class_iter = stmt
        .query_map([], |row| {
            Ok(ClassData {
                // id: row.get(0)?,
                site_id: row.get(1)?,
                class_name: row.get(2)?,
                classes: row.get(3)?,
                last_checkwork_id: row.get(4)?,
                last_class_lesson_id: row.get(5)?,
                last_created_time: row.get(6)?,
                is_expired: row.get(7)?,
            })
        })
        .map_err(|e| ServerFnError::new(format!("Database query error: {}", e)))?;
    let mut classes = Vec::new();
    for class in class_iter {
        classes.push(class.map_err(|e| ServerFnError::new(format!("Row parse error: {}", e)))?);
    }
    Ok(classes)
}

#[server(endpoint = "mark_class_expired")]
pub async fn mark_class_expired(site_id: String) -> Result<String, ServerFnError> {
    let db = get_db();
    let conn = db
        .lock()
        .map_err(|e| ServerFnError::new(format!("Database lock error: {}", e)))?;
    let updated_rows = conn
        .execute(
            "UPDATE class_data SET is_expired = 1 WHERE site_id = ?1",
            rusqlite::params![site_id],
        )
        .map_err(|e| ServerFnError::new(format!("Database update error: {}", e)))?;
    if updated_rows > 0 {
        Ok("课程已标记为过期".to_string())
    } else {
        Err(ServerFnError::new("未找到指定课程".to_string()))
    }
}
