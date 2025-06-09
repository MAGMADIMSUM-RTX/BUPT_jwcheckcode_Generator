use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use chrono::{DateTime, Utc};

// 数据库模型
#[derive(Debug, FromRow, Serialize, Clone)]
pub struct DbClass {
    pub class_lesson_id: String,
    pub lesson_name: String,
    pub last_check_id: Option<String>,
    pub last_site_id: Option<String>,
    pub last_create_time: Option<String>,
    pub is_expired: bool,
}

#[derive(Debug, Clone)]
pub struct SigningCode {
    pub id: String,
    pub site_id: String,
    pub create_time: String,
    pub class_lesson_id: String,
}

// API 请求/响应模型
#[derive(Deserialize)]
pub struct QrCodeData {
    pub content: String,
}

#[derive(Serialize)]
pub struct ApiResponse {
    pub status: String,
    pub message: String,
}

#[derive(Serialize)]
pub struct QrDataResponse {
    pub content: String,
}

#[derive(Serialize)]
pub struct ClassInfo {
    pub class_lesson_id: String,
    pub class_name: String,
    pub id: String,
    pub site_id: String,
    pub scan_timestamp: String,
    pub is_expired: bool,
    pub time_remaining: i64,
}

#[derive(Serialize)]
pub struct ClassListResponse {
    pub classes: Vec<ClassInfo>,
}

#[derive(Serialize)]
pub struct ClassNameResponse {
    pub class_lesson_id: String,
    pub class_name: String,
}

// 课程缓存条目
#[derive(Debug, Clone)]
pub struct ClassCacheEntry {
    pub classes: Vec<DbClass>,
    pub cached_at: DateTime<Utc>,
}

// 应用状态
#[derive(Debug)]
pub struct AppData {
    pub db_pool: sqlx::SqlitePool,
    pub class_cache: Option<ClassCacheEntry>,
}

impl AppData {
    pub fn new(db_pool: sqlx::SqlitePool) -> Self {
        Self {
            db_pool,
            class_cache: None,
        }
    }
}

// 应用状态类型定义 (在 pages.rs 中定义)
// pub type AppState = Arc<Mutex<AppData>>;
