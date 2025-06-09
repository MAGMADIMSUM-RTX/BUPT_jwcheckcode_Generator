use regex::Regex;
use crate::models::SigningCode;

/// QR码解析器
pub struct QrCodeParser {
    signing_code_regex: Regex,
}

impl QrCodeParser {
    pub fn new() -> Self {
        let signing_code_regex = Regex::new(
            r"checkwork\|id=(\d+)&siteId=(\d+)&createTime=([^&]+)&classLessonId=(\d+)"
        ).expect("Invalid regex pattern");
        
        Self {
            signing_code_regex,
        }
    }
    
    /// 解析QR码内容并提取签名码信息
    pub fn parse(&self, content: &str) -> Option<SigningCode> {
        if let Some(captures) = self.signing_code_regex.captures(content) {
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
}

impl Default for QrCodeParser {
    fn default() -> Self {
        Self::new()
    }
}
