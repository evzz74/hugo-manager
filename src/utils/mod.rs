#![allow(dead_code)]
use std::path::Path;

/// Check if a path is a valid Hugo project directory
pub fn is_hugo_project(path: &Path) -> bool {
    path.join("hugo.yaml").exists()
        || path.join("hugo.toml").exists()
        || path.join("hugo.json").exists()
        || path.join("config.yaml").exists()
        || path.join("config.toml").exists()
}

/// Get the Hugo config file path from a project directory
pub fn get_config_path(project_path: &Path) -> Option<std::path::PathBuf> {
    if project_path.join("hugo.yaml").exists() {
        Some(project_path.join("hugo.yaml"))
    } else if project_path.join("hugo.toml").exists() {
        Some(project_path.join("hugo.toml"))
    } else if project_path.join("config.yaml").exists() {
        Some(project_path.join("config.yaml"))
    } else if project_path.join("config.toml").exists() {
        Some(project_path.join("config.toml"))
    } else {
        None
    }
}

/// Convert a string to a URL-friendly slug
pub fn to_slug(s: &str) -> String {
    s.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c.to_ascii_lowercase()
            } else if c.is_whitespace() {
                '-'
            } else {
                c
            }
        })
        .collect()
}

/// Format a date string for display
pub fn format_date(date_str: &str) -> String {
    // Try to parse and reformat the date
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(date_str) {
        dt.format("%Y-%m-%d %H:%M").to_string()
    } else if let Ok(dt) = chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
        dt.format("%Y-%m-%d").to_string()
    } else {
        date_str.to_string()
    }
}

/// Maximum file size for reading (10 MB)
const MAX_FILE_SIZE: u64 = 10 * 1024 * 1024;

/// Read a file and return its content as a string (UTF-8 with BOM support)
pub fn read_file_content(path: &Path) -> anyhow::Result<String> {
    // Check file size before reading to prevent OOM
    let metadata = std::fs::metadata(path)?;
    if metadata.len() > MAX_FILE_SIZE {
        anyhow::bail!(
            "File too large ({:.1} MB, max {} MB): {}",
            metadata.len() as f64 / 1_048_576.0,
            MAX_FILE_SIZE / 1_048_576,
            path.display()
        );
    }

    let bytes = std::fs::read(path)?;
    
    // Try UTF-8 with BOM first
    if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
        return Ok(String::from_utf8(bytes[3..].to_vec())?);
    }
    
    // Try UTF-8
    if let Ok(s) = String::from_utf8(bytes.clone()) {
        return Ok(s);
    }
    
    // Try UTF-16 LE
    if bytes.len() >= 2 && bytes[0] == 0xFF && bytes[1] == 0xFE {
        let utf16: Vec<u16> = bytes[2..]
            .chunks_exact(2)
            .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
            .collect();
        return Ok(String::from_utf16(&utf16)?);
    }
    
    // Try UTF-16 BE
    if bytes.len() >= 2 && bytes[0] == 0xFE && bytes[1] == 0xFF {
        let utf16: Vec<u16> = bytes[2..]
            .chunks_exact(2)
            .map(|chunk| u16::from_be_bytes([chunk[0], chunk[1]]))
            .collect();
        return Ok(String::from_utf16(&utf16)?);
    }
    
    // Fallback: use lossy conversion
    Ok(String::from_utf8_lossy(&bytes).to_string())
}

/// Write content to a file
pub fn write_file_content(path: &Path, content: &str) -> anyhow::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    Ok(std::fs::write(path, content)?)
}

/// Get the content directory path for a Hugo project
pub fn get_content_dir(project_path: &Path) -> std::path::PathBuf {
    project_path.join("content")
}

/// Get the themes directory path for a Hugo project
pub fn get_themes_dir(project_path: &Path) -> std::path::PathBuf {
    project_path.join("themes")
}

/// Check if Hugo is installed and accessible
pub fn is_hugo_installed() -> bool {
    std::process::Command::new("hugo")
        .arg("version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Get Hugo version string
pub fn get_hugo_version() -> Option<String> {
    let output = std::process::Command::new("hugo")
        .arg("version")
        .output()
        .ok()?;
    
    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        None
    }
}
