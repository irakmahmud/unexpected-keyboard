use attendance_common::{Result, TimeHelper};
use attendance_database::DbPool;
use attendance_reporting::{HtmlReportRenderer, MusterRollGenerator, ReportMetadata};
use std::fs;

pub struct ExportCommand;

impl ExportCommand {
    pub fn execute(db: &DbPool, month_year: Option<&str>, format: Option<&str>) -> Result<()> {
        let target_month = month_year.unwrap_or("2026-10");
        let export_format = format.unwrap_or("html");

        println!("============================================================");
        println!("             ATTENDANCE MUSTER ROLL BATCH EXPORT            ");
        println!("============================================================");
        println!("Target Month:  {}", target_month);
        println!("Target Format: {}", export_format);

        let parts: Vec<&str> = target_month.split('-').collect();
        let year: i32 = parts.first().and_then(|y| y.parse().ok()).unwrap_or(2026);
        let month: u32 = parts.get(1).and_then(|m| m.parse().ok()).unwrap_or(10);

        let generator = MusterRollGenerator::new(db.clone());
        let rows = generator.generate_monthly_muster_roll(year, month, "TECHFIRM_01")?;

        fs::create_dir_all("./exports")?;

        let days_in_month = match month {
            4 | 6 | 9 | 11 => 30,
            2 => if (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0) { 29 } else { 28 },
            _ => 31,
        };

        match export_format {
            "html" => {
                let out_file = format!("./exports/muster_roll_{}.html", target_month);
                let meta = ReportMetadata {
                    company_name: "TechFirm Global Limited".to_string(),
                    title: format!("Monthly Form-12 Muster Roll - {}", target_month),
                    month_year: target_month.to_string(),
                    generated_at: TimeHelper::now_utc_sqlite_string(),
                    generated_by: "attendance-admin CLI".to_string(),
                };
                let html = HtmlReportRenderer::render_muster_roll_html(&meta, &rows, days_in_month);
                fs::write(&out_file, html)?;
                println!("✓ Successfully generated print-ready HTML muster roll:");
                println!("  Path: {}", out_file);
            }
            "csv" => {
                let out_file = format!("./exports/muster_roll_{}.csv", target_month);
                let mut csv_data = String::from("EmployeeCode,FullName,Department,PresentDays,LateDays,AbsentDays,LeaveDays\n");
                for row in &rows {
                    csv_data.push_str(&format!(
                        "{},\"{}\",\"{}\",{},{},{},{}\n",
                        row.employee_code,
                        row.employee_name,
                        row.department,
                        row.total_present,
                        row.total_late,
                        row.total_absent,
                        row.total_leaves
                    ));
                }
                fs::write(&out_file, csv_data)?;
                println!("✓ Successfully generated CSV muster roll:");
                println!("  Path: {}", out_file);
            }
            other => {
                eprintln!("Unsupported format '{}'. Available formats: html, csv", other);
            }
        }

        println!("============================================================");
        Ok(())
    }
}
