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
pub struct CourseCache {
    pub classes: std::collections::HashMap<String, CachedCourse>,
    pub last_cleanup: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct CachedCourse {
    pub class_data: DbClass,
    pub cached_at: DateTime<Utc>,
}

impl CourseCache {
    pub fn new() -> Self {
        Self {
            classes: std::collections::HashMap::new(),
            last_cleanup: Utc::now(),
        }
    }
    
    pub fn insert(&mut self, class: DbClass) {
        let class_id = class.class_lesson_id.clone();
        self.classes.insert(class_id, CachedCourse {
            class_data: class,
            cached_at: Utc::now(),
        });
    }
    
    pub fn get(&self, class_id: &str) -> Option<&DbClass> {
        self.classes.get(class_id).map(|cached| &cached.class_data)
    }
    
    pub fn remove(&mut self, class_id: &str) -> Option<CachedCourse> {
        self.classes.remove(class_id)
    }
    
    pub fn cleanup_expired(&mut self, cache_duration_minutes: i64, course_expiry_minutes: i64) -> Vec<String> {
        let now = Utc::now();
        let mut removed_ids = Vec::new();
        
        self.classes.retain(|class_id, cached_course| {
            // 检查缓存是否过期
            let cache_age = now.signed_duration_since(cached_course.cached_at);
            let is_cache_expired = cache_age > chrono::Duration::minutes(cache_duration_minutes);
            
            // 检查课程是否已过期
            let is_course_expired = cached_course.class_data.is_expired;
            
            // 检查课程时间是否过期
            let is_time_expired = if let Some(ref create_time) = cached_course.class_data.last_create_time {
                if let Ok(create_dt) = chrono::DateTime::parse_from_rfc3339(&format!("{}+08:00", create_time.replace('Z', ""))) {
                    let duration = now.signed_duration_since(create_dt.with_timezone(&chrono::Utc));
                    duration.num_minutes() > course_expiry_minutes
                } else {
                    false
                }
            } else {
                false
            };
            
            if is_cache_expired || is_course_expired || is_time_expired {
                removed_ids.push(class_id.clone());
                false
            } else {
                true
            }
        });
        
        self.last_cleanup = now;
        removed_ids
    }
}

// 应用状态
#[derive(Debug)]
pub struct AppData {
    pub db_pool: sqlx::SqlitePool,
    pub course_cache: CourseCache,
}

impl AppData {
    pub fn new(db_pool: sqlx::SqlitePool) -> Self {
        Self {
            db_pool,
            course_cache: CourseCache::new(),
        }
    }
}

// 应用状态类型定义 (在 pages.rs 中定义)
// pub type AppState = Arc<Mutex<AppData>>;
