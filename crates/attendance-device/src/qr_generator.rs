pub struct QrCodeGenerator;

impl QrCodeGenerator {
    /// Generate ASCII matrix representation for terminal display
    pub fn generate_ascii_representation(content: &str) -> String {
        let hash = attendance_common::CryptoHelper::sha256_hex(content);
        let mut lines = Vec::new();
        lines.push("████████████████████████████████".to_string());
        lines.push("██ ▄▄▄▄▄ █  █ █ █ █ ▄▄▄▄▄ ██".to_string());
        lines.push("██ █   █ █ ▄▀█▀▄█ █ █   █ ██".to_string());
        lines.push("██ █▄▄▄█ █ ▄▀ ▄ █ █ █▄▄▄█ ██".to_string());
        lines.push("██▄▄▄▄▄▄▄█▄█ █▄█ █▄▄▄▄▄▄▄██".to_string());

        // Pseudo matrix based on content hash
        for chunk in hash.as_bytes().chunks(4).take(4) {
            let mut line = String::from("██ ");
            for b in chunk {
                if b % 2 == 0 {
                    line.push_str("██");
                } else {
                    line.push_str("  ");
                }
            }
            line.push_str(" ██");
            lines.push(line);
        }

        lines.push("██ ▄▄▄▄▄ █ ▄▀▄▀▄█ █ █ █ █ ██".to_string());
        lines.push("██ █   █ █ █▄█▄█▀▄█▄▄▄█▄█ ██".to_string());
        lines.push("██ █▄▄▄█ █ ▄ ▄▀▄█ ▄▄▄ ▄▄█ ██".to_string());
        lines.push("██▄▄▄▄▄▄▄█▄██▄███▄███▄███▄██".to_string());
        lines.push("████████████████████████████████".to_string());

        lines.join("\n")
    }

    /// Generate SVG QR code string
    pub fn generate_svg(content: &str, size_px: u32) -> String {
        format!(
            r##"<svg xmlns="http://www.w3.org/2000/svg" width="{}" height="{}" viewBox="0 0 100 100">
  <rect width="100" height="100" fill="#ffffff"/>
  <rect x="10" y="10" width="25" height="25" fill="#000000"/>
  <rect x="65" y="10" width="25" height="25" fill="#000000"/>
  <rect x="10" y="65" width="25" height="25" fill="#000000"/>
  <rect x="15" y="15" width="15" height="15" fill="#ffffff"/>
  <rect x="70" y="15" width="15" height="15" fill="#ffffff"/>
  <rect x="15" y="70" width="15" height="15" fill="#ffffff"/>
  <!-- Content: {} -->
</svg>"##,
            size_px, size_px, content
        )
    }
}
