use attendance_auth::PasswordHasher;
use attendance_common::{Result, TimeHelper};
use attendance_database::DbPool;
use attendance_domain::*;

pub struct SeedCommand;

impl SeedCommand {
    pub fn execute(db: &DbPool) -> Result<()> {
        println!("============================================================");
        println!("              SEEDING ENTERPRISE DEMO DATA                  ");
        println!("============================================================");

        let now = TimeHelper::now_utc_sqlite_string();

        // 1. Seed Company
        println!("[1/6] Seeding company record...");
        let company = CompanyInfo {
            company_id: "TECHFIRM_01".to_string(),
            company_name: "TechFirm Global Limited".to_string(),
            password: "SecurePassword123".to_string(),
            address_road: Some("Gulshan-2, Road 11".to_string()),
            address_city: Some("Dhaka 1212, Bangladesh".to_string()),
            contact_person_name: Some("Managing Director".to_string()),
            contact_person_designation: Some("Director".to_string()),
            contact_person_number: Some("+880 2 988 1234".to_string()),
            website: Some("https://techfirm.com".to_string()),
            email: Some("contact@techfirm.com".to_string()),
            tax_id: Some("TIN-88991122".to_string()),
            created_at: Some(now.clone()),
            last_updated: Some(now.clone()),
        };
        db.save_company(&company)?;
        println!("      ✓ Company 'TechFirm Global Limited' seeded.");

        // 2. Seed Departments
        println!("[2/6] Seeding 5 corporate departments...");
        let departments = [
            ("Engineering & Software", "ENG"),
            ("Human Resources & Admin", "HR"),
            ("Finance & Accounts", "FIN"),
            ("Operations & Logistics", "OPS"),
            ("Sales & Marketing", "MKT"),
        ];
        for (name, code) in departments {
            let dept = Department {
                id: None,
                company_id: "TECHFIRM_01".to_string(),
                name: name.to_string(),
                code: code.to_string(),
                description: Some(format!("Department for {}", name)),
                head_employee_id: None,
                created_at: Some(now.clone()),
            };
            db.save_department(&dept)?;
        }
        println!("      ✓ 5 departments registered.");

        // 2b. Seed Designations
        {
            let conn = db.get_connection();
            let desigs = [
                (1, "Senior Software Engineer", "L4", 1),
                (2, "HR Executive", "L2", 2),
                (3, "Senior Accountant", "L3", 3),
                (4, "Operations Supervisor", "L3", 4),
                (5, "Marketing Specialist", "L2", 5),
            ];
            for (id, title, grade, dept_id) in desigs {
                let _ = conn.execute(
                    "INSERT OR IGNORE INTO designations (id, company_id, title, grade, department_id, description)
                     VALUES (?1, 'TECHFIRM_01', ?2, ?3, ?4, 'Standard designation')",
                    rusqlite::params![id, title, grade, dept_id],
                );
            }
        }
        println!("      ✓ 5 corporate designations seeded.");

        // 3. Seed Shifts
        println!("[3/6] Seeding 3 standard corporate shifts...");
        let shifts: [(&str, &str, ShiftType, &str, &str, u32, f64, f64); 3] = [
            ("Standard Day Shift", "DAY_09_18", ShiftType::RegularDay, "09:00:00", "18:00:00", 15, 4.0, 9.0),
            ("Overnight Shift", "NIGHT_21_06", ShiftType::NightCrossMidnight, "21:00:00", "06:00:00", 10, 4.0, 9.0),
            ("Core Flexible Hours", "FLEX_10_19", ShiftType::Flexible, "10:00:00", "19:00:00", 30, 4.0, 9.0),
        ];

        for (s_name, s_code, s_type, s_start, s_end, grace, half, full) in shifts {
            let s = Shift {
                id: None,
                company_id: "TECHFIRM_01".to_string(),
                name: s_name.to_string(),
                code: s_code.to_string(),
                shift_type: s_type,
                start_time: s_start.to_string(),
                end_time: s_end.to_string(),
                grace_period_minutes: grace,
                half_day_threshold_hours: half,
                min_work_hours_full_day: full,
                break_duration_minutes: 60,
                is_overtime_enabled: true,
                overtime_starts_after_minutes: 15,
                is_active: true,
                created_at: Some(now.clone()),
            };
            db.create_shift(&s)?;
        }
        println!("      ✓ 3 shift templates created.");

        // 4. Seed Users
        println!("[4/6] Seeding administrative user accounts...");
        let admin_hash = PasswordHasher::hash_sha256("Admin#2026");
        let hr_hash = PasswordHasher::hash_sha256("Hr#Secure2026");

        let admin_user = User {
            company_user_uuid: "ADMIN_TECH_001".to_string(),
            company_name: Some("TechFirm Global Limited".to_string()),
            company_uuid: Some("TECHFIRM_01".to_string()),
            username: "ADMIN".to_string(),
            user_uuid: Some(uuid::Uuid::new_v4().to_string()),
            user_designation: Some("System Administrator".to_string()),
            user_android_id: None,
            face_encoding: None,
            register_with_camera: Some("N".to_string()),
            password: Some(admin_hash),
            role: UserRole::SuperAdmin,
            status: UserStatus::Active,
            email: Some("admin@techfirm.com".to_string()),
            phone: Some("+8801700000001".to_string()),
            created_at: Some(now.clone()),
            last_updated: Some(now.clone()),
        };
        db.insert_or_update_user(&admin_user)?;

        let hr_user = User {
            company_user_uuid: "HR_TECH_002".to_string(),
            company_name: Some("TechFirm Global Limited".to_string()),
            company_uuid: Some("TECHFIRM_01".to_string()),
            username: "HR_MANAGER".to_string(),
            user_uuid: Some(uuid::Uuid::new_v4().to_string()),
            user_designation: Some("HR Director".to_string()),
            user_android_id: None,
            face_encoding: None,
            register_with_camera: Some("N".to_string()),
            password: Some(hr_hash),
            role: UserRole::HrManager,
            status: UserStatus::Active,
            email: Some("hr@techfirm.com".to_string()),
            phone: Some("+8801700000002".to_string()),
            created_at: Some(now.clone()),
            last_updated: Some(now.clone()),
        };
        db.insert_or_update_user(&hr_user)?;
        println!("      ✓ Admin and HR accounts created with SHA-256 security.");

        // 5. Seed Employees
        println!("[5/6] Seeding 10 enterprise employee records...");
        let employees = [
            ("EMP001", "Tanvir", "Ahmed", "ENG", 1, "RFID_A001"),
            ("EMP002", "Nusrat", "Jahan", "ENG", 1, "RFID_A002"),
            ("EMP003", "Kazi", "Mamun", "HR", 1, "RFID_A003"),
            ("EMP004", "Farhana", "Akter", "FIN", 1, "RFID_A004"),
            ("EMP005", "Mahmudul", "Hasan", "ENG", 3, "RFID_A005"),
            ("EMP006", "Sultana", "Razia", "OPS", 2, "RFID_A006"),
            ("EMP007", "Ashraful", "Islam", "MKT", 1, "RFID_A007"),
            ("EMP008", "Rashed", "Karim", "OPS", 2, "RFID_A008"),
            ("EMP009", "Mehedi", "Hasan", "ENG", 1, "RFID_A009"),
            ("EMP010", "Sabina", "Yasmin", "FIN", 1, "RFID_A010"),
        ];

        for (code, first, last, _dept, shift_id, rfid) in employees {
            let full = format!("{} {}", first, last);
            let emp = Employee {
                id: None,
                employee_code: code.to_string(),
                user_uuid: uuid::Uuid::new_v4().to_string(),
                company_id: "TECHFIRM_01".to_string(),
                first_name: first.to_string(),
                last_name: last.to_string(),
                full_name: full,
                email: Some(format!("{}.{}@techfirm.com", first.to_lowercase(), last.to_lowercase())),
                phone: Some("+880 1711 000000".to_string()),
                department_id: Some(1),
                designation_id: Some(1),
                shift_id: Some(shift_id),
                rfid_card_uid: Some(rfid.to_string()),
                pin_code: Some("1234".to_string()),
                employment_type: EmploymentType::Permanent,
                join_date: "2024-01-01".to_string(),
                end_date: None,
                is_active: true,
                avatar_url: None,
                emergency_contact_name: Some("Relative".to_string()),
                emergency_contact_phone: Some("+880 1811 000000".to_string()),
                created_at: Some(now.clone()),
                updated_at: Some(now.clone()),
            };
            db.save_employee(&emp)?;

            // Seed biometric profile (512-D embedding)
            let dummy_embedding = vec![0.044f32; 512];
            db.save_face_profile(&emp.full_name, Some(code), &dummy_embedding, 95.0, "FRONT")?;
        }
        println!("      ✓ 10 employees and 512-D biometrics seeded.");

        // 6. Seed Edge Turnstile Devices
        println!("[6/6] Seeding physical turnstile and gate hardware records...");
        let gates = [
            ("DEV_TURNSTILE_01", "Main Lobby Turnstile East", "192.168.10.101"),
            ("DEV_TURNSTILE_02", "Main Lobby Turnstile West", "192.168.10.102"),
            ("DEV_GATE_ENG", "Engineering Lab Biometric Gate", "192.168.10.105"),
        ];

        for (dev_id, label, ip) in gates {
            let dev = RegisteredDevice {
                device_id: dev_id.to_string(),
                device_name: label.to_string(),
                android_id: Some(format!("AND_{}", dev_id)),
                mac_address: Some("00:1B:44:11:3A:B7".to_string()),
                ip_address: Some(ip.to_string()),
                device_type: "TURNSTILE".to_string(),
                location_name: Some("Lobby".to_string()),
                status: DeviceStatus::Online,
                last_heartbeat: Some(now.clone()),
                app_version: Some("v2.4.0-rust-native".to_string()),
                created_at: Some(now.clone()),
            };
            db.register_device(&dev)?;
        }
        println!("      ✓ Edge hardware devices registered.");

        println!("============================================================");
        println!("Enterprise seed completed successfully!");
        Ok(())
    }
}
