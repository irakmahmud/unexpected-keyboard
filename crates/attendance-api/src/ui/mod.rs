pub mod kiosk_view;
pub mod mobile_view;
pub mod scripts;
pub mod styles;

pub use kiosk_view::render_full_portal_html;
pub use mobile_view::{render_mobile_app_html, render_mobile_manifest_json};

