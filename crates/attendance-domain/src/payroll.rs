use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SalaryStructure {
    pub employee_code: String,
    pub basic_salary: f64,
    pub house_rent_allowance: f64,
    pub medical_allowance: f64,
    pub conveyance_allowance: f64,
    pub gross_salary: f64,
    pub provident_fund_rate: f64,
    pub tax_deduction_rate: f64,
    pub currency: String,
}

impl Default for SalaryStructure {
    fn default() -> Self {
        Self {
            employee_code: String::new(),
            basic_salary: 30000.0,
            house_rent_allowance: 15000.0,
            medical_allowance: 3000.0,
            conveyance_allowance: 2000.0,
            gross_salary: 50000.0,
            provident_fund_rate: 0.05,
            tax_deduction_rate: 0.0,
            currency: "BDT".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OvertimeConfig {
    pub hourly_rate_multiplier: f64,
    pub holiday_rate_multiplier: f64,
    pub weekend_rate_multiplier: f64,
    pub minimum_minutes_to_qualify: u32,
    pub max_overtime_hours_per_day: f64,
}

impl Default for OvertimeConfig {
    fn default() -> Self {
        Self {
            hourly_rate_multiplier: 1.5,
            holiday_rate_multiplier: 2.0,
            weekend_rate_multiplier: 2.0,
            minimum_minutes_to_qualify: 30,
            max_overtime_hours_per_day: 4.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaySlip {
    pub payslip_id: String,
    pub employee_code: String,
    pub employee_name: String,
    pub month_year: String,
    pub working_days: u32,
    pub present_days: u32,
    pub absent_days: u32,
    pub leave_days: u32,
    pub holidays: u32,
    pub total_overtime_hours: f64,
    pub base_pay: f64,
    pub overtime_pay: f64,
    pub allowances_total: f64,
    pub late_deduction: f64,
    pub absence_deduction: f64,
    pub provident_fund_deduction: f64,
    pub tax_deduction: f64,
    pub gross_payable: f64,
    pub net_payable: f64,
    pub generated_at: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayrollRun {
    pub run_id: String,
    pub month: u32,
    pub year: i32,
    pub company_id: String,
    pub total_employees: usize,
    pub total_gross_disbursed: f64,
    pub total_net_disbursed: f64,
    pub total_overtime_cost: f64,
    pub total_deductions: f64,
    pub execution_timestamp: String,
    pub finalized: bool,
}
