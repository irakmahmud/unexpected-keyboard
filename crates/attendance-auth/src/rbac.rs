use attendance_domain::UserRole;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Permission {
    PunchAttendance,
    ViewOwnAttendance,
    ViewAllAttendance,
    ModifyAttendance,
    ManageUsers,
    ManageEmployees,
    ManageShifts,
    ManageLeaves,
    ApproveLeaves,
    ViewReports,
    ExportReports,
    TriggerCloudSync,
    ManageSystemSettings,
    AdminOverride,
}

pub struct RbacPolicy;

impl RbacPolicy {
    pub fn get_permissions(role: &UserRole) -> Vec<Permission> {
        match role {
            UserRole::SuperAdmin => vec![
                Permission::PunchAttendance,
                Permission::ViewOwnAttendance,
                Permission::ViewAllAttendance,
                Permission::ModifyAttendance,
                Permission::ManageUsers,
                Permission::ManageEmployees,
                Permission::ManageShifts,
                Permission::ManageLeaves,
                Permission::ApproveLeaves,
                Permission::ViewReports,
                Permission::ExportReports,
                Permission::TriggerCloudSync,
                Permission::ManageSystemSettings,
                Permission::AdminOverride,
            ],
            UserRole::CompanyAdmin => vec![
                Permission::PunchAttendance,
                Permission::ViewOwnAttendance,
                Permission::ViewAllAttendance,
                Permission::ModifyAttendance,
                Permission::ManageUsers,
                Permission::ManageEmployees,
                Permission::ManageShifts,
                Permission::ManageLeaves,
                Permission::ApproveLeaves,
                Permission::ViewReports,
                Permission::ExportReports,
                Permission::TriggerCloudSync,
                Permission::AdminOverride,
            ],
            UserRole::HrManager => vec![
                Permission::PunchAttendance,
                Permission::ViewOwnAttendance,
                Permission::ViewAllAttendance,
                Permission::ModifyAttendance,
                Permission::ManageEmployees,
                Permission::ManageLeaves,
                Permission::ApproveLeaves,
                Permission::ViewReports,
                Permission::ExportReports,
                Permission::AdminOverride,
            ],
            UserRole::DepartmentHead => vec![
                Permission::PunchAttendance,
                Permission::ViewOwnAttendance,
                Permission::ViewAllAttendance,
                Permission::ApproveLeaves,
                Permission::ViewReports,
            ],
            UserRole::Employee => vec![
                Permission::PunchAttendance,
                Permission::ViewOwnAttendance,
            ],
            UserRole::SecurityGuard => vec![
                Permission::PunchAttendance,
                Permission::ViewAllAttendance,
            ],
            UserRole::KioskTerminal => vec![
                Permission::PunchAttendance,
            ],
        }
    }

    pub fn has_permission(role: &UserRole, permission: &Permission) -> bool {
        Self::get_permissions(role).contains(permission)
    }
}
