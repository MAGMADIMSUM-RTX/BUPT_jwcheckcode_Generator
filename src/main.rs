use actix_web::{web, App, HttpResponse, HttpServer, Result, middleware::Logger};
use actix_files::{Files, NamedFile};
use serde::{Deserialize, Serialize};
use regex::Regex;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use chrono::{Utc, NaiveDateTime, Timelike};
use urlencoding;

#[derive(Debug)]
struct SigningCode{
    id: String,
    site_id: String,
    create_time: String,
    class_lesson_id: String
}

#[derive(Deserialize)]
struct QrCodeData {
    content: String,
}

#[derive(Serialize)]
struct ApiResponse {
    status: String,
    message: String,
}

#[derive(Serialize)]
struct QrDataResponse {
    content: String,
}

#[derive(Serialize)]
struct ClassInfo {
    class_lesson_id: String,
    class_name: String,    // 课程名称
    id: String,
    site_id: String,
    scan_timestamp: String,
    is_expired: bool,
    time_remaining: i64,  // 剩余时间（秒）
}

#[derive(Serialize)]
struct ClassListResponse {
    classes: Vec<ClassInfo>,
}

#[derive(Serialize)]
struct ClassNameResponse {
    class_lesson_id: String,
    class_name: String,
}

// 扫描的二维码数据结构
#[derive(Debug, Clone)]
struct ScannedData {
    id: String,
    site_id: String,
    create_time: String,
    class_lesson_id: String,
    scan_timestamp: chrono::DateTime<chrono::Utc>,  // 扫描时间戳，用于过期检查
}

// 应用状态，用于存储多个class_lesson_id对应的扫描数据
#[derive(Debug, Default)]
struct AppData {
    scanned_data: HashMap<String, ScannedData>,  // class_lesson_id -> ScannedData
}

impl Default for ScannedData {
    fn default() -> Self {
        Self {
            id: String::new(),
            site_id: String::new(),
            create_time: String::new(),
            class_lesson_id: String::new(),
            scan_timestamp: Utc::now(),
        }
    }
}

type AppState = Arc<Mutex<AppData>>;

// 根据class_lesson_id获取课程名称的函数
fn get_class_name(class_lesson_id: &str) -> String {
    // 这里可以根据实际需求实现课程名称查找逻辑
    // 目前使用简单的映射作为示例，实际项目中可能需要查询数据库
    match class_lesson_id {
        "1929761676865687554" => "2023211208-光纤通信".to_string(),
        _ => "unkown".to_string(), // 默认课程名称
    }
}

// 扫描页面 (根路径)
async fn scan_page() -> Result<NamedFile> {
    Ok(NamedFile::open("./static/scan.html")?)
}

// 选择器页面
async fn selector_page() -> Result<NamedFile> {
    Ok(NamedFile::open("./static/selector.html")?)
}

// 生成页面
async fn generate_page(path: web::Path<(String, String)>, app_state: web::Data<AppState>) -> Result<HttpResponse> {
    let (type_param, content) = path.into_inner();
    
    // 目前只支持 classid 类型
    if type_param != "classid" {
        return Ok(HttpResponse::Found()
            .append_header(("Location", "/"))
            .finish());
    }
    
    let class_lesson_id = content;
    
    // 检查是否有对应class_lesson_id的扫描数据
    let app_data = app_state.lock().unwrap();
    
    if let Some(scanned_data) = app_data.scanned_data.get(&class_lesson_id) {
        // 检查扫描时间是否超过10分钟
        let current_time = Utc::now();
        let elapsed = current_time - scanned_data.scan_timestamp;
        
        if elapsed > chrono::Duration::minutes(10) {
            // 二维码已过期，重定向到扫描页面
            return Ok(HttpResponse::Found()
                .append_header(("Location", "/"))
                .finish());
        }
    } else {
        // 没有对应的扫描数据，重定向到扫描页面
        return Ok(HttpResponse::Found()
            .append_header(("Location", "/"))
            .finish());
    }
    
    drop(app_data);
    
    // 返回生成页面
    match std::fs::read_to_string("./static/generate.html") {
        Ok(content) => Ok(HttpResponse::Ok().content_type("text/html").body(content)),
        Err(_) => Ok(HttpResponse::Found()
            .append_header(("Location", "/"))
            .finish())
    }
}

