use crate::error::{AppError, Result};
use chrono::{DateTime, Datelike, Duration, FixedOffset, NaiveDate, NaiveDateTime, NaiveTime, Timelike, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeValidationResult {
    pub is_valid: bool,
    pub drift_seconds: i64,
    pub server_time: String,
    pub client_time: Option<String>,
    pub warning: Option<String>,
}

#[derive(Debug, Clone)]
pub struct TimeHelper;

impl TimeHelper {
    /// Asia/Dhaka offset (+06:00) exactly as in Python pytz.timezone('Asia/Dhaka')
    pub fn dhaka_offset() -> FixedOffset {
        FixedOffset::east_opt(6 * 3600).expect("Valid Dhaka fixed offset (+06:00)")
    }

    /// Current UTC ISO string formatted like SQLite strftime('%Y-%m-%d %H:%M:%S', 'now', 'utc')
    pub fn now_utc_sqlite_string() -> String {
        Utc::now().format("%Y-%m-%d %H:%M:%S").to_string()
    }

    /// Current date in Asia/Dhaka as YYYY-MM-DD
    pub fn current_date_dhaka() -> String {
        let utc_now = Utc::now();
        let dhaka_time = utc_now.with_timezone(&Self::dhaka_offset());
        dhaka_time.format("%Y-%m-%d").to_string()
    }

    /// Current time in Asia/Dhaka as HH:MM:SS
    pub fn current_time_dhaka() -> String {
        let utc_now = Utc::now();
        let dhaka_time = utc_now.with_timezone(&Self::dhaka_offset());
        dhaka_time.format("%H:%M:%S").to_string()
    }

    /// Full local datetime string in Asia/Dhaka (YYYY-MM-DD HH:MM:SS)
    pub fn current_datetime_dhaka() -> String {
        let utc_now = Utc::now();
        let dhaka_time = utc_now.with_timezone(&Self::dhaka_offset());
        dhaka_time.format("%Y-%m-%d %H:%M:%S").to_string()
    }

    /// Convert UTC ISO string or timestamp to local Dhaka display string
    pub fn utc_to_dhaka_string(utc_str: &str) -> Result<String> {
        let parsed = NaiveDateTime::parse_from_str(utc_str, "%Y-%m-%d %H:%M:%S")
            .map_err(|e| AppError::Validation(format!("Invalid timestamp format: {}", e)))?;
        let utc_dt = DateTime::<Utc>::from_naive_utc_and_offset(parsed, Utc);
        let dhaka_dt = utc_dt.with_timezone(&Self::dhaka_offset());
        Ok(dhaka_dt.format("%Y-%m-%d %H:%M:%S").to_string())
    }

    /// Parse date string in format YYYY-MM-DD
    pub fn parse_date(date_str: &str) -> Result<NaiveDate> {
        let trimmed = date_str.trim();
        // If datetime string with space: "2026-10-12 09:15:30"
        let date_part = if let Some(first) = trimmed.split_whitespace().next() {
            first
        } else {
            trimmed
        };
        // If RFC3339 with 'T'
        let clean_date = date_part.split('T').next().unwrap_or(date_part);
        NaiveDate::parse_from_str(clean_date, "%Y-%m-%d")
            .map_err(|e| AppError::Validation(format!("Invalid date format (expected YYYY-MM-DD): {}", e)))
    }

    /// Generate record ID exactly matching Python attendance_v8.py line 5040:
    /// f"{company_user_uuid}_{date.day}_{date.strftime('%b')}_{date.year}"
    pub fn generate_record_id(company_user_uuid: &str, date_str: &str) -> String {
        if let Ok(date) = Self::parse_date(date_str) {
            format!("{}_{}_{}_{}", company_user_uuid, date.day(), date.format("%b"), date.year())
        } else {
            format!("{}_{}", company_user_uuid, date_str)
        }
    }

    /// Parse time string in format HH:MM:SS, HH:MM, 12-hour AM/PM, or full timestamp
    pub fn parse_time(time_str: &str) -> Result<NaiveTime> {
        let trimmed = time_str.trim();
        if trimmed.is_empty() {
            return Err(AppError::Validation("Empty time string".to_string()));
        }

        // Handle 12-hour AM/PM formats
        if trimmed.ends_with("AM") || trimmed.ends_with("PM") || trimmed.ends_with("am") || trimmed.ends_with("pm") {
            if let Ok(t) = NaiveTime::parse_from_str(trimmed, "%I:%M %p") {
                return Ok(t);
            }
            if let Ok(t) = NaiveTime::parse_from_str(trimmed, "%I:%M:%S %p") {
                return Ok(t);
            }
            // Pad single-digit hour like "9:15 AM" -> "09:15 AM"
            if let Some(pos) = trimmed.find(':') {
                if pos == 1 {
                    let padded = format!("0{}", trimmed);
                    if let Ok(t) = NaiveTime::parse_from_str(&padded, "%I:%M %p") {
                        return Ok(t);
                    }
                    if let Ok(t) = NaiveTime::parse_from_str(&padded, "%I:%M:%S %p") {
                        return Ok(t);
                    }
                }
            }
        }

        // If full ISO/datetime string with space: "2026-10-12 09:15:30"
        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if parts.len() >= 2 {
            if parts.len() == 3 && (parts[2].eq_ignore_ascii_case("AM") || parts[2].eq_ignore_ascii_case("PM")) {
                let combined = format!("{} {}", parts[1], parts[2]);
                if let Ok(t) = NaiveTime::parse_from_str(&combined, "%I:%M %p") {
                    return Ok(t);
                }
                if let Ok(t) = NaiveTime::parse_from_str(&combined, "%I:%M:%S %p") {
                    return Ok(t);
                }
            }
            return Self::parse_time(parts[1]);
        }

        // If full RFC3339 with 'T': "2026-10-12T09:15:30"
        if trimmed.contains('T') {
            if let Some(time_part) = trimmed.split('T').nth(1) {
                let clean_time = time_part.split('+').next().unwrap_or(time_part).trim_end_matches('Z');
                return Self::parse_time(clean_time);
            }
        }

        // Normalize single-digit hour for 24-hour formats (e.g. "9:15" -> "09:15", "9:15:00" -> "09:15:00")
        let norm_str = if let Some(pos) = trimmed.find(':') {
            if pos == 1 {
                format!("0{}", trimmed)
            } else {
                trimmed.to_string()
            }
        } else {
            trimmed.to_string()
        };

        if norm_str.len() == 5 {
            NaiveTime::parse_from_str(&norm_str, "%H:%M")
                .map_err(|e| AppError::Validation(format!("Invalid time format (expected HH:MM): {}", e)))
        } else {
            let first_8 = if norm_str.len() >= 8 { &norm_str[..8] } else { &norm_str };
            NaiveTime::parse_from_str(first_8, "%H:%M:%S")
                .map_err(|e| AppError::Validation(format!("Invalid time format (expected HH:MM:SS): {}", e)))
        }
    }

    /// Format time string to 12-hour AM/PM format (e.g. "09:30 AM") matching Python report
    pub fn format_12hour(time_str: &str) -> String {
        if let Ok(t) = Self::parse_time(time_str) {
            t.format("%I:%M %p").to_string()
        } else {
            "-".to_string()
        }
    }

    /// Calculate total days in a given month handling leap years
    pub fn days_in_month(year: i32, month: u32) -> u32 {
        match month {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 30,
            2 => {
                if (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0) {
                    29
                } else {
                    28
                }
            }
            _ => 30,
        }
    }

    /// Parse month name or number ("October", "Oct", "10", 10) into 1..=12 matching Python line 12943
    pub fn parse_month(month_str: &str) -> Option<u32> {
        let trimmed = month_str.trim().to_lowercase();
        if let Ok(num) = trimmed.parse::<u32>() {
            if (1..=12).contains(&num) {
                return Some(num);
            }
        }
        match trimmed.as_str() {
            "jan" | "january" => Some(1),
            "feb" | "february" => Some(2),
            "mar" | "march" => Some(3),
            "apr" | "april" => Some(4),
            "may" => Some(5),
            "jun" | "june" => Some(6),
            "jul" | "july" => Some(7),
            "aug" | "august" => Some(8),
            "sep" | "september" => Some(9),
            "oct" | "october" => Some(10),
            "nov" | "november" => Some(11),
            "dec" | "december" => Some(12),
            _ => None,
        }
    }

    /// Calculate elapsed minutes between two HH:MM:SS times on the same date
    pub fn elapsed_minutes(start_time: &str, end_time: &str) -> Result<i64> {
        let t1 = Self::parse_time(start_time)?;
        let t2 = Self::parse_time(end_time)?;
        let duration = t2 - t1;
        Ok(duration.num_minutes())
    }

    /// Calculate elapsed hours as f64 between two times
    pub fn elapsed_hours(start_time: &str, end_time: &str) -> Result<f64> {
        let mins = Self::elapsed_minutes(start_time, end_time)?;
        Ok(mins as f64 / 60.0)
    }

    /// Check if a date string is a weekend (Friday or Saturday in Bangladesh)
    pub fn is_weekend_in_dhaka(date_str: &str) -> Result<bool> {
        let date = Self::parse_date(date_str)?;
        let weekday = date.weekday();
        // Standard weekend in Bangladesh: Friday (and optionally Saturday)
        Ok(weekday == chrono::Weekday::Fri || weekday == chrono::Weekday::Sat)
    }

    /// Get list of dates between start_date and end_date inclusive
    pub fn get_date_range(start_date: &str, end_date: &str) -> Result<Vec<String>> {
        let start = Self::parse_date(start_date)?;
        let end = Self::parse_date(end_date)?;
        if start > end {
            return Err(AppError::Validation("Start date cannot be after end date".to_string()));
        }

        let mut current = start;
        let mut dates = Vec::new();
        while current <= end {
            dates.push(current.format("%Y-%m-%d").to_string());
            current += Duration::days(1);
        }
        Ok(dates)
    }

    /// Validate client punch timestamp against authoritative server time (anti-clock-tampering)
    pub fn validate_client_punch_time(client_time_str: Option<&str>, allowed_drift_secs: i64) -> TimeValidationResult {
        let now = Utc::now();
        let server_str = now.with_timezone(&Self::dhaka_offset()).format("%Y-%m-%d %H:%M:%S").to_string();

        if let Some(c_time) = client_time_str {
            if let Ok(parsed_naive) = NaiveDateTime::parse_from_str(c_time, "%Y-%m-%d %H:%M:%S") {
                let parsed_utc = DateTime::<Utc>::from_naive_utc_and_offset(parsed_naive, Utc);
                let diff = (now - parsed_utc).num_seconds().abs();
                if diff > allowed_drift_secs {
                    return TimeValidationResult {
                        is_valid: false,
                        drift_seconds: diff,
                        server_time: server_str,
                        client_time: Some(c_time.to_string()),
                        warning: Some(format!("Time drift exceeded: {} seconds difference from server clock", diff)),
                    };
                }
            }
        }

        TimeValidationResult {
            is_valid: true,
            drift_seconds: 0,
            server_time: server_str,
            client_time: client_time_str.map(|s| s.to_string()),
            warning: None,
        }
    }

    /// Generate media filepath matching Python _save_frame (line 2575) and _generate_blank_image (line 8669):
    /// "{username}/{year}/{MonthName}/{USERNAME}_{DD}_{Mon}_{YYYY}_{LOGIN_TYPE}_{HH}_{MM}_{AM/PM}.jpg"
    pub fn generate_media_filepath(username: &str, login_type: &str, date_str: &str, time_str: &str) -> String {
        let upper_user = username.to_uppercase();
        let date = Self::parse_date(date_str).unwrap_or_else(|_| NaiveDate::from_ymd_opt(2026, 10, 14).unwrap());
        let time = Self::parse_time(time_str).unwrap_or_else(|_| NaiveTime::from_hms_opt(9, 0, 0).unwrap());
        let year = date.year();
        let month_full = date.format("%B").to_string();
        let date_part = date.format("%d_%b_%Y").to_string();
        let time_part = time.format("%I_%M_%p").to_string();
        format!(
            "data/{}/{}/{}/{}_{}_{}_{}.jpg",
            upper_user, year, month_full, upper_user, date_part, login_type, time_part
        )
    }

    /// Parse date from filename matching Python parse_filename_date (line 13370):
    /// Example format: "USERNAME_DD_MMM_YYYY_TYPE_HH_MM_AM/PM.jpg"
    pub fn parse_filename_date(filename: &str) -> Option<NaiveDate> {
        let base = filename.rsplit('/').next().unwrap_or(filename);
        let parts: Vec<&str> = base.split('_').collect();
        if parts.len() >= 4 {
            let date_str = format!("{} {} {}", parts[1], parts[2], parts[3]);
            NaiveDate::parse_from_str(&date_str, "%d %b %Y").ok()
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dhaka_date_and_time() {
        let d = TimeHelper::current_date_dhaka();
        assert_eq!(d.len(), 10);
        let t = TimeHelper::current_time_dhaka();
        assert_eq!(t.len(), 8);
    }

    #[test]
    fn test_elapsed_time_calculations() {
        let mins = TimeHelper::elapsed_minutes("09:00:00", "17:30:00").unwrap();
        assert_eq!(mins, 510);
        let hours = TimeHelper::elapsed_hours("09:00:00", "17:30:00").unwrap();
        assert!((hours - 8.5).abs() < 1e-4);
    }

    #[test]
    fn test_date_range() {
        let range = TimeHelper::get_date_range("2026-10-01", "2026-10-03").unwrap();
        assert_eq!(range, vec!["2026-10-01", "2026-10-02", "2026-10-03"]);
    }

    #[test]
    fn test_python_record_id_parity() {
        let rec_id = TimeHelper::generate_record_id("CU_12345", "2026-10-12");
        assert_eq!(rec_id, "CU_12345_12_Oct_2026");

        let rec_id_single_day = TimeHelper::generate_record_id("CU_12345", "2026-10-05");
        assert_eq!(rec_id_single_day, "CU_12345_5_Oct_2026");
    }

    #[test]
    fn test_time_parsing_single_digit_and_am_pm() {
        let t1 = TimeHelper::parse_time("9:15").unwrap();
        assert_eq!(t1.hour(), 9);
        assert_eq!(t1.minute(), 15);

        let t2 = TimeHelper::parse_time("9:15:30").unwrap();
        assert_eq!(t2.hour(), 9);
        assert_eq!(t2.minute(), 15);
        assert_eq!(t2.second(), 30);

        let t3 = TimeHelper::parse_time("9:15 AM").unwrap();
        assert_eq!(t3.hour(), 9);
        assert_eq!(t3.minute(), 15);

        let t4 = TimeHelper::parse_time("05:30 PM").unwrap();
        assert_eq!(t4.hour(), 17);
        assert_eq!(t4.minute(), 30);

        assert_eq!(TimeHelper::format_12hour("17:30:00"), "05:30 PM");
        assert_eq!(TimeHelper::format_12hour(""), "-");
    }

    #[test]
    fn test_days_in_month_and_month_parsing() {
        assert_eq!(TimeHelper::days_in_month(2026, 10), 31);
        assert_eq!(TimeHelper::days_in_month(2026, 2), 28);
        assert_eq!(TimeHelper::days_in_month(2024, 2), 29); // Leap year
        assert_eq!(TimeHelper::days_in_month(2026, 4), 30);

        assert_eq!(TimeHelper::parse_month("October"), Some(10));
        assert_eq!(TimeHelper::parse_month("oct"), Some(10));
        assert_eq!(TimeHelper::parse_month("10"), Some(10));
        assert_eq!(TimeHelper::parse_month("invalid"), None);
    }
}
