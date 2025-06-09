use sqlx::SqlitePool;
use crate::models::{DbClass, SigningCode, ClassCacheEntry};
use std::io::Write;
use chrono::{Utc, Duration, FixedOffset};

/// 缓存有效期（5分钟）
const CACHE_DURATION_MINUTES: i64 = 5;

/// 课程过期时间（20分钟）
const COURSE_EXPIRY_MINUTES: i64 = 20;

/// 从数据库获取课程名称
pub async fn get_class_name_from_db(pool: &SqlitePool, class_lesson_id: &str) -> String {
    
    let result = sqlx::query_as::<_, DbClass>(
        "SELECT * FROM classes WHERE class_lesson_id = ? LIMIT 1"
    )
    .bind(class_lesson_id)
    .fetch_optional(pool)
    .await;

    match result {
        Ok(Some(class)) => class.lesson_name,
        _ => "unknown".to_string(),
    }
}

/// 向数据库插入或更新课程信息
pub async fn upsert_class_to_db(
    pool: &SqlitePool,
    class_lesson_id: &str,
    lesson_name: &str,
    check_id: &str,
    site_id: &str,
    create_time: &str,
) -> Result<(), sqlx::Error> {
    let existing_record = sqlx::query_as::<_, DbClass>(
        "SELECT * FROM classes WHERE class_lesson_id = ? LIMIT 1"
    )
    .bind(class_lesson_id)
    .fetch_optional(pool)
    .await?;
    
    match existing_record {
        Some(_) => {
            // 更新现有记录
            sqlx::query(
                r#"
                UPDATE classes 
                SET lesson_name = ?, last_check_id = ?, last_site_id = ?, last_create_time = ?, is_expired = 0
                WHERE class_lesson_id = ?
                "#
            )
            .bind(lesson_name)
            .bind(check_id)
            .bind(site_id)
            .bind(create_time)
            .bind(class_lesson_id)
            .execute(pool)
            .await?;
        }
        None => {
            // 插入新记录
            sqlx::query(
                r#"
                INSERT INTO classes (class_lesson_id, lesson_name, last_check_id, last_site_id, last_create_time, is_expired)
                VALUES (?, ?, ?, ?, ?, 0)
                "#
            )
            .bind(class_lesson_id)
            .bind(lesson_name)
            .bind(check_id)
            .bind(site_id)
            .bind(create_time)
            .execute(pool)
            .await?;
        }
    }

    Ok(())
}

/// 初始化数据库并运行迁移
pub async fn initialize_database(database_url: &str) -> Result<SqlitePool, Box<dyn std::error::Error>> {
    let db_pool = SqlitePool::connect(database_url).await?;
    
    // println!("正在运行数据库迁移...");
    let migration_sql = include_str!("../migrations/001_create_classes_table.sql");
    let statements: Vec<&str> = migration_sql
        .split(';')
        .filter(|s| !s.trim().is_empty())
        .collect();
    
    for statement in statements {
        if let Err(e) = sqlx::query(statement).execute(&db_pool).await {
            eprintln!("数据库迁移失败: {}", e);
        }
    }
    
    println!("数据库迁移完成");
    Ok(db_pool)
}

/// 写入扫描记录到日志文件
pub async fn write_scan_log(
    class_lesson_id: &str,
    check_id: &str,
    site_id: &str,
    create_time: &str,
) -> Result<(), std::io::Error> {
    let timestamp = Utc::now().format("%Y-%m-%d %H:%M:%S UTC");
    let log_entry = format!(
        "[{}] 扫描记录: class_lesson_id={}, check_id={}, site_id={}, create_time={}\n",
        timestamp, class_lesson_id, check_id, site_id, create_time
    );
    
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open("scan_logs.txt")?;
    
    file.write_all(log_entry.as_bytes())?;
    file.flush()?;
    
    println!("扫描记录已写入日志: {}", log_entry.trim());
    Ok(())
}

/// 保存扫描信息到数据库和日志
pub async fn save_scan_info(
    pool: SqlitePool,
    signing_code: SigningCode,
) {
    let class_lesson_id = signing_code.class_lesson_id.clone();
    let site_id = signing_code.site_id.clone();
    let qr_id = signing_code.id.clone();
    let create_time = signing_code.create_time.clone();
    
    // 检查数据库中是否已有该课程记录
    let existing_record = sqlx::query_as::<_, DbClass>(
        "SELECT * FROM classes WHERE class_lesson_id = ? LIMIT 1"
    )
    .bind(&class_lesson_id)
    .fetch_optional(&pool)
    .await;
    
    let lesson_name = match existing_record {
        Ok(Some(class)) => class.lesson_name,
        _ => "unknown".to_string(),
    };
    
    // 保存课程信息到数据库
    if let Err(e) = upsert_class_to_db(&pool, &class_lesson_id, &lesson_name, &qr_id, &site_id, &create_time).await {
        eprintln!("保存课程信息到数据库失败: {}", e);
        return;
    }
    
    // 写入扫描记录到日志文件
    if let Err(e) = write_scan_log(&class_lesson_id, &qr_id, &site_id, &create_time).await {
        eprintln!("写入扫描日志失败: {}", e);
    }
}

