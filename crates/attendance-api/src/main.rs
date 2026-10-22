pub mod middleware;
pub mod routes;
pub mod ui;

use attendance_auth::AuthService;
use attendance_bigquery::{BigQueryClient, BigQueryConfig};
use attendance_common::{AppError, Result, TimeHelper};
use attendance_core::AttendanceEngine;
use attendance_database::DbPool;
use attendance_device::DeviceRegistry;
use attendance_domain::*;
use attendance_face::FaceVectorIndex;
use attendance_reporting::ReportingService;
use attendance_sync::OutboxWorker;

use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Clone)]
pub struct AppState {
    pub db: DbPool,
    pub engine: AttendanceEngine,
    pub face_index: FaceVectorIndex,
    pub devices: DeviceRegistry,
    pub reporting: ReportingService,
    pub bq_client: Arc<BigQueryClient>,
    pub outbox_worker: Arc<OutboxWorker>,
    pub deepface_state: Option<Arc<attendance_face::DeepFaceAppState>>,
    pub deepface_reloadable: Arc<parking_lot::RwLock<Option<Arc<attendance_face::DeepFaceAppState>>>>,
}

impl AppState {
    pub fn get_deepface(&self) -> Option<Arc<attendance_face::DeepFaceAppState>> {
        self.deepface_reloadable
            .read()
            .clone()
            .or_else(|| self.deepface_state.clone())
    }

    pub fn reload_deepface_from_dir(&self, models_dir: &str) -> std::result::Result<(), String> {
        match attendance_face::DeepFaceAppState::load_from_dir(models_dir) {
            Ok(s) => {
                let mut guard = self.deepface_reloadable.write();
                *guard = Some(Arc::new(s));
                Ok(())
            }
            Err(e) => Err(e.to_string()),
        }
    }
}

// Custom error responder
pub struct ApiError(AppError);

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, error_message) = match &self.0 {
            AppError::NoFaceDetected | AppError::MultipleFacesDetected(_) => {
                (StatusCode::UNPROCESSABLE_ENTITY, self.0.to_string())
            }
            AppError::Validation(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
            AppError::Conflict(msg) => (StatusCode::CONFLICT, msg.clone()),
            AppError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg.clone()),
            AppError::Database(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.clone()),
            other => (StatusCode::INTERNAL_SERVER_ERROR, other.to_string()),
        };

        let body = Json(serde_json::json!({
            "error": error_message,
            "success": false
        }));

        (status, body).into_response()
    }
}

impl<E: Into<AppError>> From<E> for ApiError {
    fn from(err: E) -> Self {
        ApiError(err.into())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    if let Ok(env_content) = std::fs::read_to_string(".env") {
        for line in env_content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            if let Some((k, v)) = trimmed.split_once('=') {
                let key = k.trim();
                let val = v.trim().trim_matches('"').trim_matches('\'');
                if !key.is_empty() && std::env::var(key).is_err() {
                    std::env::set_var(key, val);
                }
            }
        }
    }
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "attendance_api=info,tower_http=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting Attendance System Rust/Axum Engine...");

    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8080);

    // CAP-01: SingleInstanceManager lock (55531 for default port 8080, offset for staging ports)
    let home = std::env::var("HOME").unwrap_or_else(|_| "/home/opc".to_string());
    let lock_port = if port == 8080 { 55531 } else { 47451u16.saturating_add(port % 10000) };
    let lock_file = if port == 8080 {
        format!("{}/attendance_system.lock", home)
    } else {
        format!("{}/attendance_system_{}.lock", home, port)
    };
    let mut single_instance = attendance_common::SingleInstanceManager::new(lock_port)
        .with_lock_file(&lock_file);
    if let Err(e) = single_instance.acquire_lock() {
        tracing::warn!("SingleInstanceManager lock warning: {}", e);
    }

    // CAP-03: Startup time integrity verification and persistence (~/.attendance_time_check)
    let _ = TimeHelper::verify_and_persist_time_integrity();

    let db_path = std::env::var("ATTENDANCE_DB_PATH")
        .or_else(|_| std::env::var("DATABASE_URL"))
        .unwrap_or_else(|_| "data/attendance.db".to_string())
        .replace("sqlite://", "");

    let db = DbPool::new(&db_path)?;
    let engine = AttendanceEngine::new(db.clone());
    let face_index = FaceVectorIndex::new(attendance_face::FACENET512_MATCH_THRESHOLD);
    let devices = DeviceRegistry::default();
    let reporting = ReportingService::new(db.clone());
    let bq_client = Arc::new(BigQueryClient::new(BigQueryConfig::default()));
    let outbox_worker = Arc::new(OutboxWorker::new(db.clone(), bq_client.clone(), 30));

    // Start background BigQuery sync worker
    outbox_worker.clone().start();

    // Populate initial sample data if database is fresh
    seed_if_empty(&db, &face_index)?;
    bootstrap_face_index(&db, &face_index)?;

    let models_dir = std::env::var("MODELS_DIR")
        .or_else(|_| std::env::var("ATTENDANCE_MODELS_DIR"))
        .unwrap_or_else(|_| "models".to_string());
    let deepface_state = match attendance_face::DeepFaceAppState::load_from_dir(&models_dir) {
        Ok(s) => {
            tracing::info!("FaceNet512 & MTCNN ONNX models loaded successfully from {}", models_dir);
            Some(Arc::new(s))
        }
        Err(e) => {
            tracing::warn!("DeepFace ONNX models not loaded from '{}': {}. (Copy models/*.onnx to enable live FaceNet512 ONNX inference)", models_dir, e);
            None
        }
    };
    let deepface_reloadable = Arc::new(parking_lot::RwLock::new(deepface_state.clone()));

    let state = AppState {
        db,
        engine,
        face_index,
        devices,
        reporting,
        bq_client,
        outbox_worker,
        deepface_state,
        deepface_reloadable,
    };

    let app = Router::new()
        // Interactive Web Kiosk & Admin Portal
        .route("/", get(render_portal_ui))
        // Root endpoints matching Python AndroidIDReceiver (lines 5829, 5843)
        .route(
            "/receive_android_id",
            post(routes::device_routes::receive_android_id)
                .get(routes::device_routes::get_receive_android_id_fallback),
        )
        .route("/session/:id", get(routes::device_routes::get_device_session))
        .route(
            "/api/receive_android_id",
            post(routes::device_routes::receive_android_id)
                .get(routes::device_routes::get_receive_android_id_fallback),
        )
        // Mount all 15 modular REST API route domains
        .nest("/api/v1", routes::configure_api_routes())
        .fallback(handle_root_fallback)
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("Server listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn render_portal_ui() -> Html<String> {
    Html(ui::render_full_portal_html())
}

