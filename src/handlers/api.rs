use actix_web::{web, HttpResponse, Result};

use crate::models::*;
use crate::database::{get_class_name_from_db, save_scan_info, check_and_update_expired_courses};
use crate::qr_parser::QrCodeParser;
use crate::time_utils::TimeProcessor;
use crate::handlers::pages::AppState;

/// 提交二维码扫描结果
pub async fn submit_qr_code(
    data: web::Json<QrCodeData>, 
    app_state: web::Data<AppState>
) -> Result<HttpResponse> {
    println!("扫描到的二维码内容: {}", data.content);
    
    let parser = QrCodeParser::new();
    
    if let Some(signing_code) = parser.parse(&data.content) {
        println!("解析到的签名码信息: {:?}", signing_code);
        
        // 直接保存到数据库
        let pool = {
            let app_data = app_state.lock().unwrap();
            app_data.db_pool.clone()
        };
        
        // 在后台任务中保存到数据库和日志
        let signing_code_for_db = signing_code.clone();
        tokio::spawn(async move {
            save_scan_info(pool, signing_code_for_db).await;
        });
        
        // 清除缓存，强制下次从数据库重新加载
        {
            let mut app_data = app_state.lock().unwrap();
            
            // 清理当前课程的缓存
            app_data.course_cache.remove(&signing_code.class_lesson_id);
            
            // 在后台任务中进行更深度的缓存清理
            let pool_for_cleanup = app_data.db_pool.clone();
            tokio::spawn(async move {
                let mut temp_cache = crate::models::CourseCache::new();
                if let Err(e) = crate::database::cleanup_expired_cache_and_db(&pool_for_cleanup, &mut temp_cache).await {
                    eprintln!("后台清理缓存失败: {}", e);
                }
            });
        }
        
        let response = ApiResponse {
            status: "success".to_string(),
            message: format!("/gencode/classid/{}", signing_code.class_lesson_id),
        };
        
        Ok(HttpResponse::Ok().json(response))
    } else {
        println!("未能匹配到有效的签名码格式");
        
        let response = ApiResponse {
            status: "error".to_string(),
            message: "二维码内容不符".to_string(),
        };
        
        Ok(HttpResponse::BadRequest().json(response))
    }
}

/// 获取QR码数据
pub async fn get_qr_data(
    path: web::Path<String>, 
    app_state: web::Data<AppState>
) -> Result<HttpResponse> {
    let class_lesson_id = path.into_inner();
    
    // 从数据库获取课程信息
    let db_pool = {
        let app_data = app_state.lock().unwrap();
        app_data.db_pool.clone()
    };
    
    // 检查并更新所有课程的过期状态
    match check_and_update_expired_courses(&db_pool).await {
        Ok(classes) => {
            // 查找对应的课程
            let course = classes.iter().find(|c| c.class_lesson_id == class_lesson_id);
            
            match course {
                Some(course) => {
                    // 检查课程是否过期
                    if course.is_expired {
                        println!("课程{}已过期，无法生成二维码", class_lesson_id);
                        let response = ApiResponse {
                            status: "error".to_string(),
                            message: "课程已过期，无法生成二维码".to_string(),
                        };
                        return Ok(HttpResponse::BadRequest().json(response));
                    }
                    
                    // 检查课程是否有必要的数据
                    let (id, site_id, create_time) = match (&course.last_check_id, &course.last_site_id, &course.last_create_time) {
                        (Some(id), Some(site_id), Some(create_time)) => (id, site_id, create_time),
                        _ => {
                            println!("课程{}缺少必要的签到数据，无法生成二维码", class_lesson_id);
                            let response = ApiResponse {
                                status: "error".to_string(),
                                message: "课程缺少签到数据，请先扫描二维码".to_string(),
                            };
                            return Ok(HttpResponse::BadRequest().json(response));
                        }
                    };
                    
                    // 解析扫描时间
                    let scanned_time = match TimeProcessor::parse_time_string(create_time) {
                        Ok(time) => time,
                        Err(error_msg) => {
                            println!("{}", error_msg);
                            let response = ApiResponse {
                                status: "error".to_string(),
                                message: error_msg,
                            };
                            return Ok(HttpResponse::BadRequest().json(response));
                        }
                    };
                    
                    // 计算最终时间
                    let final_time = TimeProcessor::calculate_final_time(scanned_time);
                    let new_time_str = TimeProcessor::format_time_with_centiseconds(final_time);

                    // 构造content字符串
                    let content = format!(
                        "checkwork|id={}&siteId={}&createTime={}&classLessonId={}", 
                        id, 
                        site_id, 
                        new_time_str, 
                        class_lesson_id
                    );
                    
                    println!("生成二维码数据: {}", content);
                    
                    let response = QrDataResponse { content };
                    Ok(HttpResponse::Ok().json(response))
                }
                None => {
                    println!("尚未扫描过class_lesson_id={}的二维码，无法生成", class_lesson_id);
                    let response = ApiResponse {
                        status: "error".to_string(),
                        message: "请先扫描二维码后再生成".to_string(),
                    };
                    Ok(HttpResponse::BadRequest().json(response))
                }
            }
        }
        Err(e) => {
            eprintln!("获取课程数据失败: {}", e);
            let response = ApiResponse {
                status: "error".to_string(),
                message: "获取课程数据失败".to_string(),
            };
            Ok(HttpResponse::InternalServerError().json(response))
        }
    }
}

