pub mod attendance_routes;
pub mod audit_routes;
pub mod auth_routes;
pub mod company_routes;
pub mod device_routes;
pub mod employee_routes;
pub mod face_routes;
pub mod holiday_routes;
pub mod leave_routes;
pub mod payroll_routes;
pub mod report_routes;
pub mod shift_routes;
pub mod sync_routes;
pub mod system_routes;
pub mod user_routes;

use crate::AppState;
use axum::Router;

pub fn configure_api_routes() -> Router<AppState> {
    Router::new()
        .nest("/attendance", attendance_routes::attendance_router())
        .nest("/auth", auth_routes::auth_router())
        .nest("/users", user_routes::user_router())
        .nest("/employees", employee_routes::employee_router())
        .nest("/shifts", shift_routes::shift_router())
        .nest("/leaves", leave_routes::leave_router())
        .nest("/holidays", holiday_routes::holiday_router())
        .nest("/companies", company_routes::company_router())
        .nest("/devices", device_routes::device_router())
        .nest("/face", face_routes::face_router())
        .nest("/sync", sync_routes::sync_router())
        .nest("/audit", audit_routes::audit_router())
        .nest("/payroll", payroll_routes::payroll_router())
        .nest("/reports", report_routes::report_router())
        .nest("/system", system_routes::system_router())
        .route("/receive_android_id", axum::routing::post(device_routes::receive_android_id))
        .route("/session/:id", axum::routing::get(device_routes::get_device_session))
        .route("/captures/:filename", axum::routing::get(attendance_routes::serve_capture_image))
        .route("/verify", axum::routing::post(face_routes::handle_verify_json))
        .route("/verify_multipart", axum::routing::post(face_routes::handle_verify_multipart))
        .route("/health", axum::routing::get(system_routes::get_health_status))
}
