pub mod pages;
pub mod api;

// 重新导出所有处理函数，使其可以从 handlers 模块直接访问
pub use pages::{scan_page, selector_page, generate_page};
pub use api::{submit_qr_code, get_qr_data, get_class_list, get_class_name_api, get_all_courses};
