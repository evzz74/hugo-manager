use crate::utils::read_file_content;
use chrono::Local;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct Article {
    pub title: String,
    pub date: String,
    pub tags: Vec<String>,
    pub categories: String,
    pub content: String,
    pub draft: bool,
    pub path: PathBuf,
}

pub struct ArticleManager {
    project_path: PathBuf,
    pub articles: Vec<Article>,
}

impl ArticleManager {
    pub fn new(project_path: &Path) -> Self {
        let mut manager = Self {
            project_path: project_path.to_path_buf(),
            articles: Vec::new(),
        };
        manager.load_articles();
        manager
    }

    fn load_articles(&mut self) {
        self.articles.clear();
        let content_dir = self.project_path.join("content");

        if !content_dir.exists() {
            return;
        }

        self.scan_directory(&content_dir, 0);
        self.articles.sort_by(|a, b| b.date.cmp(&a.date));
    }

    const MAX_SCAN_DEPTH: usize = 10;

    fn scan_directory(&mut self, dir: &Path, depth: usize) {
        if depth > Self::MAX_SCAN_DEPTH {
            return;
        }
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();

                // Skip symlinks to prevent infinite loops
                if entry.file_type().map_or(true, |ft| ft.is_symlink()) {
                    continue;
                }

                if path.is_dir() {
                    self.scan_directory(&path, depth + 1);
                } else if path.extension().map_or(false, |ext| ext == "md") {
                    if let Some(article) = self.parse_article(&path) {
                        self.articles.push(article);
                    }
                }
            }
        }
    }

    fn parse_article(&self, path: &Path) -> Option<Article> {
        let content = read_file_content(path).ok()?;
        let (frontmatter, body) = self.split_frontmatter(&content)?;

        let title = self
            .extract_field(&frontmatter, "title")
            .unwrap_or_else(|| "Untitled".to_string());

        let date = self
            .extract_field(&frontmatter, "date")
            .unwrap_or_else(|| Local::now().format("%Y-%m-%d").to_string());

        let tags = self.extract_list_field(&frontmatter, "tags");

        let categories = self
            .extract_field(&frontmatter, "categories")
            .unwrap_or_default();

        let draft = self
            .extract_field(&frontmatter, "draft")
            .map(|d| d == "true")
            .unwrap_or(false);

        Some(Article {
            title,
            date,
            tags,
            categories,
            content: body.to_string(),
            draft,
            path: path.to_path_buf(),
        })
    }

    fn split_frontmatter<'a>(&self, content: &'a str) -> Option<(&'a str, &'a str)> {
        if !content.starts_with("+++") {
            return None;
        }

        let end = content[3..].find("+++")?;
        let frontmatter = &content[3..3 + end];
        let body = &content[3 + end + 3..];

        Some((frontmatter.trim(), body.trim()))
    }

    fn extract_field(&self, frontmatter: &str, field: &str) -> Option<String> {
        for line in frontmatter.lines() {
            let line = line.trim();
            if line.starts_with(&format!("{} =", field)) || line.starts_with(&format!("{}=", field))
            {
                let value = line.split('=').nth(1)?.trim();
                // Remove quotes if present
                let value = value.trim_matches(|c| c == '"' || c == '\'');
                return Some(value.to_string());
            }
        }
        None
    }

    fn extract_list_field(&self, frontmatter: &str, field: &str) -> Vec<String> {
        for line in frontmatter.lines() {
            let line = line.trim();
            if line.starts_with(&format!("{}=", field)) || line.starts_with(&format!("{} =", field))
            {
                let value = line.split('=').nth(1).unwrap_or("").trim();

                // Parse array format: ['tag1', 'tag2'] or ["tag1", "tag2"]
                if value.starts_with('[') && value.ends_with(']') {
                    let inner = &value[1..value.len() - 1];
                    return inner
                        .split(',')
                        .map(|s| s.trim().trim_matches(|c| c == '"' || c == '\'').to_string())
                        .filter(|s| !s.is_empty())
                        .collect();
                }

                return vec![value.to_string()];
            }
        }
        Vec::new()
    }

    /// Escape a string for use in TOML literal (single-quoted) values.
    /// Strips characters that are illegal in TOML literal strings:
    /// single quote, newlines, and all control characters per TOML spec.
    fn escape_toml_value(s: &str) -> String {
        s.chars()
            .filter(|c| {
                // TOML literal strings forbid: ' \n \r and control chars
                // Control chars: U+0000-U+0008, U+000A-U+000C, U+000E-U+001F, U+007F
                // We allow U+0009 (tab) as TOML permits it
                if *c == '\'' || *c == '\n' || *c == '\r' {
                    return false;
                }
                if c.is_control() && *c != '\t' {
                    return false;
                }
                true
            })
            .collect()
    }

    pub fn create_article(
        &mut self,
        title: &str,
        tags: &[String],
        categories: &str,
        content: &str,
        draft: bool,
    ) -> anyhow::Result<PathBuf> {
        let date = Local::now();
        let date_str = date.format("%Y-%m-%dT%H:%M:%S%:z").to_string();

        // Create slug from title
        let slug = self.title_to_slug(title);
        if slug.is_empty() {
            anyhow::bail!("Article title cannot be empty or contain only special characters");
        }

        // Validate the resolved path is still inside content/post/ BEFORE creating anything.
        // We canonicalize the base (which must exist) and do a lexical check on the slug
        // to ensure no traversal components sneak through.
        let canonical_base = self.project_path.join("content").join("post");
        std::fs::create_dir_all(&canonical_base)?;
        let canonical_base = canonical_base.canonicalize()?;
        // Resolve what the final dir would be by joining the slug onto the canonical base
        let expected_dir = canonical_base.join(&slug);
        // Verify slug doesn't contain traversal (.. or absolute components)
        if slug.contains("..") || std::path::Path::new(&slug).is_absolute() {
            anyhow::bail!("Path traversal detected in article slug");
        }
        // Now safe to create
        std::fs::create_dir_all(&expected_dir)?;
        let canonical_dir = expected_dir.canonicalize()?;
        if !canonical_dir.starts_with(&canonical_base) {
            // Clean up the directory we just created if it somehow escaped
            let _ = std::fs::remove_dir(&canonical_dir);
            anyhow::bail!("Path traversal detected: slug resolved outside content/post/");
        }

        let article_path = expected_dir.join("index.md");

        // Generate frontmatter with escaped values
        let safe_title = Self::escape_toml_value(title);
        let mut frontmatter = format!(
            r#"+++
date = '{}'
draft = {}
title = '{}'"#,
            date_str, draft, safe_title
        );

        if !tags.is_empty() {
            let tags_str: Vec<String> = tags.iter()
                .map(|t| format!("'{}'", Self::escape_toml_value(t)))
                .collect();
            frontmatter.push_str(&format!("\ntags=[{}]", tags_str.join(", ")));
        }

        if !categories.is_empty() {
            frontmatter.push_str(&format!(
                "\ncategories='{}'",
                Self::escape_toml_value(categories)
            ));
        }

        frontmatter.push_str("\n+++\n\n");

        let full_content = format!("{}{}", frontmatter, content);
        std::fs::write(&article_path, full_content)?;

        // Reload articles
        self.load_articles();

        Ok(article_path)
    }

    pub fn update_article(
        &mut self,
        path: &Path,
        title: &str,
        tags: &[String],
        categories: &str,
        content: &str,
        draft: bool,
    ) -> anyhow::Result<()> {
        // Always validate path is within project content directory
        self.validate_content_path(path)?;

        // Read existing file to preserve date
        let existing = read_file_content(path)?;
        let (frontmatter, _) = self.split_frontmatter(&existing).unwrap_or(("", ""));

        let date = self
            .extract_field(frontmatter, "date")
            .unwrap_or_else(|| Local::now().format("%Y-%m-%dT%H:%M:%S+08:00").to_string());

        // Generate new frontmatter with escaped values
        let safe_title = Self::escape_toml_value(title);
        let mut new_frontmatter = format!(
            r#"+++
date = '{}'
draft = {}
title = '{}'"#,
            date, draft, safe_title
        );

        if !tags.is_empty() {
            let tags_str: Vec<String> = tags.iter()
                .map(|t| format!("'{}'", Self::escape_toml_value(t)))
                .collect();
            new_frontmatter.push_str(&format!("\ntags=[{}]", tags_str.join(", ")));
        }

        if !categories.is_empty() {
            new_frontmatter.push_str(&format!(
                "\ncategories='{}'",
                Self::escape_toml_value(categories)
            ));
        }

        new_frontmatter.push_str("\n+++\n\n");

        let full_content = format!("{}{}", new_frontmatter, content);
        std::fs::write(path, full_content)?;

        // Reload articles
        self.load_articles();

        Ok(())
    }

    pub fn delete_article(&mut self, path: &Path) -> anyhow::Result<()> {
        if path.exists() {
            // Always validate path is within project content directory
            self.validate_content_path(path)?;

            // Move to trash instead of permanent delete
            if let Some(parent) = path.parent() {
                // Only delete parent dir if it's a leaf article folder (not content/ or post/)
                let parent_name = parent.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if parent_name != "content" && parent_name != "post" && parent != self.project_path {
                    // Extra safety: verify parent is also inside content/
                    self.validate_content_path(parent)?;
                    trash::delete(parent)?;
                } else {
                    // Delete single file
                    trash::delete(path)?;
                }
            }
        }

        // Reload articles
        self.load_articles();

        Ok(())
    }

    fn title_to_slug(&self, title: &str) -> String {
        let slug: String = title
            .chars()
            .map(|c| {
                if c.is_alphanumeric() || c == '-' || c == '_' {
                    c
                } else if c.is_whitespace() {
                    '-'
                } else if c == '.' || c == '/' || c == '\\' {
                    // Strip path separators and dots to prevent traversal
                    '-'
                } else {
                    // For CJK characters, keep them as is
                    c
                }
            })
            .collect::<String>()
            .to_lowercase();

        // Remove any remaining path traversal patterns
        slug.replace("..", "").replace("--", "-").trim_matches('-').to_string()
    }

    /// Validate that a path is within the project's content directory.
    /// Always enforced regardless of whether content dir currently exists.
    fn validate_content_path(&self, path: &Path) -> anyhow::Result<()> {
        let content_base = self.project_path.join("content");

        // Both paths must be canonicalizable (i.e. must exist on disk)
        let canonical_path = path.canonicalize().map_err(|_| {
            anyhow::anyhow!("Cannot resolve path: {}", path.display())
        })?;
        let canonical_base = content_base.canonicalize().map_err(|_| {
            anyhow::anyhow!("Content directory does not exist: {}", content_base.display())
        })?;

        if !canonical_path.starts_with(&canonical_base) {
            anyhow::bail!(
                "Security: path '{}' is outside content directory",
                path.display()
            );
        }
        Ok(())
    }
}
