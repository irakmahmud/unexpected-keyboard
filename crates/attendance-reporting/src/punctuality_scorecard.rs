use attendance_common::{AppError, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PunctualityScorecard {
    pub employee_code: String,
    pub employee_name: String,
    pub month_year: String,
    pub score: f32, // 0.0 to 100.0
    pub grade: String, // "A+", "A", "B", "C", "F"
    pub total_working_days: u32,
    pub on_time_days: u32,
    pub late_days: u32,
    pub early_exit_days: u32,
    pub unapproved_absence_days: u32,
    pub penalty_points: f32,
    pub bonus_points: f32,
}

pub struct PunctualityAnalyzer;

impl PunctualityAnalyzer {
    /// Calculate standard enterprise punctuality scorecard
    /// Base: 100 points
    /// Penalties:
    /// - Late arrival: -3 points per incident
    /// - Early exit: -2 points per incident
    /// - Unapproved absence: -10 points per day
    /// Bonus:
    /// - 100% on-time streak: +5 bonus points
    pub fn compute_scorecard(
        employee_code: &str,
        employee_name: &str,
        month_year: &str,
        working_days: u32,
        on_time: u32,
        late: u32,
        early_exit: u32,
        absent: u32,
    ) -> Result<PunctualityScorecard> {
        if working_days == 0 {
            return Err(AppError::Validation("Working days must be > 0".to_string()));
        }

        let penalty = (late as f32 * 3.0) + (early_exit as f32 * 2.0) + (absent as f32 * 10.0);
        let bonus = if late == 0 && early_exit == 0 && absent == 0 && on_time == working_days {
            5.0
        } else {
            0.0
        };

        let final_score = (100.0 - penalty + bonus).clamp(0.0, 105.0);

        let grade = if final_score >= 95.0 {
            "A+"
        } else if final_score >= 85.0 {
            "A"
        } else if final_score >= 75.0 {
            "B"
        } else if final_score >= 60.0 {
            "C"
        } else {
            "F"
        };

        Ok(PunctualityScorecard {
            employee_code: employee_code.to_string(),
            employee_name: employee_name.to_string(),
            month_year: month_year.to_string(),
            score: final_score,
            grade: grade.to_string(),
            total_working_days: working_days,
            on_time_days: on_time,
            late_days: late,
            early_exit_days: early_exit,
            unapproved_absence_days: absent,
            penalty_points: penalty,
            bonus_points: bonus,
        })
    }
}
