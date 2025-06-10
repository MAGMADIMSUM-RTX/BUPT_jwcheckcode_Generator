use chrono::{FixedOffset, NaiveDateTime, Timelike, Utc};
use urlencoding;

/// 时间处理工具
pub struct TimeProcessor;

impl TimeProcessor {
    /// 解析时间字符串，支持多种格式
    pub fn parse_time_string(time_str: &str) -> Result<NaiveDateTime, String> {
        // URL解码时间字符串
        let decoded_time_str =
            urlencoding::decode(time_str).map_err(|e| format!("URL解码错误: {}", e))?;

        // 支持的时间格式列表
        let formats = [
            "%Y-%m-%dT%H:%M:%S%.f", // 2025-06-04T09:52:14.04
            "%Y-%m-%dT%H:%M:%S",    // 2025-06-04T09:52:14
            "%Y-%m-%d+%H:%M:%S%.f", // 2025-06-04+09:52:14.04 (URL编码后的格式)
            "%Y-%m-%d+%H:%M:%S",    // 2025-06-04+09:52:14
            "%Y-%m-%d %H:%M:%S%.f", // 2025-06-04 09:52:14.04
            "%Y-%m-%d %H:%M:%S",    // 2025-06-04 09:52:14
        ];

        for format in &formats {
            if let Ok(time) = NaiveDateTime::parse_from_str(&decoded_time_str, format) {
                return Ok(time);
            }
        }

        Err(format!("时间格式解析失败: '{}'", decoded_time_str))
    }

    /// 计算最终时间：找到距离当前时间最近的过去时间，该时间是扫描时间+n*5秒
    pub fn calculate_final_time(_scanned_time: NaiveDateTime) -> NaiveDateTime {
        // 获取当前时间（UTC+8时区）
        let utc_offset = FixedOffset::east_opt(8 * 3600).unwrap(); // UTC+8
        let current_time = Utc::now().with_timezone(&utc_offset).naive_local();

        // let mut candidate_time = scanned_time;
        // let five_seconds = chrono::Duration::seconds(5);
        let one_seconds = chrono::Duration::seconds(1);

        // // 循环增加5秒，直到超过当前时间
        // while candidate_time <= current_time {
        //     candidate_time = candidate_time + five_seconds;
        // }

        // // // 回退一个5秒间隔，得到最近的过去时间
        // // candidate_time - five_seconds
        // candidate_time - five_seconds + one_seconds

        current_time + one_seconds
    }

    /// 格式化时间为指定格式
    pub fn format_time_with_centiseconds(time: NaiveDateTime) -> String {
        let formatted_time = time.format("%Y-%m-%dT%H:%M:%S");
        let nanoseconds = time.nanosecond();
        let centiseconds = nanoseconds / 10_000_000; // 转换为百分之一秒
        format!("{}.{:03}", formatted_time, centiseconds)
    }
}
