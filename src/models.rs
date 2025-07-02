use serde::{Deserialize, Serialize};

// 签到码结构体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SigningCode {
    pub id: String,
    pub site_id: String,
    pub create_time: String,
    pub class_lesson_id: String,
}

// 课程数据结构体
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ClassData {
    pub site_id: String,
    pub class_name: String,
    pub classes: String,
    pub last_checkwork_id: Option<String>,
    pub last_class_lesson_id: Option<String>,
    pub last_created_time: Option<String>,
    pub is_expired: bool,
}



// #[derive(PartialEq)]
// pub enum CodeGenOptions {
//     Name(String),
//     Id(String),
// }