async fn handle_root_fallback(
    State(state): State<AppState>,
    uri: axum::http::Uri,
) -> impl IntoResponse {
    let path = uri.path().trim_start_matches('/');
    if !path.is_empty() && !path.contains('/') {
        if let Ok(Some(session)) = state.db.get_device_session(path) {
            return (
                StatusCode::OK,
                Json(serde_json::json!({
                    "status": "ready",
                    "session_id": session.session_id,
                    "registered_android_id": session.registered_android_id,
                    "session_status": session.status,
                    "created_at": session.created_at
                })),
            ).into_response();
        } else {
            return (
                StatusCode::OK,
                Json(serde_json::json!({
                    "status": "ready",
                    "session_id": path
                })),
            ).into_response();
        }
    }

    (
        StatusCode::NOT_FOUND,
        Json(serde_json::json!({
            "error": "Not Found",
            "success": false
        })),
    ).into_response()
}

fn seed_if_empty(db: &DbPool, _face_index: &FaceVectorIndex) -> Result<()> {
    if db.get_company("HQ_001")?.is_none() {
        let now = TimeHelper::now_utc_sqlite_string();
        let default_comp = CompanyInfo {
            company_id: "HQ_001".to_string(),
            company_name: "Smart Attendance HQ".to_string(),
            password: AuthService::hash_password("admin123"),
            address_road: Some("100 Motijheel C/A".to_string()),
            address_city: Some("Dhaka".to_string()),
            contact_person_name: Some("HR Admin".to_string()),
            contact_person_designation: Some("Head of HR".to_string()),
            contact_person_number: Some("01700000000".to_string()),
            website: Some("https://attendance.local".to_string()),
            email: Some("hr@attendance.local".to_string()),
            tax_id: None,
            created_at: Some(now.clone()),
            last_updated: Some(now),
        };
        let _ = db.save_company(&default_comp);
        if db.get_hr_password("HQ_001")?.is_none() {
            let _ = attendance_auth::HrAuthService::set_hr_password(
                db,
                "HQ_001",
                "Smart Attendance HQ",
                "HR@123",
            );
        }
    }

    if let Ok(companies) = db.list_companies() {
        for mut comp in companies {
            let is_sha256_hex = comp.password.len() == 64
                && comp.password.chars().all(|c| c.is_ascii_hexdigit());
            if !comp.password.is_empty() && !is_sha256_hex {
                comp.password = AuthService::hash_password(&comp.password);
                let _ = db.save_company(&comp);
            }
        }
    }

    let users = db.get_all_users()?;
    if users.is_empty() {
        tracing::info!("Seeding initial users...");
        let sample_users = vec![
            ("ADMIN", "System Administrator", "ADM000"),
            ("RAHIM", "Software Architect", "DEV001"),
            ("KARIM", "HR Specialist", "HR002"),
            ("FATIMA", "Finance Manager", "FIN003"),
            ("NAZIA", "Operations Director", "OPS004"),
        ];

        for (name, role, emp_id) in sample_users {
            let u = User {
                company_user_uuid: format!("C_{}", emp_id),
                company_name: Some("Smart Attendance HQ".to_string()),
                company_uuid: Some("HQ_001".to_string()),
                username: name.to_string(),
                user_uuid: Some(uuid::Uuid::new_v4().to_string()),
                user_designation: Some(role.to_string()),
                user_android_id: Some(format!("ANDR_{}", emp_id)),
                face_encoding: None,
                register_with_camera: Some("N".to_string()),
                password: Some(AuthService::hash_password("admin123")),
                role: if name == "ADMIN" { UserRole::SuperAdmin } else { UserRole::Employee },
                status: UserStatus::Active,
                email: Some(format!("{}@company.com", name.to_lowercase())),
                phone: Some("01700000000".to_string()),
                created_at: Some(TimeHelper::now_utc_sqlite_string()),
                last_updated: Some(TimeHelper::now_utc_sqlite_string()),
            };
            db.insert_or_update_user(&u)?;

            let emp = Employee {
                id: None,
                employee_code: emp_id.to_string(),
                user_uuid: u.user_uuid.clone().unwrap(),
                company_id: "HQ_001".to_string(),
                first_name: name.to_string(),
                last_name: "Staff".to_string(),
                full_name: format!("{} Staff", name),
                email: u.email.clone(),
                phone: u.phone.clone(),
                department_id: None,
                designation_id: None,
                shift_id: None,
                rfid_card_uid: Some(format!("RFID_{}", emp_id)),
                pin_code: Some("1234".to_string()),
                employment_type: EmploymentType::Permanent,
                join_date: "2026-01-01".to_string(),
                end_date: None,
                is_active: true,
                avatar_url: None,
                emergency_contact_name: None,
                emergency_contact_phone: None,
                created_at: Some(TimeHelper::now_utc_sqlite_string()),
                updated_at: Some(TimeHelper::now_utc_sqlite_string()),
            };
            db.save_employee(&emp)?;
        }
    }
    Ok(())
}