async fn submit_qr_code(data: web::Json<QrCodeData>, app_state: web::Data<AppState>) -> Result<HttpResponse> {
    // 在后台打印二维码内容
    println!("扫描到的二维码内容: {}", data.content);
    
    // 使用正则表达式匹配签名码
    let signing_code_regex = Regex::new(r"checkwork\|id=(\d+)&siteId=(\d+)&createTime=([^&]+)&classLessonId=(\d+)").unwrap();
    
    if let Some(captures) = signing_code_regex.captures(&data.content) {
        let signing_code = SigningCode {
            id: captures.get(1).unwrap().as_str().to_string(),
            site_id: captures.get(2).unwrap().as_str().to_string(),
            create_time: captures.get(3).unwrap().as_str().to_string(),
            class_lesson_id: captures.get(4).unwrap().as_str().to_string(),
        };
        
        println!("解析到的签名码信息: {:?}", signing_code);
        
        // 更新应用状态中的扫描数据
        {
            let mut app_data = app_state.lock().unwrap();
            let scanned_data = ScannedData {
                id: signing_code.id.clone(),
                site_id: signing_code.site_id.clone(),
                create_time: signing_code.create_time.clone(),
                class_lesson_id: signing_code.class_lesson_id.clone(),
                scan_timestamp: Utc::now(),  // 记录扫描时间戳
            };
            
            // 将扫描数据存储到对应的class_lesson_id下
            app_data.scanned_data.insert(signing_code.class_lesson_id.clone(), scanned_data);
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

async fn get_qr_data(path: web::Path<String>, app_state: web::Data<AppState>) -> Result<HttpResponse> {
    let class_lesson_id = path.into_inner();
    
    // 获取应用状态中的扫描数据
    let app_data = app_state.lock().unwrap();
    
    // 检查是否有对应class_lesson_id的扫描数据
    let scanned_data = match app_data.scanned_data.get(&class_lesson_id) {
        Some(data) => data,
        None => {
            println!("尚未扫描过class_lesson_id={}的二维码，无法生成", class_lesson_id);
            let response = ApiResponse {
                status: "error".to_string(),
                message: "请先扫描二维码后再生成".to_string(),
            };
            return Ok(HttpResponse::BadRequest().json(response));
        }
    };
    
    // 检查扫描时间是否超过10分钟
    let current_time = Utc::now();
    let elapsed = current_time - scanned_data.scan_timestamp;
    
    if elapsed > chrono::Duration::minutes(10) {
        println!("二维码已过期（超过10分钟），需要重新扫描");
        // 注意：这里暂时不清除过期数据，只是返回错误
        let response = ApiResponse {
            status: "error".to_string(),
            message: "二维码已过期，请重新扫描".to_string(),
        };
        return Ok(HttpResponse::BadRequest().json(response));
    }
    
    // 解析扫描时间 (格式: 2025-06-04T09:52:14.04)
    let scanned_time_str = &scanned_data.create_time;
    
    // URL解码时间字符串（如果需要的话）
    let decoded_time_str = urlencoding::decode(scanned_time_str)
        .map_err(|e| {
            println!("URL解码错误: {}", e);
            e
        })
        .unwrap_or(std::borrow::Cow::Borrowed(scanned_time_str));
    
    // 尝试不同的时间格式进行解析
    let scanned_time = {
        // 常见的时间格式列表
        let formats = [
            "%Y-%m-%dT%H:%M:%S%.f",     // 2025-06-04T09:52:14.04
            "%Y-%m-%dT%H:%M:%S",        // 2025-06-04T09:52:14
            "%Y-%m-%d+%H:%M:%S%.f",     // 2025-06-04+09:52:14.04 (URL编码后的格式)
            "%Y-%m-%d+%H:%M:%S",        // 2025-06-04+09:52:14
            "%Y-%m-%d %H:%M:%S%.f",     // 2025-06-04 09:52:14.04
            "%Y-%m-%d %H:%M:%S",        // 2025-06-04 09:52:14
        ];
        
        let mut parsed_time = None;
        for format in &formats {
            match NaiveDateTime::parse_from_str(&decoded_time_str, format) {
                Ok(time) => {
                    parsed_time = Some(time);
                    break;
                }
                Err(_) => {
                    // 静默忽略解析错误，继续尝试下一个格式
                }
            }
        }
        
        match parsed_time {
            Some(time) => time,
            None => {
                println!("时间格式解析失败: '{}'", decoded_time_str);
                let response = ApiResponse {
                    status: "error".to_string(),
                    message: format!("时间格式解析失败: '{}'", decoded_time_str),
                };
                return Ok(HttpResponse::BadRequest().json(response));
            }
        }
    };
    
    // 获取当前时间（UTC+8时区）
    let utc_offset = chrono::FixedOffset::east_opt(8 * 3600).unwrap(); // UTC+8
    let current_time = Utc::now().with_timezone(&utc_offset).naive_local();
    
    // 计算最终时间：找到距离当前时间最近的过去时间，该时间是扫描时间+n*5秒
    let final_time = {
        let mut candidate_time = scanned_time;
        let five_seconds = chrono::Duration::seconds(5);
        
        // 循环增加5秒，直到超过当前时间
        while candidate_time <= current_time {
            candidate_time = candidate_time + five_seconds;
        }
        
        // 回退一个5秒间隔，得到最近的过去时间
        candidate_time - five_seconds
    };
    
    // 格式化最终时间，保持与原始格式一致 (3位小数)
    let formatted_time = final_time.format("%Y-%m-%dT%H:%M:%S");
    let nanoseconds = final_time.nanosecond();
    let centiseconds = nanoseconds / 10_000_000; // 转换为百分之一秒
    let new_time_str = format!("{}.{:03}", formatted_time, centiseconds);

    // 构造content字符串
    let content = format!("checkwork|id={}&siteId={}&createTime={}&classLessonId={}", 
        scanned_data.id, 
        scanned_data.site_id, 
        new_time_str, 
        scanned_data.class_lesson_id);
    
    println!("生成二维码数据: {}", content);
    
    let response = QrDataResponse {
        content,
    };
    
    Ok(HttpResponse::Ok().json(response))
}

// 获取所有有效的课程列表
async fn get_class_list(app_state: web::Data<AppState>) -> Result<HttpResponse> {
    let app_data = app_state.lock().unwrap();
    let current_time = Utc::now();
    
    let mut classes: Vec<ClassInfo> = Vec::new();
    
    for (class_lesson_id, scanned_data) in &app_data.scanned_data {
        let elapsed = current_time - scanned_data.scan_timestamp;
        let is_expired = elapsed > chrono::Duration::minutes(45);
        let time_remaining = if is_expired {
            0
        } else {
            let remaining_duration = chrono::Duration::minutes(45) - elapsed;
            remaining_duration.num_seconds().max(0)
        };
        
        let class_info = ClassInfo {
            class_lesson_id: class_lesson_id.clone(),
            class_name: get_class_name(class_lesson_id),
            id: scanned_data.id.clone(),
            site_id: scanned_data.site_id.clone(),
            scan_timestamp: scanned_data.scan_timestamp.format("%Y-%m-%d %H:%M:%S UTC").to_string(),
            is_expired,
            time_remaining,
        };
        
        classes.push(class_info);
    }
    
    // 按扫描时间排序（最新的在前）
    classes.sort_by(|a, b| b.scan_timestamp.cmp(&a.scan_timestamp));
    
    let response = ClassListResponse {
        classes,
    };
    
    Ok(HttpResponse::Ok().json(response))
}

// 根据class_lesson_id获取课程名称
async fn get_class_name_api(path: web::Path<String>) -> Result<HttpResponse> {
    let class_lesson_id = path.into_inner();
    let class_name = get_class_name(&class_lesson_id);
    
    let response = ClassNameResponse {
        class_lesson_id,
        class_name,
    };
    
    Ok(HttpResponse::Ok().json(response))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    
    // 初始化应用状态
    let app_state = web::Data::new(AppState::default());
    
    // 根据编译模式选择端口
    let port = if cfg!(debug_assertions) {
        2234 // debug模式使用2234端口
    } else {
        2233 // release模式使用2233端口
    };
    
    println!("启动服务器，端口: {}", port);
    
    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .wrap(Logger::default())
            .route("/", web::get().to(scan_page))
            .route("/selector", web::get().to(selector_page))
            .route("/gencode/{type}/{content}", web::get().to(generate_page))
            .route("/api/qr-code", web::post().to(submit_qr_code))
            .route("/api/qr-data/{class_lesson_id}", web::get().to(get_qr_data))
            .route("/api/class-list", web::get().to(get_class_list))
            .route("/api/class-name/{class_lesson_id}", web::get().to(get_class_name_api))
            .service(Files::new("/static", "./static").show_files_listing())
    })
    .bind(format!("0.0.0.0:{}", port))?
    .run()
    .await
}
