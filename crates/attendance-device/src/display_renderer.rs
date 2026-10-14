use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LcdScreenState {
    pub line1: String,
    pub line2: String,
    pub line3: String,
    pub line4: String,
}

pub struct DisplayRenderer;

impl DisplayRenderer {
    /// Format text for a 16x2 character LCD
    pub fn render_16x2(line1: &str, line2: &str) -> LcdScreenState {
        LcdScreenState {
            line1: Self::pad_or_truncate(line1, 16),
            line2: Self::pad_or_truncate(line2, 16),
            line3: String::new(),
            line4: String::new(),
        }
    }

    /// Format text for a 20x4 character LCD
    pub fn render_20x4(l1: &str, l2: &str, l3: &str, l4: &str) -> LcdScreenState {
        LcdScreenState {
            line1: Self::pad_or_truncate(l1, 20),
            line2: Self::pad_or_truncate(l2, 20),
            line3: Self::pad_or_truncate(l3, 20),
            line4: Self::pad_or_truncate(l4, 20),
        }
    }

    /// Kiosk idle screen
    pub fn idle_screen_20x4(company_name: &str, dhaka_time: &str) -> LcdScreenState {
        Self::render_20x4(
            company_name,
            "ATTENDANCE KIOSK",
            dhaka_time,
            "LOOK AT CAMERA / SCAN",
        )
    }

    /// Success screen
    pub fn punch_success_screen(
        user: &str,
        punch_type: &str,
        time: &str,
    ) -> LcdScreenState {
        Self::render_20x4(
            "ACCESS GRANTED",
            &format!("USER: {}", user),
            &format!("{}: {}", punch_type, time),
            "HAVE A GREAT DAY!",
        )
    }

    fn pad_or_truncate(s: &str, width: usize) -> String {
        let chars: Vec<char> = s.chars().collect();
        if chars.len() > width {
            chars[..width].iter().collect()
        } else {
            format!("{:<width$}", s, width = width)
        }
    }
}
