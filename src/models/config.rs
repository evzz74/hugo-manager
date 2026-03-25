use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HugoConfig {
    pub base_url: String,
    pub language_code: String,
    pub theme: String,
    pub title: String,
    pub copyright: String,
    pub default_language: String,
    pub has_cjk: bool,
    pub sidebar_subtitle: String,
    pub sidebar_emoji: String,
    pub math_enabled: bool,
    pub toc_enabled: bool,
    pub reading_time: bool,
    pub color_scheme: String,
    pub pagination_size: i64,
}

impl Default for HugoConfig {
    fn default() -> Self {
        Self {
            base_url: "https://example.com/".to_string(),
            language_code: "zh-cn".to_string(),
            theme: "hugo-theme-stack".to_string(),
            title: "My Blog".to_string(),
            copyright: String::new(),
            default_language: "zh-cn".to_string(),
            has_cjk: true,
            sidebar_subtitle: String::new(),
            sidebar_emoji: String::new(),
            math_enabled: false,
            toc_enabled: true,
            reading_time: true,
            color_scheme: "auto".to_string(),
            pagination_size: 10,
        }
    }
}

impl HugoConfig {
    pub fn load(project_path: &Path) -> anyhow::Result<Self> {
        let config_path = if project_path.join("hugo.yaml").exists() {
            project_path.join("hugo.yaml")
        } else if project_path.join("hugo.toml").exists() {
            project_path.join("hugo.toml")
        } else if project_path.join("config.yaml").exists() {
            project_path.join("config.yaml")
        } else if project_path.join("config.toml").exists() {
            project_path.join("config.toml")
        } else {
            anyhow::bail!("No Hugo configuration file found");
        };

        let content = std::fs::read_to_string(&config_path)?;
        
        if config_path.extension().map_or(false, |ext| ext == "yaml") {
            let yaml_value: serde_yaml::Value = serde_yaml::from_str(&content)?;
            Ok(Self::from_yaml(yaml_value))
        } else {
            // For TOML, we'd need a proper parser
            Ok(Self::default())
        }
    }

    fn from_yaml(value: serde_yaml::Value) -> Self {
        let map = value.as_mapping().cloned().unwrap_or_default();
        
        Self {
            base_url: map.get("baseurl")
                .and_then(|v| v.as_str())
                .unwrap_or("https://example.com/")
                .to_string(),
            language_code: map.get("languageCode")
                .and_then(|v| v.as_str())
                .unwrap_or("zh-cn")
                .to_string(),
            theme: map.get("theme")
                .and_then(|v| v.as_str())
                .unwrap_or("hugo-theme-stack")
                .to_string(),
            title: map.get("title")
                .and_then(|v| v.as_str())
                .unwrap_or("My Blog")
                .to_string(),
            copyright: map.get("copyright")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            default_language: map.get("DefaultContentLanguage")
                .and_then(|v| v.as_str())
                .unwrap_or("zh-cn")
                .to_string(),
            has_cjk: map.get("hasCJKLanguage")
                .and_then(|v| v.as_bool())
                .unwrap_or(true),
            sidebar_subtitle: Self::get_sidebar_subtitle(&map),
            sidebar_emoji: Self::get_sidebar_emoji(&map),
            math_enabled: Self::get_article_param(&map, "math"),
            toc_enabled: Self::get_article_param(&map, "toc"),
            reading_time: Self::get_article_param(&map, "readingTime"),
            color_scheme: Self::get_color_scheme(&map),
            pagination_size: map.get("pagination")
                .and_then(|p| p.as_mapping())
                .and_then(|m| m.get("pagerSize"))
                .and_then(|v| v.as_i64())
                .unwrap_or(10),
        }
    }

