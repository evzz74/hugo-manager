#![allow(dead_code)]
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct Theme {
    pub name: String,
    pub path: PathBuf,
    pub description: String,
    pub author: String,
    pub version: String,
}

pub struct ThemeManager {
    project_path: PathBuf,
    pub available_themes: Vec<Theme>,
}

impl ThemeManager {
    pub fn new(project_path: &Path) -> Self {
        let mut manager = Self {
            project_path: project_path.to_path_buf(),
            available_themes: Vec::new(),
        };
        manager.load_themes();
        manager
    }

    fn load_themes(&mut self) {
        self.available_themes.clear();
        let themes_dir = self.project_path.join("themes");
        
        if !themes_dir.exists() {
            return;
        }
        
        if let Ok(entries) = std::fs::read_dir(&themes_dir) {
            for entry in entries.flatten() {
                // Skip symlinks to prevent following links outside project
                if entry.file_type().map_or(true, |ft| ft.is_symlink()) {
                    continue;
                }
                let path = entry.path();
                
                if path.is_dir() {
                    if let Some(theme) = self.parse_theme(&path) {
                        self.available_themes.push(theme);
                    }
                }
            }
        }
    }

    fn parse_theme(&self, path: &Path) -> Option<Theme> {
        let theme_toml = path.join("theme.toml");
        
        if theme_toml.exists() {
            let content = std::fs::read_to_string(&theme_toml).ok()?;
            let toml_value: toml::Value = content.parse().ok()?;
            
            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("Unknown")
                .to_string();
            
            let description = toml_value
                .get("description")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            
            let author = toml_value
                .get("author")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            
            let version = toml_value
                .get("min_version")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            
            Some(Theme {
                name,
                path: path.to_path_buf(),
                description,
                author,
                version,
            })
        } else {
            // Try to detect theme without theme.toml
            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("Unknown")
                .to_string();
            
            Some(Theme {
                name,
                path: path.to_path_buf(),
                description: String::new(),
                author: String::new(),
                version: String::new(),
            })
        }
    }

    pub fn get_theme_config(&self, theme_name: &str) -> Option<ThemeConfig> {
        let theme = self.available_themes.iter().find(|t| t.name == theme_name)?;
        
        // Look for example site config
        let example_config = theme.path.join("exampleSite").join("config.yaml");
        if example_config.exists() {
            if let Ok(content) = std::fs::read_to_string(&example_config) {
                if let Ok(value) = serde_yaml::from_str::<serde_yaml::Value>(&content) {
                    return Some(ThemeConfig::from_yaml(value));
                }
            }
        }
        
        None
    }

    pub fn list_theme_files(&self, theme_name: &str) -> Vec<PathBuf> {
        let mut files = Vec::new();

        if let Some(theme) = self.available_themes.iter().find(|t| t.name == theme_name) {
            self.scan_theme_files(&theme.path, &mut files, 0);
        }

        files
    }

    const MAX_SCAN_DEPTH: usize = 10;

    fn scan_theme_files(&self, dir: &Path, files: &mut Vec<PathBuf>, depth: usize) {
        if depth > Self::MAX_SCAN_DEPTH {
            return;
        }
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                // Skip symlinks
                if entry.file_type().map_or(true, |ft| ft.is_symlink()) {
                    continue;
                }
                let path = entry.path();

                if path.is_dir() {
                    // Skip exampleSite and other non-essential directories
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        if name != "exampleSite" && name != ".git" && name != "node_modules" {
                            self.scan_theme_files(&path, files, depth + 1);
                        }
                    }
                } else {
                    files.push(path);
                }
            }
        }
    }

    pub fn refresh(&mut self) {
        self.load_themes();
    }
}

#[derive(Debug, Clone)]
pub struct ThemeConfig {
    pub params: std::collections::HashMap<String, serde_yaml::Value>,
}

impl ThemeConfig {
    fn from_yaml(value: serde_yaml::Value) -> Self {
        let mut params = std::collections::HashMap::new();
        
        if let Some(map) = value.as_mapping() {
            for (key, val) in map {
                if let Some(key_str) = key.as_str() {
                    params.insert(key_str.to_string(), val.clone());
                }
            }
        }
        
        Self { params }
    }
}