/// 获取所有有效的课程列表
pub async fn get_class_list(app_state: web::Data<AppState>) -> Result<HttpResponse> {
    let db_pool = {
        let app_data = app_state.lock().unwrap();
        app_data.db_pool.clone()
    };
    
    // 检查并更新所有课程的过期状态
    match check_and_update_expired_courses(&db_pool).await {
        Ok(classes) => {
            let mut class_infos: Vec<ClassInfo> = Vec::new();
            
            for class in classes {
                // 只显示未过期且有签到数据的课程
                if !class.is_expired && class.last_check_id.is_some() && class.last_site_id.is_some() && class.last_create_time.is_some() {
                    let time_remaining = if let Some(ref create_time) = class.last_create_time {
                        // 计算从创建时间到现在过去的分钟数
                        if let Ok(create_time_parsed) = chrono::DateTime::parse_from_rfc3339(&format!("{}Z", create_time)) {
                            let now = chrono::Utc::now();
                            let duration = now.signed_duration_since(create_time_parsed.with_timezone(&chrono::Utc));
                            let elapsed_minutes = duration.num_minutes();
                            std::cmp::max(0, 45 - elapsed_minutes) // 45分钟有效期
                        } else {
                            0
                        }
                    } else {
                        0
                    };
                    
                    let class_info = ClassInfo {
                        class_lesson_id: class.class_lesson_id.clone(),
                        class_name: class.lesson_name.clone(),
                        id: class.last_check_id.unwrap_or_default(),
                        site_id: class.last_site_id.unwrap_or_default(),
                        scan_timestamp: class.last_create_time.unwrap_or_default(),
                        is_expired: class.is_expired,
                        time_remaining,
                    };
                    
                    class_infos.push(class_info);
                }
            }
            
            // 按创建时间排序（最新的在前）
            class_infos.sort_by(|a, b| b.scan_timestamp.cmp(&a.scan_timestamp));
            
            let response = ClassListResponse { classes: class_infos };
            Ok(HttpResponse::Ok().json(response))
        }
        Err(e) => {
            eprintln!("获取课程列表失败: {}", e);
            let response = ApiResponse {
                status: "error".to_string(),
                message: "获取课程列表失败".to_string(),
            };
            Ok(HttpResponse::InternalServerError().json(response))
        }
    }
}

/// 根据class_lesson_id获取课程名称
pub async fn get_class_name_api(
    path: web::Path<String>, 
    app_state: web::Data<AppState>
) -> Result<HttpResponse> {
    let class_lesson_id = path.into_inner();
    let app_data = app_state.lock().unwrap();
    let class_name = get_class_name_from_db(&app_data.db_pool, &class_lesson_id).await;
    
    let response = ClassNameResponse {
        class_lesson_id,
        class_name,
    };
    
    Ok(HttpResponse::Ok().json(response))
}

/// 获取所有数据库中的课程列表（用于选择器页面）
pub async fn get_all_courses(app_state: web::Data<AppState>) -> Result<HttpResponse> {
    // 先进行缓存清理
    {
        let mut app_data = app_state.lock().unwrap();
        let pool = app_data.db_pool.clone();
        // 清理过期缓存
        if let Err(e) = crate::database::cleanup_expired_cache_and_db(&pool, &mut app_data.course_cache).await {
            eprintln!("清理缓存失败: {}", e);
        }
    }
    
    // 从数据库加载并检查过期状态
    let db_pool = {
        let app_data = app_state.lock().unwrap();
        app_data.db_pool.clone()
    };
    
    // 检查并更新所有课程的过期状态
    match check_and_update_expired_courses(&db_pool).await {
        Ok(classes) => {
            // 将课程加载到缓存中
            {
                let mut app_data = app_state.lock().unwrap();
                for class in &classes {
                    app_data.course_cache.insert(class.clone());
                }
                println!("从数据库加载课程数据到缓存，并已检查过期状态");
            }
            
            let course_list: Vec<_> = classes.into_iter().map(|class| {
                serde_json::json!({
                    "class_lesson_id": class.class_lesson_id,
                    "lesson_name": class.lesson_name,
                    "last_check_id": class.last_check_id,
                    "last_site_id": class.last_site_id,
                    "last_create_time": class.last_create_time,
                    "is_expired": class.is_expired
                })
            }).collect();
            
            Ok(HttpResponse::Ok().json(serde_json::json!({
                "status": "success",
                "courses": course_list
            })))
        }
        Err(e) => {
            eprintln!("检查课程过期状态失败: {}", e);
            Ok(HttpResponse::InternalServerError().json(ApiResponse {
                status: "error".to_string(),
                message: "获取课程列表失败".to_string(),
            }))
        }
    }
}
