use attendance_common::{AppError, Result};
use attendance_domain::{OvertimeConfig, PaySlip, SalaryStructure};

pub struct PayrollCalculator;

impl PayrollCalculator {
    /// Calculate complete monthly salary based on employee attendance records
    pub fn calculate_monthly_payslip(
        salary: &SalaryStructure,
        ot_config: &OvertimeConfig,
        employee_name: &str,
        month_year: &str, // "2026-10"
        total_working_days: u32,
        present_days: u32,
        absent_days: u32,
        approved_leave_days: u32,
        holidays_count: u32,
        total_overtime_hours: f64,
        late_days_count: u32,
    ) -> Result<PaySlip> {
        if total_working_days == 0 {
            return Err(AppError::Validation(
                "Total working days cannot be zero".to_string(),
            ));
        }

        let per_day_base_rate = salary.basic_salary / total_working_days as f64;
        let per_hour_rate = per_day_base_rate / 8.0;

        // Base earnings: Basic + Allowances
        let allowances_total =
            salary.house_rent_allowance + salary.medical_allowance + salary.conveyance_allowance;
        let gross_standard = salary.basic_salary + allowances_total;

        // Overtime compensation
        let effective_ot_hours = total_overtime_hours.min(30.0 * ot_config.max_overtime_hours_per_day);
        let overtime_pay = effective_ot_hours * per_hour_rate * ot_config.hourly_rate_multiplier;

        // Deductions
        // 1. Absence deduction: unpaid absent days
        let absence_deduction = absent_days as f64 * per_day_base_rate;

        // 2. Late deduction policy: every 3 late arrivals incur half-day basic deduction
        let late_penalty_days = (late_days_count / 3) as f64 * 0.5;
        let late_deduction = late_penalty_days * per_day_base_rate;

        // Gross payable before statutory deductions
        let gross_payable = (gross_standard + overtime_pay - absence_deduction - late_deduction).max(0.0);

        // 3. Provident fund deduction: percentage of basic salary
        let pf_deduction = salary.basic_salary * salary.provident_fund_rate.clamp(0.0, 0.25);

        // 4. Tax deduction
        let tax_deduction = gross_payable * salary.tax_deduction_rate.clamp(0.0, 0.50);

        // Net payable
        let net_payable = (gross_payable - pf_deduction - tax_deduction).max(0.0);

        let payslip_id = format!("PS_{}_{}", salary.employee_code, month_year.replace('-', ""));

        Ok(PaySlip {
            payslip_id,
            employee_code: salary.employee_code.clone(),
            employee_name: employee_name.to_string(),
            month_year: month_year.to_string(),
            working_days: total_working_days,
            present_days,
            absent_days,
            leave_days: approved_leave_days,
            holidays: holidays_count,
            total_overtime_hours: effective_ot_hours,
            base_pay: salary.basic_salary,
            overtime_pay,
            allowances_total,
            late_deduction,
            absence_deduction,
            provident_fund_deduction: pf_deduction,
            tax_deduction,
            gross_payable,
            net_payable,
            generated_at: chrono::Utc::now().to_rfc3339(),
            status: "CALCULATED".to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_payslip_calculation() {
        let salary = SalaryStructure::default();
        let ot_config = OvertimeConfig::default();

        let payslip = PayrollCalculator::calculate_monthly_payslip(
            &salary,
            &ot_config,
            "Mohammad Rahim",
            "2026-10",
            22,
            22,
            0,
            0,
            4,
            10.0,
            1,
        )
        .unwrap();

        assert_eq!(payslip.present_days, 22);
        assert_eq!(payslip.absent_days, 0);
        assert!(payslip.gross_payable > 50000.0);
        assert!(payslip.net_payable > 45000.0);
    }
}
