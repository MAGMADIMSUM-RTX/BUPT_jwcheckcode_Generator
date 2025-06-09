use actix_web::{web, HttpResponse, Result};
use actix_files::NamedFile;
use chrono::Utc;
use std::sync::{Arc, Mutex};

use crate::models::*;
use crate::database::{get_all_classes_from_db, is_cache_valid, check_and_update_expired_courses};

pub type AppState = Arc<Mutex<AppData>>;

/// 扫描页面 (根路径)
pub async fn scan_page() -> Result<NamedFile> {
    Ok(NamedFile::open("./static/scan.html")?)
}

/// 选择器页面
pub async fn selector_page(app_state: web::Data<AppState>) -> Result<NamedFile> {
    // 预加载课程数据到缓存（在访问选择器页面时触发）
    {
        let app_data = app_state.lock().unwrap();
        let should_load_cache = if let Some(ref cache_entry) = app_data.class_cache {
            !is_cache_valid(cache_entry)
        } else {
            true
        };
        
        if should_load_cache {
            let db_pool = app_data.db_pool.clone();
            drop(app_data);
            
            // 异步加载课程数据到缓存
            if let Ok(classes) = get_all_classes_from_db(&db_pool).await {
                let mut app_data = app_state.lock().unwrap();
                app_data.class_cache = Some(ClassCacheEntry {
                    classes,
                    cached_at: Utc::now(),
                });
                println!("选择器页面访问时预加载课程数据到缓存");
            }
        }
    }
    
    Ok(NamedFile::open("./static/selector.html")?)
}

/// 生成页面
pub async fn generate_page(
    path: web::Path<(String, String)>, 
    app_state: web::Data<AppState>
) -> Result<HttpResponse> {
    let (type_param, content) = path.into_inner();
    
    // 目前只支持 classid 类型
    if type_param != "classid" {
        return Ok(HttpResponse::Found()
            .append_header(("Location", "/"))
            .finish());
    }
    
    let class_lesson_id = content;
    
    // 预加载课程数据到缓存（在访问生成页面时触发）
    {
        let app_data = app_state.lock().unwrap();
        let should_load_cache = if let Some(ref cache_entry) = app_data.class_cache {
            !is_cache_valid(cache_entry)
        } else {
            true
        };
        
        if should_load_cache {
            let db_pool = app_data.db_pool.clone();
            drop(app_data);
            
            // 异步加载课程数据到缓存
            if let Ok(classes) = get_all_classes_from_db(&db_pool).await {
                let mut app_data = app_state.lock().unwrap();
                app_data.class_cache = Some(ClassCacheEntry {
                    classes,
                    cached_at: Utc::now(),
                });
                println!("生成页面访问时预加载课程数据到缓存");
            }
        }
    }
    
    // 检查课程是否在数据库中且未过期
    let db_pool = {
        let app_data = app_state.lock().unwrap();
        app_data.db_pool.clone()
    };
    
    // 从数据库检查课程状态
    match check_and_update_expired_courses(&db_pool).await {
        Ok(classes) => {
            let course = classes.iter().find(|c| c.class_lesson_id == class_lesson_id);
            
            match course {
                Some(course) => {
                    // 检查课程是否过期或缺少签到数据
                    if course.is_expired || course.last_check_id.is_none() || course.last_site_id.is_none() || course.last_create_time.is_none() {
                        // 课程过期或无效，重定向到扫描页面
                        return Ok(HttpResponse::Found()
                            .append_header(("Location", "/"))
                            .finish());
                    }
                }
                None => {
                    // 没有对应的课程数据，重定向到扫描页面
                    return Ok(HttpResponse::Found()
                        .append_header(("Location", "/"))
                        .finish());
                }
            }
        }
        Err(e) => {
            eprintln!("检查课程状态失败: {}", e);
            // 出错时重定向到扫描页面
            return Ok(HttpResponse::Found()
                .append_header(("Location", "/"))
                .finish());
        }
    }
    
    // 返回生成页面
    match std::fs::read_to_string("./static/generate.html") {
        Ok(content) => Ok(HttpResponse::Ok().content_type("text/html").body(content)),
        Err(_) => Ok(HttpResponse::Found()
            .append_header(("Location", "/"))
            .finish())
    }
}
