use actix_web::{web, App, HttpServer, middleware::Logger};
use actix_files::Files;
use std::sync::{Arc, Mutex};

use crate::models::AppData;
use crate::database::initialize_database;
use crate::handlers::{
    scan_page, selector_page, generate_page,
    submit_qr_code, get_qr_data, get_class_list, get_class_name_api, get_all_courses
};

/// 配置服务器路由
fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg
        .route("/", web::get().to(scan_page))
        .route("/selector", web::get().to(selector_page))
        .route("/gencode/{type}/{content}", web::get().to(generate_page))
        .route("/api/qr-code", web::post().to(submit_qr_code))
        .route("/api/qr-data/{class_lesson_id}", web::get().to(get_qr_data))
        .route("/api/class-list", web::get().to(get_class_list))
        .route("/api/class-name/{class_lesson_id}", web::get().to(get_class_name_api))
        .route("/api/all-courses", web::get().to(get_all_courses))
        .service(Files::new("/static", "./static").show_files_listing());
}

/// 启动服务器
pub async fn start_server() -> std::io::Result<()> {
    env_logger::init();
    
    // 初始化数据库连接
    let database_url = "sqlite:./lessons_data.db";
    let db_pool = initialize_database(database_url)
        .await
        .expect("Failed to initialize database");
    
    // 初始化应用状态
    let app_data = AppData::new(db_pool);
    let app_state = web::Data::new(Arc::new(Mutex::new(app_data)));
    
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
            .configure(configure_routes)
    })
    .bind(format!("0.0.0.0:{}", port))?
    .run()
    .await
}
