use crate::models::SigningCode;
use regex::Regex;
use crate::models::ClassData;
use crate::utils::time::get_formatted_time;

// 解析签到码内容
pub fn parse_signing_code(content: &str) -> Option<SigningCode> {
    let regex = Regex::new(r"checkwork\|id=(\d+)&siteId=(\d+)&createTime=([^&]+)&classLessonId=(\d+)").ok()?;
    if let Some(captures) = regex.captures(content) {
        Some(SigningCode {
            id: captures.get(1)?.as_str().to_string(),
            site_id: captures.get(2)?.as_str().to_string(),
            create_time: captures.get(3)?.as_str().to_string(),
            class_lesson_id: captures.get(4)?.as_str().to_string(),
        })
    } else {
        None
    }
}

pub fn format_signing_code(code: &ClassData) -> String {
    format!(
        "checkwork|id={}&siteId={}&createTime={}&classLessonId={}",
        code.last_checkwork_id.as_ref().unwrap_or(&"".to_string()),
        code.site_id,
        // code.last_created_time.as_ref().unwrap_or(&"".to_string()),
        get_formatted_time(),
        code.last_class_lesson_id.as_ref().unwrap_or(&"".to_string())
    )
}