/// 从数据库获取所有课程列表
pub async fn get_all_classes_from_db(pool: &SqlitePool) -> Result<Vec<DbClass>, sqlx::Error> {
    let classes = sqlx::query_as::<_, DbClass>(
        "SELECT * FROM classes ORDER BY lesson_name"
    )
    .fetch_all(pool)
    .await?;
    
    Ok(classes)
}

/// 检查缓存是否有效
pub fn is_cache_valid(cache_entry: &ClassCacheEntry) -> bool {
    let now = Utc::now();
    let cache_age = now.signed_duration_since(cache_entry.cached_at);
    cache_age < Duration::minutes(CACHE_DURATION_MINUTES)
}

/// 更新课程的过期状态
pub async fn update_course_expired_status(
    pool: &SqlitePool, 
    class_lesson_id: &str, 
    is_expired: bool
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE classes SET is_expired = ? WHERE class_lesson_id = ?"
    )
    .bind(is_expired)
    .bind(class_lesson_id)
    .execute(pool)
    .await?;
    
    Ok(())
}

/// 检查并更新所有课程的过期状态
pub async fn check_and_update_expired_courses(
    pool: &SqlitePool
) -> Result<Vec<DbClass>, sqlx::Error> {
    // 获取所有课程
    let mut classes = get_all_classes_from_db(pool).await?;
    
    // 北京时间偏移量 (UTC+8)
    let beijing_offset = FixedOffset::east_opt(8 * 3600).unwrap();
    
    // 检查每个课程的过期状态
    for class in &mut classes {
        let mut should_update = false;
        let mut new_expired_status = class.is_expired;
        
        // 检查是否应该根据last_create_time标记为过期
        if let Some(ref last_create_time) = class.last_create_time {
            println!("检查课程 {} 的过期状态，创建时间: {}", class.class_lesson_id, last_create_time);
            
            // 尝试多种时间格式解析
            let create_time_result = if last_create_time.ends_with('Z') {
                // 标准 RFC3339 格式，如 "2025-06-09T20:30:04.003Z"
                chrono::DateTime::parse_from_rfc3339(last_create_time)
            } else if last_create_time.contains('+') {
                // 带时区偏移的格式，如 "2025-06-09T20:30:04.003+08:00"
                chrono::DateTime::parse_from_rfc3339(last_create_time)
            } else if last_create_time.contains('T') {
                // ISO 格式但没有时区，如 "2025-06-09T20:30:04.003"
                // 假设为北京时间，添加 +08:00 后缀
                let time_with_tz = if last_create_time.len() == 23 {
                    // 处理毫秒格式 "2025-06-09T20:30:04.003"
                    format!("{}+08:00", last_create_time)
                } else if last_create_time.len() == 19 {
                    // 处理秒格式 "2025-06-09T20:30:04"
                    format!("{}+08:00", last_create_time)
                } else {
                    format!("{}+08:00", last_create_time)
                };
                chrono::DateTime::parse_from_rfc3339(&time_with_tz)
            } else {
                // 其他格式，如 "2025-06-09 20:30:04"
                let time_with_tz = if last_create_time.contains(' ') {
                    format!("{}Z", last_create_time.replace(' ', "T"))
                } else {
                    format!("{}Z", last_create_time)
                };
                chrono::DateTime::parse_from_rfc3339(&time_with_tz)
            };
            
            if let Ok(create_time) = create_time_result {
                // 转换为北京时间进行计算
                let create_time_beijing = create_time.with_timezone(&beijing_offset);
                let now_beijing = chrono::Utc::now().with_timezone(&beijing_offset);
                let duration = now_beijing.signed_duration_since(create_time_beijing);
                let minutes_elapsed = duration.num_minutes();
                
                println!("课程 {} 创建于: {} (北京时间), 当前时间: {} (北京时间), 已过去{}分钟", 
                    class.class_lesson_id, create_time_beijing, now_beijing, minutes_elapsed);
                
                // 如果超过指定时间，标记为过期
                if minutes_elapsed > COURSE_EXPIRY_MINUTES && !class.is_expired {
                    new_expired_status = true;
                    should_update = true;
                    println!("课程 {} 应该标记为过期 ({}分钟已过期)", class.class_lesson_id, COURSE_EXPIRY_MINUTES);
                } else if minutes_elapsed <= COURSE_EXPIRY_MINUTES && class.is_expired {
                    // 如果时间在有效期内但被标记为过期，则取消过期状态
                    new_expired_status = false;
                    should_update = true;
                    println!("课程 {} 应该标记为未过期", class.class_lesson_id);
                }
            } else {
                println!("无法解析课程 {} 的时间格式: {}", class.class_lesson_id, last_create_time);
            }
        }
        
        // 更新数据库中的过期状态
        if should_update {
            if let Err(e) = update_course_expired_status(pool, &class.class_lesson_id, new_expired_status).await {
                eprintln!("更新课程 {} 过期状态失败: {}", class.class_lesson_id, e);
            } else {
                class.is_expired = new_expired_status;
                println!("更新课程 {} 过期状态为: {}", class.class_lesson_id, new_expired_status);
            }
        }
    }
    
    Ok(classes)
}
