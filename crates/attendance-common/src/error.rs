use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug, Serialize, Deserialize, Clone)]
pub enum ErrorCode {
    #[error("DATABASE_ERROR")]
    DatabaseError,
    #[error("RECORD_NOT_FOUND")]
    NotFound,
    #[error("VALIDATION_FAILED")]
    ValidationFailed,
    #[error("AUTHENTICATION_FAILED")]
    Unauthorized,
    #[error("PERMISSION_DENIED")]
    Forbidden,
    #[error("CONFLICT_DETECTED")]
    Conflict,
    #[error("COOLDOWN_ACTIVE")]
    CooldownActive,
    #[error("SINGLE_INSTANCE_VIOLATION")]
    SingleInstanceLock,
    #[error("DEVICE_OFFLINE")]
    DeviceOffline,
    #[error("FACE_RECOGNITION_ERROR")]
    FaceRecognitionError,
    #[error("CLOUD_SYNC_ERROR")]
    CloudSyncError,
    #[error("INTERNAL_SERVER_ERROR")]
    InternalError,
}

#[derive(Error, Debug)]
pub enum AppError {
    #[error("[DATABASE] {0}")]
    Database(String),

    #[error("[NOT_FOUND] {0}")]
    NotFound(String),

    #[error("[VALIDATION] {0}")]
    Validation(String),

    #[error("[UNAUTHORIZED] {0}")]
    Unauthorized(String),

    #[error("[FORBIDDEN] {0}")]
    Forbidden(String),

    #[error("[CONFLICT] {0}")]
    Conflict(String),

    #[error("[COOLDOWN] {0}")]
    Cooldown(String),

    #[error("[LOCK_ERROR] {0}")]
    LockError(String),

    #[error("[IO_ERROR] {0}")]
    Io(#[from] std::io::Error),

    #[error("[SERIALIZATION] {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("[INTERNAL] {0}")]
    Internal(String),

    #[error("[FACE_RECOGNITION] {0}")]
    Face(String),

    #[error("No face detected in image")]
    NoFaceDetected,

    #[error("Multiple faces detected in image ({0} faces); please ensure only 1 face is in frame")]
    MultipleFacesDetected(usize),

    #[error("[BIGQUERY_SYNC] {0}")]
    Sync(String),

    #[error("[DEVICE_ERROR] {0}")]
    Device(String),
}

impl AppError {
    pub fn error_code(&self) -> ErrorCode {
        match self {
            AppError::Database(_) => ErrorCode::DatabaseError,
            AppError::NotFound(_) => ErrorCode::NotFound,
            AppError::Validation(_) => ErrorCode::ValidationFailed,
            AppError::Unauthorized(_) => ErrorCode::Unauthorized,
            AppError::Forbidden(_) => ErrorCode::Forbidden,
            AppError::Conflict(_) => ErrorCode::Conflict,
            AppError::Cooldown(_) => ErrorCode::CooldownActive,
            AppError::LockError(_) => ErrorCode::SingleInstanceLock,
            AppError::Face(_) | AppError::NoFaceDetected | AppError::MultipleFacesDetected(_) => {
                ErrorCode::FaceRecognitionError
            }
            AppError::Sync(_) => ErrorCode::CloudSyncError,
            AppError::Device(_) => ErrorCode::DeviceOffline,
            _ => ErrorCode::InternalError,
        }
    }
}

pub type Result<T> = std::result::Result<T, AppError>;
