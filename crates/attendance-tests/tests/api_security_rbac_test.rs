use attendance_auth::{ApiKeyManager, JwtClaims, JwtService, PasswordHasher, Permission, RbacPolicy};
use attendance_common::SingleInstanceManager;
use attendance_domain::UserRole;

#[test]
fn test_password_hashing_and_verification() {
    let raw_password = "SecureAdminPassword!2026";
    let hash = PasswordHasher::hash_sha256(raw_password);

    assert!(!hash.is_empty());
    assert_ne!(hash, raw_password);

    // Correct verification
    let is_valid = PasswordHasher::verify(raw_password, &hash);
    assert!(is_valid, "Correct password must verify successfully");

    // Incorrect password
    let is_invalid = PasswordHasher::verify("WrongPassword123", &hash);
    assert!(!is_invalid, "Incorrect password must be rejected");
}

#[test]
fn test_jwt_token_lifecycle_and_claims() {
    let jwt = JwtService::new("my-super-secret-jwt-signing-key-for-attendance-system-2026");

    let claims = JwtClaims {
        sub: "admin_user".to_string(),
        username: "admin_user".to_string(),
        role: "SuperAdmin".to_string(),
        company_id: Some("TECHFIRM_01".to_string()),
        exp: chrono::Utc::now().timestamp() as u64 + 3600,
        iat: chrono::Utc::now().timestamp() as u64,
    };

    let token = jwt.sign(&claims).expect("JWT signing failed");
    assert!(!token.is_empty());

    // Validate claims
    let verified = jwt.verify(&token).expect("Token verification failed");
    assert_eq!(verified.sub, "admin_user");
    assert_eq!(verified.role, "SuperAdmin");
    assert_eq!(verified.company_id.as_deref(), Some("TECHFIRM_01"));
}

#[test]
fn test_rbac_permission_matrix() {
    // SuperAdmin has access to everything
    assert!(RbacPolicy::has_permission(&UserRole::SuperAdmin, &Permission::ManageUsers));
    assert!(RbacPolicy::has_permission(&UserRole::SuperAdmin, &Permission::ApproveLeaves));
    assert!(RbacPolicy::has_permission(&UserRole::SuperAdmin, &Permission::ModifyAttendance));
    assert!(RbacPolicy::has_permission(&UserRole::SuperAdmin, &Permission::ViewReports));

    // CompanyAdmin has access to operational settings and reports
    assert!(RbacPolicy::has_permission(&UserRole::CompanyAdmin, &Permission::ApproveLeaves));
    assert!(RbacPolicy::has_permission(&UserRole::CompanyAdmin, &Permission::ViewReports));

    // HrManager can approve leaves and view reports, but cannot manage users
    assert!(RbacPolicy::has_permission(&UserRole::HrManager, &Permission::ApproveLeaves));
    assert!(RbacPolicy::has_permission(&UserRole::HrManager, &Permission::ViewReports));
    assert!(!RbacPolicy::has_permission(&UserRole::HrManager, &Permission::ManageUsers));

    // Employee cannot approve leaves or manage users
    assert!(!RbacPolicy::has_permission(&UserRole::Employee, &Permission::ApproveLeaves));
    assert!(!RbacPolicy::has_permission(&UserRole::Employee, &Permission::ManageUsers));
    assert!(RbacPolicy::has_permission(&UserRole::Employee, &Permission::PunchAttendance));
}

#[test]
fn test_device_api_key_generation_and_validation() {
    let client_name = "DEV_TURNSTILE_MAIN_ENTRANCE";
    let (full_key, key_hash, _info) = ApiKeyManager::generate_api_key(client_name, vec!["punch".to_string()]);

    assert!(full_key.starts_with("ak_"));

    // Validation
    assert!(
        ApiKeyManager::verify_key(&full_key, &key_hash).unwrap_or(false),
        "Generated key must match key hash"
    );

    assert!(
        ApiKeyManager::verify_key("ak_invalid_key_secret", &key_hash).is_err(),
        "Mismatched key must be rejected"
    );
}

#[test]
fn test_single_instance_lock() {
    // Test port 19876
    let mut lock1 = SingleInstanceManager::new(19876);
    let res1 = lock1.acquire_lock();
    assert!(res1.is_ok(), "First single instance lock must acquire");

    let mut lock2 = SingleInstanceManager::new(19876);
    let res2 = lock2.acquire_lock();
    assert!(
        res2.is_err(),
        "Second simultaneous single instance lock must fail"
    );

    // Release lock1
    drop(lock1);

    // Now acquiring should succeed again
    let mut lock3 = SingleInstanceManager::new(19876);
    let res3 = lock3.acquire_lock();
    assert!(
        res3.is_ok(),
        "Acquiring lock after previous drop must succeed"
    );
}
