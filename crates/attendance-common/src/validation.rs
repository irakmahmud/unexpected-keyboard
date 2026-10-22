use crate::error::{AppError, Result};
use std::path::{Component, Path, PathBuf};

/// Validates a file path or filename to prevent directory traversal (CAP-04, Python validate_filepath lines 162-188).
pub fn validate_filepath(
    file_path: &str,
    allowed_dir: Option<&Path>,
    allowed_extensions: &[&str],
) -> Result<PathBuf> {
    let trimmed = file_path.trim();
    if trimmed.is_empty() {
        return Err(AppError::Validation("File path cannot be empty".to_string()));
    }
    if trimmed.contains('\0') {
        return Err(AppError::Validation("File path contains illegal null byte".to_string()));
    }

    let raw_path = Path::new(trimmed);
    for comp in raw_path.components() {
        if matches!(comp, Component::ParentDir) {
            return Err(AppError::Validation(
                "Path traversal ('..') is strictly forbidden".to_string(),
            ));
        }
    }

    if !allowed_extensions.is_empty() {
        let ext = raw_path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_ascii_lowercase())
            .unwrap_or_default();
        let ext_with_dot = format!(".{}", ext);
        let valid_ext = allowed_extensions.iter().any(|allowed| {
            let a = allowed.trim_start_matches('.').to_ascii_lowercase();
            a == ext || allowed.eq_ignore_ascii_case(&ext_with_dot)
        });
        if !valid_ext {
            return Err(AppError::Validation(format!(
                "Invalid file extension '.{}'; allowed: {:?}",
                ext, allowed_extensions
            )));
        }
    }

    if let Some(base_dir) = allowed_dir {
        let _ = std::fs::create_dir_all(base_dir);
        let canonical_base = base_dir
            .canonicalize()
            .unwrap_or_else(|_| base_dir.to_path_buf());
        let file_name = raw_path
            .file_name()
            .ok_or_else(|| AppError::Validation("Invalid file name".to_string()))?;
        let joined = canonical_base.join(file_name);
        if !joined.starts_with(&canonical_base) {
            return Err(AppError::Validation(
                "Resolved path escapes allowed directory".to_string(),
            ));
        }
        Ok(joined)
    } else {
        Ok(raw_path.to_path_buf())
    }
}

/// Sanitizes a simple filename to prevent path injection.
pub fn sanitize_filename(name: &str) -> Result<String> {
    let trimmed = name.trim();
    if trimmed.is_empty() || trimmed.contains("..") || trimmed.contains('/') || trimmed.contains('\\') || trimmed.contains('\0') {
        return Err(AppError::Validation(format!("Unsafe filename: {}", name)));
    }
    Ok(trimmed.to_string())
}
