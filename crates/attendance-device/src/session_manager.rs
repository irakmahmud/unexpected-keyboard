use attendance_common::Result;
use attendance_domain::DeviceSession;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Clone, Default)]
pub struct SessionManager {
    sessions: Arc<RwLock<HashMap<String, DeviceSession>>>,
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn create_session(&self) -> String {
        let sid = uuid::Uuid::new_v4().to_string()[..8].to_string();
        let session = DeviceSession {
            session_id: sid.clone(),
            created_at: chrono::Utc::now().to_rfc3339(),
            registered_android_id: None,
            status: "PENDING".to_string(),
        };

        self.sessions.write().insert(sid.clone(), session);
        sid
    }

    pub fn register_android_id(&self, session_id: Option<&str>, android_id: &str) -> Result<String> {
        let mut map = self.sessions.write();

        if let Some(sid) = session_id {
            if let Some(sess) = map.get_mut(sid) {
                sess.registered_android_id = Some(android_id.to_string());
                sess.status = "CONNECTED".to_string();
                return Ok(sid.to_string());
            }
        }

        let sid = uuid::Uuid::new_v4().to_string()[..8].to_string();
        map.insert(
            sid.clone(),
            DeviceSession {
                session_id: sid.clone(),
                created_at: chrono::Utc::now().to_rfc3339(),
                registered_android_id: Some(android_id.to_string()),
                status: "CONNECTED".to_string(),
            },
        );
        Ok(sid)
    }

    pub fn get_session(&self, sid: &str) -> Option<DeviceSession> {
        self.sessions.read().get(sid).cloned()
    }
}
