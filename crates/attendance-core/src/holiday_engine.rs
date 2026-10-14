use attendance_domain::Holiday;
use chrono::{Datelike, NaiveDate, Weekday};

pub struct HolidayEngine;

impl HolidayEngine {
    /// Check if a given date falls on a weekend based on company policy
    pub fn is_weekend(date_str: &str, weekend_days: &[Weekday]) -> bool {
        if let Ok(date) = NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
            let weekday = date.weekday();
            weekend_days.contains(&weekday)
        } else {
            false
        }
    }

    /// Bangladesh standard weekend: Friday and Saturday
    pub fn is_bangladesh_weekend(date_str: &str) -> bool {
        Self::is_weekend(date_str, &[Weekday::Fri, Weekday::Sat])
    }

    /// Check if a date is a listed gazetted holiday
    pub fn is_gazetted_holiday(date_str: &str, holidays: &[Holiday]) -> Option<String> {
        for h in holidays {
            if h.holiday_date == date_str {
                return Some(h.name.clone());
            }
        }
        None
    }

    /// Evaluate the "Sandwich Rule":
    /// If an employee takes leave on the working day immediately preceding AND immediately
    /// following a holiday or weekend, the holiday/weekend days are counted as leave.
    pub fn evaluate_sandwich_rule(
        holiday_or_weekend_date: &str,
        leave_dates: &[String],
    ) -> bool {
        if let Ok(date) = NaiveDate::parse_from_str(holiday_or_weekend_date, "%Y-%m-%d") {
            let prev_day = date.pred_opt().map(|d| d.format("%Y-%m-%d").to_string());
            let next_day = date.succ_opt().map(|d| d.format("%Y-%m-%d").to_string());

            if let (Some(prev), Some(next)) = (prev_day, next_day) {
                let has_prev_leave = leave_dates.contains(&prev);
                let has_next_leave = leave_dates.contains(&next);
                return has_prev_leave && has_next_leave;
            }
        }
        false
    }

    /// Calculate effective working days in a month given weekends and holidays
    pub fn calculate_working_days(
        year: i32,
        month: u32,
        holidays: &[Holiday],
        weekend_days: &[Weekday],
    ) -> u32 {
        let mut count = 0;
        let mut day = 1;

        while let Some(date) = NaiveDate::from_ymd_opt(year, month, day) {
            let date_str = date.format("%Y-%m-%d").to_string();
            let is_wknd = weekend_days.contains(&date.weekday());
            let is_hld = holidays.iter().any(|h| h.holiday_date == date_str);

            if !is_wknd && !is_hld {
                count += 1;
            }
            day += 1;
        }

        count
    }
}
