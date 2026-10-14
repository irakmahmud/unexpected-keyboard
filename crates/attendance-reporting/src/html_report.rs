use crate::muster_roll::EmployeeMusterRow;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportMetadata {
    pub company_name: String,
    pub title: String,
    pub month_year: String,
    pub generated_at: String,
    pub generated_by: String,
}

pub struct HtmlReportRenderer;

impl HtmlReportRenderer {
    /// Render full-screen, print-optimized HTML muster roll table
    pub fn render_muster_roll_html(
        metadata: &ReportMetadata,
        rows: &[EmployeeMusterRow],
        days_in_month: u32,
    ) -> String {
        let mut days_headers = String::new();
        for d in 1..=days_in_month {
            days_headers.push_str(&format!("<th class='day-col'>{:02}</th>", d));
        }

        let mut table_rows = String::new();
        for (idx, row) in rows.iter().enumerate() {
            let mut days_cells = String::new();
            for d in 1..=days_in_month {
                let status = row.days.get(&d).cloned().unwrap_or_else(|| "-".to_string());
                let cell_class = match status.as_str() {
                    "P" => "status-p",
                    "L" => "status-l",
                    "A" => "status-a",
                    "LV" => "status-lv",
                    "W" => "status-w",
                    "H" => "status-h",
                    _ => "",
                };
                days_cells.push_str(&format!("<td class='day-cell {}'>{}</td>", cell_class, status));
            }

            table_rows.push_str(&format!(
                r#"<tr>
                    <td class="text-center">{}</td>
                    <td class="font-bold">{}</td>
                    <td>{}</td>
                    <td>{}</td>
                    {}
                    <td class="text-center font-bold status-p">{}</td>
                    <td class="text-center font-bold status-l">{}</td>
                    <td class="text-center font-bold status-a">{}</td>
                    <td class="text-center font-bold status-lv">{}</td>
                </tr>"#,
                idx + 1,
                row.employee_code,
                row.employee_name,
                row.department,
                days_cells,
                row.total_present,
                row.total_late,
                row.total_absent,
                row.total_leaves
            ));
        }

        format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{} - Muster Roll</title>
    <style>
        body {{
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
            margin: 20px;
            color: #1f2937;
            background: #fff;
        }}
        .header {{
            text-align: center;
            border-bottom: 2px solid #3b82f6;
            padding-bottom: 12px;
            margin-bottom: 16px;
        }}
        .company-name {{ font-size: 24px; font-weight: 800; color: #1e3a8a; margin: 0; }}
        .report-title {{ font-size: 16px; font-weight: 600; color: #4b5563; margin-top: 4px; }}
        .meta-bar {{
            display: flex;
            justify-content: space-between;
            font-size: 12px;
            color: #6b7280;
            margin-bottom: 12px;
        }}
        table {{
            width: 100%;
            border-collapse: collapse;
            font-size: 11px;
        }}
        th, td {{
            border: 1px solid #d1d5db;
            padding: 4px 6px;
            white-space: nowrap;
        }}
        th {{
            background: #f3f4f6;
            font-weight: 700;
            color: #374151;
        }}
        .day-col {{ width: 20px; text-align: center; }}
        .day-cell {{ text-align: center; font-weight: 600; font-size: 10px; }}
        .text-center {{ text-align: center; }}
        .font-bold {{ font-weight: bold; }}
        .status-p {{ color: #047857; background: #ecfdf5; }}
        .status-l {{ color: #b45309; background: #fffbeb; }}
        .status-a {{ color: #b91c1c; background: #fef2f2; }}
        .status-lv {{ color: #4338ca; background: #eef2ff; }}
        .status-w {{ color: #6b7280; background: #f9fafb; }}
        .status-h {{ color: #0891b2; background: #ecfeff; }}
        .legend {{
            display: flex;
            gap: 16px;
            margin-top: 16px;
            font-size: 11px;
            flex-wrap: wrap;
        }}
        .legend-item {{ display: flex; align-items: center; gap: 6px; }}
        .legend-box {{ width: 14px; height: 14px; border: 1px solid #ccc; border-radius: 2px; }}
        @media print {{
            body {{ margin: 0; }}
            @page {{ size: landscape; margin: 8mm; }}
            .no-print {{ display: none; }}
        }}
    </style>
</head>
<body>
    <div class="header">
        <h1 class="company-name">{}</h1>
        <div class="report-title">{} — {}</div>
    </div>
    <div class="meta-bar">
        <div><strong>Month:</strong> {}</div>
        <div><strong>Generated At:</strong> {}</div>
        <div><strong>Generated By:</strong> {}</div>
    </div>

    <table id="muster-table">
        <thead>
            <tr>
                <th rowspan="2">#</th>
                <th rowspan="2">Emp Code</th>
                <th rowspan="2">Name</th>
                <th rowspan="2">Dept</th>
                <th colspan="{}">Days of Month</th>
                <th colspan="4">Summary</th>
            </tr>
            <tr>
                {}
                <th>P</th>
                <th>L</th>
                <th>A</th>
                <th>LV</th>
            </tr>
        </thead>
        <tbody>
            {}
        </tbody>
    </table>

    <div class="legend">
        <div class="legend-item"><div class="legend-box status-p"></div> P: Present</div>
        <div class="legend-item"><div class="legend-box status-l"></div> L: Late</div>
        <div class="legend-item"><div class="legend-box status-a"></div> A: Absent</div>
        <div class="legend-item"><div class="legend-box status-lv"></div> LV: Leave</div>
        <div class="legend-item"><div class="legend-box status-w"></div> W: Weekend</div>
        <div class="legend-item"><div class="legend-box status-h"></div> H: Holiday</div>
    </div>
</body>
</html>"#,
            metadata.title,
            metadata.company_name,
            metadata.title,
            metadata.month_year,
            metadata.month_year,
            metadata.generated_at,
            metadata.generated_by,
            days_in_month,
            days_headers,
            table_rows
        )
    }
}
