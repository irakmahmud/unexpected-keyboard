use attendance_common::TimeHelper;
use attendance_domain::{AttendanceStatus, Shift};
use chrono::NaiveTime;

#[derive(Debug, Clone)]
pub struct ShiftEvaluation {
    pub status: AttendanceStatus,
    pub late_minutes: u32,
    pub early_exit_minutes: u32,
    pub work_hours: f64,
    pub overtime_hours: f64,
    pub is_half_day: bool,
}

pub struct ShiftCalculator;

impl ShiftCalculator {
    /// Evaluate attendance times against shift configuration
    pub fn evaluate_attendance(
        in_time_str: Option<&str>,
        out_time_str: Option<&str>,
        shift: &Shift,
    ) -> ShiftEvaluation {
        let mut late_minutes = 0u32;
        let mut early_exit_minutes = 0u32;
        let mut work_hours = 0.0f64;
        let mut overtime_hours = 0.0f64;

        let shift_start = TimeHelper::parse_time(&shift.start_time)
            .unwrap_or_else(|_| NaiveTime::from_hms_opt(9, 0, 0).unwrap());
        let shift_end = TimeHelper::parse_time(&shift.end_time)
            .unwrap_or_else(|_| NaiveTime::from_hms_opt(17, 0, 0).unwrap());

        let crosses_midnight = shift_end < shift_start;

        // 1. Evaluate In-Time (Lateness)
        if let Some(in_t_str) = in_time_str {
            if let Ok(actual_in) = TimeHelper::parse_time(in_t_str) {
                let diff_minutes = if crosses_midnight {
                    if actual_in > shift_start {
                        // Arrived after shift_start on the same evening
                        (actual_in - shift_start).num_minutes()
                    } else if actual_in <= shift_end {
                        // Arrived after midnight into the next morning
                        (actual_in - shift_start).num_minutes() + (24 * 60)
                    } else {
                        // Arrived before shift_start on the same evening (on-time / early)
                        0
                    }
                } else if actual_in > shift_start {
                    (actual_in - shift_start).num_minutes()
                } else {
                    0
                };

                if diff_minutes > shift.grace_period_minutes as i64 {
                    late_minutes = diff_minutes as u32;
                }
            }
        }

        // 2. Evaluate Out-Time & Total Duration
        if let (Some(in_t_str), Some(out_t_str)) = (in_time_str, out_time_str) {
            if let (Ok(actual_in), Ok(actual_out)) = (TimeHelper::parse_time(in_t_str), TimeHelper::parse_time(out_t_str)) {
                let total_minutes = if crosses_midnight && actual_out < actual_in {
                    (actual_out - actual_in).num_minutes() + (24 * 60)
                } else if actual_out >= actual_in {
                    (actual_out - actual_in).num_minutes()
                } else {
                    0
                };

                if total_minutes > 0 {
                    work_hours = (total_minutes as f64) / 60.0;
                }

                // Check early departure and overtime
                if crosses_midnight {
                    if actual_out >= shift_start {
                        // Punched out before midnight (very early departure from night shift)
                        let exit_diff = (shift_end - actual_out).num_minutes() + (24 * 60);
                        if exit_diff > 10 {
                            early_exit_minutes = exit_diff as u32;
                        }
                    } else if actual_out < shift_end {
                        // Punched out after midnight before shift_end in the morning
                        let exit_diff = (shift_end - actual_out).num_minutes();
                        if exit_diff > 10 {
                            early_exit_minutes = exit_diff as u32;
                        }
                    } else if shift.is_overtime_enabled {
                        let ot_diff = (actual_out - shift_end).num_minutes();
                        if ot_diff >= shift.overtime_starts_after_minutes as i64 {
                            overtime_hours = (ot_diff as f64) / 60.0;
                        }
                    }
                } else {
                    // Regular Daytime Shift
                    if actual_out < shift_end {
                        let exit_diff = (shift_end - actual_out).num_minutes();
                        if exit_diff > 10 {
                            early_exit_minutes = exit_diff as u32;
                        }
                    } else if shift.is_overtime_enabled {
                        let ot_diff = (actual_out - shift_end).num_minutes();
                        if ot_diff >= shift.overtime_starts_after_minutes as i64 {
                            overtime_hours = (ot_diff as f64) / 60.0;
                        }
                    }
                }
            }
        }

        // 3. Determine Overall Status
        let is_half_day = work_hours > 0.0 && work_hours < shift.half_day_threshold_hours;

        let status = if is_half_day {
            AttendanceStatus::HalfDay
        } else if late_minutes >= 60 {
            // Arriving 1+ hour late (>= 10:00 AM for standard 9am shift) is categorized as Late Absent (Python parity lines 7300-7305)
            AttendanceStatus::LateAbsent
        } else if late_minutes > 0 {
            AttendanceStatus::Late
        } else if early_exit_minutes > 10 {
            AttendanceStatus::EarlyDeparture
        } else if in_time_str.is_some() {
            AttendanceStatus::Present
        } else {
            AttendanceStatus::Absent
        };

        ShiftEvaluation {
            status,
            late_minutes,
            early_exit_minutes,
            work_hours,
            overtime_hours,
            is_half_day,
        }
    }
}
