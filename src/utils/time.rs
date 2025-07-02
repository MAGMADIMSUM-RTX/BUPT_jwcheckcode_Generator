#[cfg(not(feature = "server"))]
pub fn format_current_time() -> String {
    use js_sys::Date;
    let now = Date::new_0();
    
    // 获取本地时间（不手动调整时区）
    let year = now.get_full_year() as u32;
    let month = (now.get_month() as u32) + 1;
    let day = now.get_date() as u32;
    let hours = now.get_hours() as u32;
    let minutes = now.get_minutes() as u32;
    let seconds = now.get_seconds() as u32;
    let milliseconds = now.get_milliseconds() as u32;
    
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:03}",
        year, month, day, hours, minutes, seconds, milliseconds
    )
}

pub fn get_formatted_time() -> String {
    #[cfg(feature = "server")]
    {
        use chrono::{FixedOffset, Utc};
        let china_tz = FixedOffset::east_opt(8 * 3600).unwrap();
        Utc::now()
            .with_timezone(&china_tz)
            .format("%Y-%m-%dT%H:%M:%S%.3f")
            .to_string()
    }
    #[cfg(not(feature = "server"))]
    {
        format_current_time()
    }
}

pub fn compare_formatted_time(time1: &str, time2: &str, gap_time: u64) -> bool {
    #[cfg(feature = "server")]
    {
        use chrono::{DateTime, FixedOffset};
        let china_tz = FixedOffset::east_opt(8 * 3600).unwrap();
        let datetime1 = match DateTime::parse_from_str(&format!("{}+08:00", time1), "%Y-%m-%dT%H:%M:%S%.3f%z") {
            Ok(dt) => dt.with_timezone(&china_tz),
            Err(_) => return false,
        };
        let datetime2 = match DateTime::parse_from_str(&format!("{}+08:00", time2), "%Y-%m-%dT%H:%M:%S%.3f%z") {
            Ok(dt) => dt.with_timezone(&china_tz),
            Err(_) => return false,
        };
        let duration = datetime2.signed_duration_since(datetime1);
        let diff_minutes = duration.num_minutes();
        diff_minutes >= 0 && diff_minutes <= gap_time as i64
    }
    #[cfg(not(feature = "server"))]
    {
        use js_sys::Date;
        let parse_time = |time_str: &str| -> Option<f64> {
            let parts: Vec<&str> = time_str.split('T').collect();
            if parts.len() != 2 { return None; }
            let date_parts: Vec<&str> = parts[0].split('-').collect();
            if date_parts.len() != 3 { return None; }
            let time_parts: Vec<&str> = parts[1].split(':').collect();
            if time_parts.len() != 3 { return None; }
            let seconds_parts: Vec<&str> = time_parts[2].split('.').collect();
            if seconds_parts.len() != 2 { return None; }
            let iso_string = format!("{}T{}Z", parts[0], parts[1]);
            let date = Date::new(&iso_string.into());
            let utc_time = date.get_time();
            let china_time = utc_time + 8.0 * 60.0 * 60.0 * 1000.0;
            Some(china_time)
        };
        let timestamp1 = match parse_time(time1) { Some(t) => t, None => return false, };
        let timestamp2 = match parse_time(time2) { Some(t) => t, None => return false, };
        let diff_ms = timestamp2 - timestamp1;
        let diff_minutes = (diff_ms / (1000.0 * 60.0)) as i64;
        diff_minutes >= 0 && diff_minutes <= gap_time as i64
    }
}