    fn get_sidebar_subtitle(map: &serde_yaml::Mapping) -> String {
        map.get("params")
            .and_then(|p| p.as_mapping())
            .and_then(|m| m.get("sidebar"))
            .and_then(|s| s.as_mapping())
            .and_then(|m| m.get("subtitle"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string()
    }

    fn get_sidebar_emoji(map: &serde_yaml::Mapping) -> String {
        map.get("params")
            .and_then(|p| p.as_mapping())
            .and_then(|m| m.get("sidebar"))
            .and_then(|s| s.as_mapping())
            .and_then(|m| m.get("emoji"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string()
    }

    fn get_article_param(map: &serde_yaml::Mapping, param: &str) -> bool {
        map.get("params")
            .and_then(|p| p.as_mapping())
            .and_then(|m| m.get("article"))
            .and_then(|a| a.as_mapping())
            .and_then(|m| m.get(param))
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
    }

    fn get_color_scheme(map: &serde_yaml::Mapping) -> String {
        map.get("params")
            .and_then(|p| p.as_mapping())
            .and_then(|m| m.get("colorScheme"))
            .and_then(|c| c.as_mapping())
            .and_then(|m| m.get("default"))
            .and_then(|v| v.as_str())
            .unwrap_or("auto")
            .to_string()
    }

    pub fn save(&self, project_path: &Path) -> anyhow::Result<()> {
        let config_path = project_path.join("hugo.yaml");

        // Load existing YAML to preserve unknown fields
        let mut yaml = if config_path.exists() {
            let existing = std::fs::read_to_string(&config_path)?;
            match serde_yaml::from_str::<serde_yaml::Value>(&existing) {
                Ok(serde_yaml::Value::Mapping(m)) => m,
                _ => serde_yaml::Mapping::new(),
            }
        } else {
            serde_yaml::Mapping::new()
        };

        // Update only managed fields
        yaml.insert(
            serde_yaml::Value::String("baseurl".to_string()),
            serde_yaml::Value::String(self.base_url.clone()),
        );
        yaml.insert(
            serde_yaml::Value::String("languageCode".to_string()),
            serde_yaml::Value::String(self.language_code.clone()),
        );
        yaml.insert(
            serde_yaml::Value::String("theme".to_string()),
            serde_yaml::Value::String(self.theme.clone()),
        );
        yaml.insert(
            serde_yaml::Value::String("title".to_string()),
            serde_yaml::Value::String(self.title.clone()),
        );
        yaml.insert(
            serde_yaml::Value::String("copyright".to_string()),
            serde_yaml::Value::String(self.copyright.clone()),
        );
        yaml.insert(
            serde_yaml::Value::String("DefaultContentLanguage".to_string()),
            serde_yaml::Value::String(self.default_language.clone()),
        );
        yaml.insert(
            serde_yaml::Value::String("hasCJKLanguage".to_string()),
            serde_yaml::Value::Bool(self.has_cjk),
        );

        // Merge into existing params, preserving unknown sub-keys
        let mut params = yaml
            .get("params")
            .and_then(|v| v.as_mapping().cloned())
            .unwrap_or_default();

        // Sidebar - merge
        let mut sidebar = params
            .get("sidebar")
            .and_then(|v| v.as_mapping().cloned())
            .unwrap_or_default();
        sidebar.insert(
            serde_yaml::Value::String("subtitle".to_string()),
            serde_yaml::Value::String(self.sidebar_subtitle.clone()),
        );
        sidebar.insert(
            serde_yaml::Value::String("emoji".to_string()),
            serde_yaml::Value::String(self.sidebar_emoji.clone()),
        );
        params.insert(
            serde_yaml::Value::String("sidebar".to_string()),
            serde_yaml::Value::Mapping(sidebar),
        );

        // Article settings - merge
        let mut article = params
            .get("article")
            .and_then(|v| v.as_mapping().cloned())
            .unwrap_or_default();
        article.insert(
            serde_yaml::Value::String("math".to_string()),
            serde_yaml::Value::Bool(self.math_enabled),
        );
        article.insert(
            serde_yaml::Value::String("toc".to_string()),
            serde_yaml::Value::Bool(self.toc_enabled),
        );
        article.insert(
            serde_yaml::Value::String("readingTime".to_string()),
            serde_yaml::Value::Bool(self.reading_time),
        );
        params.insert(
            serde_yaml::Value::String("article".to_string()),
            serde_yaml::Value::Mapping(article),
        );

        // Color scheme - merge
        let mut color_scheme = params
            .get("colorScheme")
            .and_then(|v| v.as_mapping().cloned())
            .unwrap_or_default();
        color_scheme.insert(
            serde_yaml::Value::String("default".to_string()),
            serde_yaml::Value::String(self.color_scheme.clone()),
        );
        params.insert(
            serde_yaml::Value::String("colorScheme".to_string()),
            serde_yaml::Value::Mapping(color_scheme),
        );

        yaml.insert(
            serde_yaml::Value::String("params".to_string()),
            serde_yaml::Value::Mapping(params),
        );

        // Pagination - merge
        let mut pagination = yaml
            .get("pagination")
            .and_then(|v| v.as_mapping().cloned())
            .unwrap_or_default();
        pagination.insert(
            serde_yaml::Value::String("pagerSize".to_string()),
            serde_yaml::Value::Number(self.pagination_size.into()),
        );
        yaml.insert(
            serde_yaml::Value::String("pagination".to_string()),
            serde_yaml::Value::Mapping(pagination),
        );

        let yaml_str = serde_yaml::to_string(&serde_yaml::Value::Mapping(yaml))?;

        // Atomic write: write to temp file, then rename to avoid corruption on crash
        let tmp_path = config_path.with_extension("yaml.tmp");
        std::fs::write(&tmp_path, &yaml_str)?;
        if let Err(e) = std::fs::rename(&tmp_path, &config_path) {
            // Clean up temp file if rename fails
            let _ = std::fs::remove_file(&tmp_path);
            return Err(e.into());
        }

        Ok(())
    }
}