fn bootstrap_face_index(db: &DbPool, face_index: &FaceVectorIndex) -> Result<()> {
    let mut loaded = 0;
    let mut users_with_profiles = std::collections::HashSet::new();

    if let Ok(profiles) = db.get_all_face_profiles() {
        for p in profiles {
            if !p.embedding_vector.is_empty()
                && !FaceVectorIndex::is_corrupted_constant_vector(&p.embedding_vector)
            {
                if face_index.enroll_user(&p.username, p.embedding_vector).is_ok() {
                    users_with_profiles.insert(p.username.to_uppercase());
                    loaded += 1;
                }
            } else if let Some(pid) = p.id {
                // CAP-42: Clean up corrupted constant seed vectors from face_profiles
                let conn = db.get_connection();
                let _ = conn.execute("DELETE FROM face_profiles WHERE id = ?1", rusqlite::params![pid]);
                tracing::warn!("Removed corrupted constant face_profile id={} for user={}", pid, p.username);
            }
        }
    }

    let users = db.get_all_users()?;
    for u in users {
        if users_with_profiles.contains(&u.username.to_uppercase()) {
            continue;
        }
        if let Some(enc_str) = &u.face_encoding {
            if let Ok(vec) = serde_json::from_str::<Vec<f32>>(enc_str) {
                let is_zero_placeholder = !vec.is_empty()
                    && vec.iter().all(|&x| x == 0.0)
                    && u.register_with_camera.as_deref() == Some("N");
                if is_zero_placeholder {
                    // CAP-17: Preserve 512-zero placeholder for non-camera users without enrolling into biometric index
                    continue;
                } else if !vec.is_empty() && !FaceVectorIndex::is_corrupted_constant_vector(&vec) {
                    if let Ok(()) = face_index.enroll_user(&u.username, vec) {
                        loaded += 1;
                    }
                } else {
                    let conn = db.get_connection();
                    let _ = conn.execute(
                        "UPDATE users SET face_encoding = NULL, register_with_camera = 'N' WHERE UPPER(username) = ?1",
                        rusqlite::params![u.username.to_uppercase()],
                    );
                }
            }
        }
    }
    let _ = face_index.sync_faiss_files();
    tracing::info!("Bootstrapped {} valid face embeddings into memory index", loaded);
    Ok(())
}
