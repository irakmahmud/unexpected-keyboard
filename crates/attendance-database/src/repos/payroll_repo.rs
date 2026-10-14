use attendance_common::{AppError, Result};
use attendance_domain::{PaySlip, SalaryStructure};
use rusqlite::{params, Connection};

pub struct PayrollRepo;

impl PayrollRepo {
    pub fn save_salary_structure(conn: &Connection, salary: &SalaryStructure) -> Result<()> {
        conn.execute(
            "INSERT INTO salary_structures 
             (employee_code, basic_salary, house_rent_allowance, medical_allowance, conveyance_allowance, 
              gross_salary, provident_fund_rate, tax_deduction_rate, currency, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, strftime('%Y-%m-%d %H:%M:%S', 'now', 'utc'))
             ON CONFLICT(employee_code) DO UPDATE SET
              basic_salary = excluded.basic_salary,
              house_rent_allowance = excluded.house_rent_allowance,
              medical_allowance = excluded.medical_allowance,
              conveyance_allowance = excluded.conveyance_allowance,
              gross_salary = excluded.gross_salary,
              provident_fund_rate = excluded.provident_fund_rate,
              tax_deduction_rate = excluded.tax_deduction_rate,
              currency = excluded.currency,
              updated_at = excluded.updated_at",
            params![
                salary.employee_code,
                salary.basic_salary,
                salary.house_rent_allowance,
                salary.medical_allowance,
                salary.conveyance_allowance,
                salary.gross_salary,
                salary.provident_fund_rate,
                salary.tax_deduction_rate,
                salary.currency
            ],
        )
        .map_err(|e| AppError::Database(format!("Failed to save salary structure: {}", e)))?;

        Ok(())
    }

    pub fn get_salary_structure(
        conn: &Connection,
        employee_code: &str,
    ) -> Result<Option<SalaryStructure>> {
        let mut stmt = conn
            .prepare(
                "SELECT employee_code, basic_salary, house_rent_allowance, medical_allowance, 
                        conveyance_allowance, gross_salary, provident_fund_rate, tax_deduction_rate, currency
                 FROM salary_structures WHERE employee_code = ?1",
            )
            .map_err(|e| AppError::Database(e.to_string()))?;

        let mut rows = stmt
            .query_map(params![employee_code], |row| {
                Ok(SalaryStructure {
                    employee_code: row.get(0)?,
                    basic_salary: row.get(1)?,
                    house_rent_allowance: row.get(2)?,
                    medical_allowance: row.get(3)?,
                    conveyance_allowance: row.get(4)?,
                    gross_salary: row.get(5)?,
                    provident_fund_rate: row.get(6)?,
                    tax_deduction_rate: row.get(7)?,
                    currency: row.get(8)?,
                })
            })
            .map_err(|e| AppError::Database(e.to_string()))?;

        if let Some(r) = rows.next() {
            Ok(Some(r.map_err(|e| AppError::Database(e.to_string()))?))
        } else {
            Ok(None)
        }
    }

    pub fn save_payslip(conn: &Connection, slip: &PaySlip) -> Result<()> {
        conn.execute(
            "INSERT INTO payslips (
                payslip_id, employee_code, employee_name, month_year, working_days,
                present_days, absent_days, leave_days, holidays, total_overtime_hours,
                base_pay, overtime_pay, allowances_total, late_deduction, absence_deduction,
                provident_fund_deduction, tax_deduction, gross_payable, net_payable,
                generated_at, status
             ) VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21
             )
             ON CONFLICT(payslip_id) DO UPDATE SET
                working_days = excluded.working_days,
                present_days = excluded.present_days,
                absent_days = excluded.absent_days,
                leave_days = excluded.leave_days,
                holidays = excluded.holidays,
                total_overtime_hours = excluded.total_overtime_hours,
                base_pay = excluded.base_pay,
                overtime_pay = excluded.overtime_pay,
                gross_payable = excluded.gross_payable,
                net_payable = excluded.net_payable,
                status = excluded.status",
            params![
                slip.payslip_id,
                slip.employee_code,
                slip.employee_name,
                slip.month_year,
                slip.working_days,
                slip.present_days,
                slip.absent_days,
                slip.leave_days,
                slip.holidays,
                slip.total_overtime_hours,
                slip.base_pay,
                slip.overtime_pay,
                slip.allowances_total,
                slip.late_deduction,
                slip.absence_deduction,
                slip.provident_fund_deduction,
                slip.tax_deduction,
                slip.gross_payable,
                slip.net_payable,
                slip.generated_at,
                slip.status
            ],
        )
        .map_err(|e| AppError::Database(format!("Failed to save payslip: {}", e)))?;

        Ok(())
    }

    pub fn get_payslip(
        conn: &Connection,
        employee_code: &str,
        month_year: &str,
    ) -> Result<Option<PaySlip>> {
        let mut stmt = conn
            .prepare(
                "SELECT payslip_id, employee_code, employee_name, month_year, working_days,
                        present_days, absent_days, leave_days, holidays, total_overtime_hours,
                        base_pay, overtime_pay, allowances_total, late_deduction, absence_deduction,
                        provident_fund_deduction, tax_deduction, gross_payable, net_payable,
                        generated_at, status
                 FROM payslips WHERE employee_code = ?1 AND month_year = ?2",
            )
            .map_err(|e| AppError::Database(e.to_string()))?;

        let mut rows = stmt
            .query_map(params![employee_code, month_year], |row| {
                Ok(PaySlip {
                    payslip_id: row.get(0)?,
                    employee_code: row.get(1)?,
                    employee_name: row.get(2)?,
                    month_year: row.get(3)?,
                    working_days: row.get(4)?,
                    present_days: row.get(5)?,
                    absent_days: row.get(6)?,
                    leave_days: row.get(7)?,
                    holidays: row.get(8)?,
                    total_overtime_hours: row.get(9)?,
                    base_pay: row.get(10)?,
                    overtime_pay: row.get(11)?,
                    allowances_total: row.get(12)?,
                    late_deduction: row.get(13)?,
                    absence_deduction: row.get(14)?,
                    provident_fund_deduction: row.get(15)?,
                    tax_deduction: row.get(16)?,
                    gross_payable: row.get(17)?,
                    net_payable: row.get(18)?,
                    generated_at: row.get(19)?,
                    status: row.get(20)?,
                })
            })
            .map_err(|e| AppError::Database(e.to_string()))?;

        if let Some(r) = rows.next() {
            Ok(Some(r.map_err(|e| AppError::Database(e.to_string()))?))
        } else {
            Ok(None)
        }
    }
}
