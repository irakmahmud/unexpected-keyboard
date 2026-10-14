use attendance_common::{AppError, Result};
use attendance_domain::Shift;
use chrono::NaiveTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RotationalShiftPattern {
    pub pattern_id: String,
    pub name: String,
    pub shift_sequence: Vec<String>, // Sequence of Shift codes e.g. ["MORNING", "MORNING", "EVENING", "EVENING", "NIGHT", "OFF", "OFF"]
    pub cycle_days: usize,
}

pub struct RosterScheduler;

impl RosterScheduler {
    /// Automatically identify the matching shift for a punch timestamp
    /// Matches the closest shift start time within a tolerance window
    pub fn match_shift_by_punch_time<'a>(
        punch_time: &str, // "08:45:00"
        candidate_shifts: &'a [Shift],
    ) -> Option<&'a Shift> {
        let punch_naive = NaiveTime::parse_from_str(punch_time, "%H:%M:%S").ok()?;

        let mut closest_shift: Option<&'a Shift> = None;
        let mut min_diff_minutes = i64::MAX;

        for shift in candidate_shifts {
            if !shift.is_active {
                continue;
            }

            if let Ok(start_naive) = NaiveTime::parse_from_str(&shift.start_time, "%H:%M:%S") {
                let diff = (punch_naive - start_naive).num_minutes().abs();
                if diff < min_diff_minutes {
                    min_diff_minutes = diff;
                    closest_shift = Some(shift);
                }
            }
        }

        // Only match if within 180 minutes (3 hours) of shift start
        if min_diff_minutes <= 180 {
            closest_shift
        } else {
            None
        }
    }

    /// Calculate which shift an employee should work on day N based on rotational pattern
    pub fn get_shift_for_day_in_rotation(
        pattern: &RotationalShiftPattern,
        day_offset_from_start: usize,
    ) -> Result<String> {
        if pattern.shift_sequence.is_empty() {
            return Err(AppError::Validation("Pattern sequence is empty".to_string()));
        }

        let idx = day_offset_from_start % pattern.shift_sequence.len();
        Ok(pattern.shift_sequence[idx].clone())
    }
}
