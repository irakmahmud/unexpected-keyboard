use attendance_common::Result;
use attendance_database::DbPool;

pub struct CsvExporter {
    db: DbPool,
}

impl CsvExporter {
    pub fn new(db: DbPool) -> Self {
        Self { db }
    }

    pub fn export_date_range_csv(&self, start_date: &str, end_date: &str) -> Result<String> {
        let dates = attendance_common::TimeHelper::get_date_range(start_date, end_date)?;
        let mut csv = String::from("Username,Date,In_Time,Out_Time,Status,Work_Hours,Overtime_Hours,Late_Minutes,In_Camera,In_QR\n");

        for d in dates {
            let records = self.db.get_attendance_by_date(&d)?;
            for r in records {
                csv.push_str(&format!(
                    "{},{},{},{},{:?},{:.2},{:.2},{},{},{}\n",
                    r.username,
                    r.date,
                    r.in_time.as_deref().unwrap_or(""),
                    r.out_time.as_deref().unwrap_or(""),
                    r.status,
                    r.work_hours,
                    r.overtime_hours,
                    r.late_minutes,
                    r.in_time_with_camera.as_deref().unwrap_or("N"),
                    r.in_time_with_qr.as_deref().unwrap_or("N"),
                ));
            }
        }

        Ok(csv)
    }

    /// Generate customized attendance matrix CSV exactly matching Python attendance_v8.py lines 12985-13126:
    /// Headers: User ID, Username, [Day - In/Out/Camera/Late/Early], Summary totals
    pub fn export_customized_attendance_matrix_csv(
        &self,
        username_filter: Option<&str>,
        year: i32,
        month: u32,
        from_day: u32,
        to_day: u32,
        in_time_threshold: &str,
        out_time_threshold: &str,
    ) -> Result<String> {
        let set_in_time = attendance_common::TimeHelper::parse_time(in_time_threshold)
            .unwrap_or_else(|_| chrono::NaiveTime::from_hms_opt(9, 0, 0).unwrap());
        let set_out_time = attendance_common::TimeHelper::parse_time(out_time_threshold)
            .unwrap_or_else(|_| chrono::NaiveTime::from_hms_opt(17, 0, 0).unwrap());

        let max_days = attendance_common::TimeHelper::days_in_month(year, month);
        let from_day_clamped = from_day.clamp(1, max_days);
        let to_day_clamped = to_day.clamp(from_day_clamped, max_days);

        let start_date = format!("{:04}-{:02}-{:02}", year, month, from_day_clamped);
        let end_date = format!("{:04}-{:02}-{:02}", year, month, to_day_clamped);

        let all_users = self.db.get_all_users()?;
        let users: Vec<_> = match username_filter {
            Some(u) if !u.trim().is_empty() && !u.eq_ignore_ascii_case("All Users") => {
                all_users.into_iter().filter(|usr| usr.username.eq_ignore_ascii_case(u)).collect()
            }
            _ => all_users,
        };

        // Determine working days in period (distinct dates where anyone had in_time or out_time)
        let all_period_records = self.db.get_attendance_records_by_range(&start_date, &end_date, None)?;
        let mut active_dates = std::collections::HashSet::new();
        for r in &all_period_records {
            if r.in_time.is_some() || r.out_time.is_some() {
                active_dates.insert(r.date.clone());
            }
        }
        let working_days = active_dates.len().max(1);
        let total_office_hours = working_days * 8;

        // Build CSV Header
        let mut csv = String::new();
        csv.push_str("User ID,Username");
        for day in from_day_clamped..=to_day_clamped {
            csv.push_str(&format!(
                ",{0} - In Time,{0} - Out Time,{0} - Camera In,{0} - Camera Out,{0} - Late,{0} - Early Leave",
                day
            ));
        }
        csv.push_str(",Total Lates,Total Early Leaves,Total In Time With Camera,Total Out Time With Camera,Working Days in Period,Total Office Hours Required,Number of Absent Days,Actual Office Stay (Hours)\n");

        for u in users {
            let user_records = self.db.get_attendance_records_by_range(&start_date, &end_date, Some(&u.username))?;
            let record_map: std::collections::HashMap<String, _> = user_records
                .into_iter()
                .map(|r| (r.date.clone(), r))
                .collect();

            let mut total_lates = 0usize;
            let mut total_early_leaves = 0usize;
            let mut total_in_camera = 0usize;
            let mut total_out_camera = 0usize;
            let mut total_actual_hours = 0.0f64;
            let mut present_days = 0usize;

            let mut row_days_csv = String::new();

            for day in from_day_clamped..=to_day_clamped {
                let d_str = format!("{:04}-{:02}-{:02}", year, month, day);
                if let Some(r) = record_map.get(&d_str) {
                    let in_t_parsed = r.in_time.as_deref().and_then(|t| attendance_common::TimeHelper::parse_time(t).ok());
                    let out_t_parsed = r.out_time.as_deref().and_then(|t| attendance_common::TimeHelper::parse_time(t).ok());

                    let in_time_display = r.in_time.as_deref().map(attendance_common::TimeHelper::format_12hour).unwrap_or_else(|| "-".to_string());
                    let out_time_display = r.out_time.as_deref().map(attendance_common::TimeHelper::format_12hour).unwrap_or_else(|| "-".to_string());

                    let in_cam = r.in_time_with_camera.as_deref().unwrap_or("N");
                    let out_cam = r.out_time_with_camera.as_deref().unwrap_or("N");

                    if in_cam == "Y" { total_in_camera += 1; }
                    if out_cam == "Y" { total_out_camera += 1; }

                    let is_late = if let Some(t_in) = in_t_parsed {
                        if t_in > set_in_time {
                            total_lates += 1;
                            "Yes"
                        } else {
                            "No"
                        }
                    } else {
                        "No"
                    };

                    let is_early = if let Some(t_out) = out_t_parsed {
                        if t_out < set_out_time {
                            total_early_leaves += 1;
                            "Yes"
                        } else {
                            "No"
                        }
                    } else {
                        "No"
                    };

                    if let (Some(t_in), Some(t_out)) = (in_t_parsed, out_t_parsed) {
                        present_days += 1;
                        let diff_mins = if t_out >= t_in {
                            (t_out - t_in).num_minutes()
                        } else {
                            (t_out - t_in).num_minutes() + (24 * 60)
                        };
                        if diff_mins > 0 {
                            total_actual_hours += diff_mins as f64 / 60.0;
                        }
                    }

                    row_days_csv.push_str(&format!(
                        ",{},{},{},{},{},{}",
                        in_time_display, out_time_display, in_cam, out_cam, is_late, is_early
                    ));
                } else {
                    // Matching Python attendance_v8.py line 13109: ws.cell(row=row, column=col + i, value='-')
                    row_days_csv.push_str(",-,-,-,-,-,-");
                }
            }

            let absent_days = working_days.saturating_sub(present_days);

            csv.push_str(&format!(
                "{},{}{},{},{},{},{},{},{} hours,{},{:.2} hours\n",
                u.company_user_uuid,
                u.username,
                row_days_csv,
                total_lates,
                total_early_leaves,
                total_in_camera,
                total_out_camera,
                working_days,
                total_office_hours,
                absent_days,
                total_actual_hours
            ));
        }

        Ok(csv)
    }
}
