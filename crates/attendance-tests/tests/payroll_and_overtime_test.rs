use attendance_core::PayrollCalculator;
use attendance_domain::*;
use attendance_tests::TestFixtures;

#[test]
fn test_salary_structure_and_allowances() {
    let db = TestFixtures::setup_in_memory_db().expect("DB setup failed");
    let emp = TestFixtures::create_test_employee(&db, "EMP_PAY_001", "Tanvir", "Hasan")
        .expect("Employee creation failed");

    let salary_struct = SalaryStructure {
        employee_code: emp.employee_code.clone(),
        basic_salary: 50000.0,
        house_rent_allowance: 25000.0,
        medical_allowance: 5000.0,
        conveyance_allowance: 3000.0,
        gross_salary: 83000.0,
        provident_fund_rate: 0.05,
        tax_deduction_rate: 0.03,
        currency: "BDT".to_string(),
    };

    db.save_salary_structure(&salary_struct).expect("Salary structure save failed");

    let fetched = db
        .get_salary_structure(&emp.employee_code)
        .expect("Query failed")
        .expect("Salary structure should exist");

    assert_eq!(fetched.basic_salary, 50000.0);
    assert_eq!(fetched.gross_salary, 83000.0);
    assert_eq!(fetched.currency, "BDT");
}

#[test]
fn test_payroll_full_monthly_payslip_calculation() {
    let salary = SalaryStructure {
        employee_code: "EMP_PAY_002".to_string(),
        basic_salary: 60000.0,
        house_rent_allowance: 30000.0,
        medical_allowance: 5000.0,
        conveyance_allowance: 5000.0,
        gross_salary: 100000.0,
        provident_fund_rate: 0.05,
        tax_deduction_rate: 0.05,
        currency: "BDT".to_string(),
    };

    let ot_config = OvertimeConfig::default();

    // In a 30-day month with 2 days unexcused absence, 3 late days, 10 hours overtime
    let payslip = PayrollCalculator::calculate_monthly_payslip(
        &salary,
        &ot_config,
        "Karim Ahmed",
        "2026-10",
        30,
        28,
        2,  // 2 absent days
        0,
        4,
        10.0, // 10 hrs overtime
        3,    // 3 late days (triggers 0.5 day deduction)
    ).expect("Calculation failed");

    assert_eq!(payslip.employee_code, "EMP_PAY_002");
    assert_eq!(payslip.present_days, 28);
    assert!(payslip.absence_deduction > 0.0);
    assert!(payslip.late_deduction > 0.0);
    assert!(payslip.overtime_pay > 0.0);
    assert!(payslip.net_payable > 0.0);
}

#[test]
fn test_payslip_generation_and_persistence() {
    let db = TestFixtures::setup_in_memory_db().expect("DB setup failed");
    let emp = TestFixtures::create_test_employee(&db, "EMP_PAY_003", "Anisur", "Rahman")
        .expect("Employee creation failed");

    let payslip = PaySlip {
        payslip_id: "PS-2026-10-EMP003".to_string(),
        employee_code: emp.employee_code.clone(),
        employee_name: emp.full_name.clone(),
        month_year: "2026-10".to_string(),
        working_days: 26,
        present_days: 24,
        absent_days: 2,
        leave_days: 0,
        holidays: 4,
        total_overtime_hours: 8.5,
        base_pay: 40000.0,
        overtime_pay: 3500.0,
        allowances_total: 20000.0,
        late_deduction: 1538.46,
        absence_deduction: 3076.92,
        provident_fund_deduction: 3200.0,
        tax_deduction: 1500.0,
        gross_payable: 63500.0,
        net_payable: 54184.62,
        generated_at: "2026-11-01T00:00:00Z".to_string(),
        status: "GENERATED".to_string(),
    };

    db.save_payslip(&payslip).expect("Payslip save failed");

    let retrieved = db
        .get_payslip(&emp.employee_code, "2026-10")
        .expect("Query failed")
        .expect("Payslip should exist");

    assert_eq!(retrieved.month_year, "2026-10");
    assert!((retrieved.net_payable - 54184.62).abs() < 0.01);
}
