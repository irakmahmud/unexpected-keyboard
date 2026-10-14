use parking_lot::Mutex;
use std::sync::Arc;
use std::time::Instant;

#[derive(Debug, Clone)]
pub struct CameraFrame {
    pub frame_id: u64,
    pub width: u32,
    pub height: u32,
    pub timestamp_ms: u64,
    pub raw_data: Vec<u8>,
}

pub struct CameraPipeline {
    frame_counter: Arc<Mutex<u64>>,
    last_fps_check: Arc<Mutex<Instant>>,
}

impl Default for CameraPipeline {
    fn default() -> Self {
        Self {
            frame_counter: Arc::new(Mutex::new(0)),
            last_fps_check: Arc::new(Mutex::new(Instant::now())),
        }
    }
}

impl CameraPipeline {
    pub fn new() -> Self {
        Self::default()
    }

    /// Process incoming camera frame from web stream or hardware capture
    pub fn process_frame(&self, width: u32, height: u32, raw_bytes: Vec<u8>) -> CameraFrame {
        let mut count = self.frame_counter.lock();
        *count += 1;

        CameraFrame {
            frame_id: *count,
            width,
            height,
            timestamp_ms: chrono::Utc::now().timestamp_millis() as u64,
            raw_data: raw_bytes,
        }
    }

    /// Measure actual stream FPS
    pub fn calculate_fps(&self) -> f64 {
        let count = *self.frame_counter.lock();
        let elapsed = self.last_fps_check.lock().elapsed().as_secs_f64();
        if elapsed > 0.0 {
            count as f64 / elapsed
        } else {
            0.0
        }
    }
}
