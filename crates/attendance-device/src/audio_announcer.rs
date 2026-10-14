use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AudioCueType {
    SuccessIn,
    SuccessOut,
    AlreadyPunched,
    AccessDenied,
    FaceNotRecognized,
    TemperatureHigh,
    MaskRequired,
    DeviceReady,
}

pub struct AudioAnnouncer;

impl AudioAnnouncer {
    /// Return the Bengali & English text announcement corresponding to the event cue
    pub fn get_announcement_text(cue: AudioCueType, name: Option<&str>) -> (String, String) {
        let employee = name.unwrap_or("ব্যবহারকারী");
        match cue {
            AudioCueType::SuccessIn => (
                format!("স্বাগতম {}, আপনার আগমন নিশ্চিত হয়েছে।", employee),
                format!("Welcome {}, check-in recorded.", employee),
            ),
            AudioCueType::SuccessOut => (
                format!("ধন্যবাদ {}, আপনার প্রস্থান নিশ্চিত হয়েছে।", employee),
                format!("Thank you {}, check-out recorded.", employee),
            ),
            AudioCueType::AlreadyPunched => (
                format!("{}, আপনি ইতোমধ্যে উপস্থিতি দিয়েছেন। কিছুক্ষণ অপেক্ষা করুন।", employee),
                format!("{}, you have already punched. Please wait.", employee),
            ),
            AudioCueType::AccessDenied => (
                "অনুমতি নেই। অনুগ্রহ করে এইচআর এর সাথে যোগাযোগ করুন।".to_string(),
                "Access denied. Please contact HR.".to_string(),
            ),
            AudioCueType::FaceNotRecognized => (
                "চেহারা শনাক্ত করা যায়নি। অনুগ্রহ করে ক্যামেরার দিকে সরাসরি তাকান।".to_string(),
                "Face not recognized. Please look directly at the camera.".to_string(),
            ),
            AudioCueType::TemperatureHigh => (
                "উচ্চ তাপমাত্রা শনাক্ত হয়েছে। প্রবেশ নিষিদ্ধ।".to_string(),
                "High temperature detected. Access restricted.".to_string(),
            ),
            AudioCueType::MaskRequired => (
                "অনুগ্রহ করে মাস্ক পরুন।".to_string(),
                "Please wear a mask.".to_string(),
            ),
            AudioCueType::DeviceReady => (
                "ডিভাইস প্রস্তুত। উপস্থিতি প্রদান করুন।".to_string(),
                "Device ready. Please punch attendance.".to_string(),
            ),
        }
    }

    /// Simulate audio frequency beep pattern (frequency Hz, duration ms)
    pub fn get_beep_pattern(cue: &AudioCueType) -> Vec<(u32, u32)> {
        match cue {
            AudioCueType::SuccessIn | AudioCueType::SuccessOut => vec![(1800, 150), (2400, 200)],
            AudioCueType::AlreadyPunched => vec![(1000, 100), (1000, 100)],
            AudioCueType::AccessDenied | AudioCueType::FaceNotRecognized => {
                vec![(400, 300), (300, 400)]
            }
            AudioCueType::TemperatureHigh => vec![(2800, 100), (2800, 100), (2800, 100)],
            _ => vec![(1200, 100)],
        }
    }
}
