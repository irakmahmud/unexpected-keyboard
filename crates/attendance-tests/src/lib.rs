use attendance_common::{Result, TimeHelper};
use attendance_database::DbPool;
use attendance_domain::*;

pub struct TestFixtures;

impl TestFixtures {
    /// Create a fully initialized in-memory SQLite database for testing
    pub fn setup_in_memory_db() -> Result<DbPool> {
        let pool = DbPool::new_in_memory()?;
        Self::seed_base_data(&pool)?;
        Ok(pool)
    }

    /// Seed baseline company, shifts, and departments
    pub fn seed_base_data(db: &DbPool) -> Result<()> {
        let now = TimeHelper::now_utc_sqlite_string();

        let company = CompanyInfo {
            company_id: "TECHFIRM_01".to_string(),
            company_name: "TechFirm Global Ltd".to_string(),
            password: "SecurePassword123".to_string(),
            address_road: Some("Road 11, Block D".to_string()),
            address_city: Some("Dhaka".to_string()),
            contact_person_name: Some("MD Director".to_string()),
            contact_person_designation: Some("Director".to_string()),
            contact_person_number: Some("+8801700000000".to_string()),
            website: Some("https://techfirm.com".to_string()),
            email: Some("contact@techfirm.com".to_string()),
            tax_id: Some("TIN-12345678".to_string()),
            created_at: Some(now.clone()),
            last_updated: Some(now.clone()),
        };
        db.save_company(&company)?;

        // Seed 3 standard shifts
        let morning_shift = Shift {
            id: None,
            company_id: "TECHFIRM_01".to_string(),
            name: "Morning General Shift".to_string(),
            code: "MORN_09_18".to_string(),
            shift_type: ShiftType::RegularDay,
            start_time: "09:00:00".to_string(),
            end_time: "18:00:00".to_string(),
            grace_period_minutes: 15,
            half_day_threshold_hours: 4.0,
            min_work_hours_full_day: 9.0,
            break_duration_minutes: 60,
            is_overtime_enabled: true,
            overtime_starts_after_minutes: 15,
            is_active: true,
            created_at: Some(now.clone()),
        };
        db.create_shift(&morning_shift)?;

        let evening_shift = Shift {
            id: None,
            company_id: "TECHFIRM_01".to_string(),
            name: "Night Cross-Midnight Shift".to_string(),
            code: "NIGHT_20_05".to_string(),
            shift_type: ShiftType::NightCrossMidnight,
            start_time: "20:00:00".to_string(),
            end_time: "05:00:00".to_string(),
            grace_period_minutes: 10,
            half_day_threshold_hours: 4.0,
            min_work_hours_full_day: 9.0,
            break_duration_minutes: 60,
            is_overtime_enabled: true,
            overtime_starts_after_minutes: 15,
            is_active: true,
            created_at: Some(now.clone()),
        };
        db.create_shift(&evening_shift)?;

        let flex_shift = Shift {
            id: None,
            company_id: "TECHFIRM_01".to_string(),
            name: "Flexible Core Hours".to_string(),
            code: "FLEX_10_19".to_string(),
            shift_type: ShiftType::Flexible,
            start_time: "10:00:00".to_string(),
            end_time: "19:00:00".to_string(),
            grace_period_minutes: 30,
            half_day_threshold_hours: 4.0,
            min_work_hours_full_day: 9.0,
            break_duration_minutes: 60,
            is_overtime_enabled: true,
            overtime_starts_after_minutes: 30,
            is_active: true,
            created_at: Some(now.clone()),
        };
        db.create_shift(&flex_shift)?;

        // Seed departments
        let engineering = Department {
            id: None,
            company_id: "TECHFIRM_01".to_string(),
            name: "Engineering & R&D".to_string(),
            code: "ENG".to_string(),
            description: Some("Software and Embedded Systems".to_string()),
            head_employee_id: None,
            created_at: Some(now.clone()),
        };
        db.save_department(&engineering)?;

        let hr_dept = Department {
            id: None,
            company_id: "TECHFIRM_01".to_string(),
            name: "Human Resources".to_string(),
            code: "HR".to_string(),
            description: Some("People and Operations".to_string()),
            head_employee_id: None,
            created_at: Some(now.clone()),
        };
        db.save_department(&hr_dept)?;

        let finance_dept = Department {
            id: None,
            company_id: "TECHFIRM_01".to_string(),
            name: "Finance & Accounts".to_string(),
            code: "FIN".to_string(),
            description: Some("Accounts and Payroll".to_string()),
            head_employee_id: None,
            created_at: Some(now),
        };
        db.save_department(&finance_dept)?;

        // Seed default designation
        let conn = db.get_connection();
        conn.execute(
            "INSERT INTO designations (id, company_id, title, grade, department_id, description)
             VALUES (1, 'TECHFIRM_01', 'Software Engineer', 'L3', 1, 'Core engineering')",
            [],
        ).map_err(|e| attendance_common::AppError::Database(e.to_string()))?;

        Ok(())
    }

    /// Create and persist a test employee
    pub fn create_test_employee(db: &DbPool, code: &str, first_name: &str, last_name: &str) -> Result<Employee> {
        let now = TimeHelper::now_utc_sqlite_string();
        let emp = Employee {
            id: None,
            employee_code: code.to_uppercase(),
            user_uuid: uuid::Uuid::new_v4().to_string(),
            company_id: "TECHFIRM_01".to_string(),
            first_name: first_name.to_string(),
            last_name: last_name.to_string(),
            full_name: format!("{} {}", first_name, last_name),
            email: Some(format!("{}.{}@techfirm.com", first_name.to_lowercase(), last_name.to_lowercase())),
            phone: Some("+8801812345678".to_string()),
            department_id: Some(1),
            designation_id: Some(1),
            shift_id: Some(1),
            rfid_card_uid: Some(format!("RFID_{}", code)),
            pin_code: Some("1234".to_string()),
            employment_type: EmploymentType::Permanent,
            join_date: "2024-01-01".to_string(),
            end_date: None,
            is_active: true,
            avatar_url: None,
            emergency_contact_name: Some("Emergency Contact".to_string()),
            emergency_contact_phone: Some("+8801900000000".to_string()),
            created_at: Some(now.clone()),
            updated_at: Some(now.clone()),
        };

        db.save_employee(&emp)?;

        let user = User {
            company_user_uuid: format!("UUID_{}", code),
            company_name: Some("TechFirm Global Ltd".to_string()),
            company_uuid: Some("TECHFIRM_01".to_string()),
            username: code.to_uppercase(),
            user_uuid: Some(emp.user_uuid.clone()),
            user_designation: Some("Software Engineer".to_string()),
            user_android_id: None,
            face_encoding: None,
            password: None,
            role: UserRole::Employee,
            status: UserStatus::Active,
            email: emp.email.clone(),
            phone: emp.phone.clone(),
            created_at: Some(now.clone()),
            last_updated: Some(now),
        };
        db.insert_or_update_user(&user)?;

        Ok(emp)
    }

    /// Generate a 512-dimensional synthetic normalized vector for face matching
    pub fn generate_synthetic_embedding(seed: u32) -> Vec<f32> {
        let mut vec = Vec::with_capacity(512);
        let mut norm_sq = 0.0f32;
        for i in 0..512 {
            let val = ((seed.wrapping_mul(31).wrapping_add(i as u32) % 1000) as f32 / 1000.0) - 0.5;
            vec.push(val);
            norm_sq += val * val;
        }

        let norm = norm_sq.sqrt().max(1e-9);
        for v in &mut vec {
            *v /= norm;
        }
        vec
    }
}
