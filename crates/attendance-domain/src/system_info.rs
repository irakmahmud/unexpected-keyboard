use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemHealthReport {
    pub status: String,
    pub service_name: String,
    pub version: String,
    pub server_time_dhaka: String,
    pub uptime_seconds: u64,
    pub database_status: DatabaseHealth,
    pub outbox_status: OutboxHealth,
    pub face_index_status: FaceIndexHealth,
    pub active_devices_count: usize,
    pub memory_stats: MemoryHealth,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseHealth {
    pub dialect: String,
    pub connected: bool,
    pub pool_size: u32,
    pub wal_mode: bool,
    pub foreign_keys_enabled: bool,
    pub database_size_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutboxHealth {
    pub pending_count: usize,
    pub processed_count: usize,
    pub failed_count: usize,
    pub dead_letter_count: usize,
    pub cloud_target: String,
    pub worker_active: bool,
    pub last_sync_timestamp: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaceIndexHealth {
    pub total_enrolled_faces: usize,
    pub vector_dimension: usize,
    pub distance_metric: String,
    pub threshold: f32,
    pub index_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryHealth {
    pub virtual_memory_bytes: u64,
    pub resident_set_bytes: u64,
    pub allocation_overhead_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseBackupInfo {
    pub backup_id: String,
    pub filename: String,
    pub size_bytes: u64,
    pub created_at: String,
    pub checksum_sha256: String,
    pub total_records: u64,
}
