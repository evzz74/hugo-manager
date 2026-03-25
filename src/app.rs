use eframe::egui;
use std::path::PathBuf;

use crate::models::{
    article::ArticleManager, config::HugoConfig, theme::ThemeManager,
};

pub struct HugoApp {
    // Project paths
    project_path: PathBuf,

    // Managers
    config: HugoConfig,
    article_manager: ArticleManager,
    theme_manager: ThemeManager,

    // UI state
    current_tab: Tab,
    status_message: String,
    show_error: bool,
    error_message: String,

    // Article editing state
    editing_article: Option<EditingArticle>,
    show_article_editor: bool,

    // Article delete confirmation state
    show_delete_confirm: bool,
    delete_article_path: Option<PathBuf>,
    delete_article_title: String,

    // Article pagination & search
    articles_page: usize,
    articles_per_page: usize,
    articles_search: String,

    // Theme editing state
    _selected_theme: Option<String>,

    // Context for repaint
    _ctx: egui::Context,
}

#[derive(PartialEq, Clone, Copy)]
enum Tab {
    Dashboard,
    Articles,
    Themes,
    Settings,
}

struct EditingArticle {
    title: String,
    tags: String,
    categories: String,
    content: String,
    draft: bool,
    original_path: Option<PathBuf>,
}

impl HugoApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let project_path = Self::find_hugo_project()
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());

        let config = HugoConfig::load(&project_path).unwrap_or_default();
        let article_manager = ArticleManager::new(&project_path);
        let theme_manager = ThemeManager::new(&project_path);

        Self {
            project_path,
            config,
            article_manager,
            theme_manager,
            current_tab: Tab::Dashboard,
            status_message: String::new(),
            show_error: false,
            error_message: String::new(),
            editing_article: None,
            show_article_editor: false,
            show_delete_confirm: false,
            delete_article_path: None,
            delete_article_title: String::new(),
            articles_page: 0,
            articles_per_page: 15,
            articles_search: String::new(),
            _selected_theme: None,
            _ctx: cc.egui_ctx.clone(),
        }
    }

    fn find_hugo_project() -> Option<PathBuf> {
        let mut current = std::env::current_dir().ok()?;

        loop {
            if current.join("hugo.yaml").exists()
                || current.join("hugo.toml").exists()
                || current.join("hugo.json").exists()
                || current.join("config.yaml").exists()
                || current.join("config.toml").exists()
            {
                return Some(current);
            }

            if !current.pop() {
                break;
            }
        }

        None
    }

    fn card_frame(ui: &egui::Ui) -> egui::Frame {
        egui::Frame::default()
            .inner_margin(16.0)
            .rounding(8.0)
            .fill(ui.visuals().window_fill)
            .stroke(ui.visuals().window_stroke)
    }

    fn show_dashboard(&mut self, ui: &mut egui::Ui) {
        ui.heading("Hugo Blog Manager");
        ui.add_space(16.0);

        // Project Overview Card (full width)
        Self::card_frame(ui).show(ui, |ui| {
            ui.label(egui::RichText::new("Project Overview").strong().size(15.0));
            ui.add_space(8.0);

            ui.horizontal(|ui| {
                ui.label("Project Path:");
                ui.label(
                    egui::RichText::new(self.project_path.display().to_string())
                        .monospace(),
                );
                if ui.button("Change...").clicked() {
                    self.select_project_path();
                }
            });

            ui.add_space(6.0);

            egui::Grid::new("site_info_grid")
                .num_columns(2)
                .spacing([12.0, 4.0])
                .show(ui, |ui| {
                    ui.label("Site Title:");
                    ui.label(egui::RichText::new(&self.config.title).strong());
                    ui.end_row();

                    ui.label("Theme:");
                    ui.label(&self.config.theme);
                    ui.end_row();

                    ui.label("Base URL:");
                    ui.label(
                        egui::RichText::new(&self.config.base_url).monospace(),
                    );
                    ui.end_row();
                });
        });

        ui.add_space(12.0);

        // Stats row: Articles + Themes side by side
        ui.horizontal(|ui| {
            let half_width = (ui.available_width() - 12.0) / 2.0;

            // Articles card
            ui.allocate_ui(egui::vec2(half_width, 80.0), |ui| {
                Self::card_frame(ui).show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new(
                                self.article_manager.articles.len().to_string(),
                            )
                            .size(28.0)
                            .strong()
                            .color(egui::Color32::from_rgb(59, 130, 246)),
                        );
                        ui.label(egui::RichText::new("Articles").size(15.0));
                    });
                });
            });

            ui.add_space(12.0);

            // Themes card
            ui.allocate_ui(egui::vec2(half_width, 80.0), |ui| {
                Self::card_frame(ui).show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new(
                                self.theme_manager.available_themes.len().to_string(),
                            )
                            .size(28.0)
                            .strong()
                            .color(egui::Color32::from_rgb(16, 185, 129)),
                        );
                        ui.label(egui::RichText::new("Themes").size(15.0));
                    });
                });
            });
        });

        ui.add_space(12.0);

        // Quick Actions Card
        Self::card_frame(ui).show(ui, |ui| {
            ui.label(egui::RichText::new("Quick Actions").strong().size(15.0));
            ui.add_space(10.0);

            let blue = egui::Color32::from_rgb(59, 130, 246);
            let white = egui::Color32::WHITE;
            let btn_rounding = egui::Rounding::same(6.0);

            ui.horizontal(|ui| {
                let browser_btn = egui::Button::new(
                    egui::RichText::new("\u{1F310}  Open in Browser").color(white),
                )
                .fill(blue)
                .rounding(btn_rounding);
                if ui.add(browser_btn).clicked() {
                    let url = &self.config.base_url;
                    let is_safe_url = (url.starts_with("http://") || url.starts_with("https://"))
                        && !url.contains(|c: char| c.is_control());
                    if is_safe_url {
                        let _ = open::that(url);
                    } else {
                        self.show_error_with_message(
                            "Invalid base URL: must start with http:// or https://",
                        );
                    }
                }

                let server_btn = egui::Button::new(
                    egui::RichText::new("\u{26A1}  Hugo Server").color(white),
                )
                .fill(blue)
                .rounding(btn_rounding);
                if ui.add(server_btn).clicked() {
                    self.start_hugo_server();
                }

                let build_btn = egui::Button::new(
                    egui::RichText::new("\u{1F528}  Build Site").color(white),
                )
                .fill(blue)
                .rounding(btn_rounding);
                if ui.add(build_btn).clicked() {
                    self.build_site();
                }
            });
        });
    }

    fn show_articles(&mut self, ui: &mut egui::Ui) {
        let blue = egui::Color32::from_rgb(59, 130, 246);
        let white = egui::Color32::WHITE;
        let btn_rounding = egui::Rounding::same(6.0);
        let toolbar_bg = ui.visuals().window_fill();
        let separator_color = egui::Color32::from_rgb(234, 236, 240);
        let tag_bg = egui::Color32::from_rgb(241, 243, 245);
        let icon_gray = egui::Color32::from_rgb(107, 114, 128);
        let _icon_hover = egui::Color32::from_rgb(55, 65, 81);

        // ── Filter articles by search ──
        let all_articles = self.article_manager.articles.clone();
        let search_lower = self.articles_search.to_lowercase();
        let filtered: Vec<_> = if search_lower.is_empty() {
            all_articles
        } else {
            all_articles
                .into_iter()
                .filter(|a| {
                    a.title.to_lowercase().contains(&search_lower)
                        || a.categories.to_lowercase().contains(&search_lower)
                        || a.tags.iter().any(|t| t.to_lowercase().contains(&search_lower))
                })
                .collect()
        };

        let total = filtered.len();
        let total_pages = if total == 0 {
            1
        } else {
            (total + self.articles_per_page - 1) / self.articles_per_page
        };
        if self.articles_page >= total_pages {
            self.articles_page = total_pages.saturating_sub(1);
        }
        let start = self.articles_page * self.articles_per_page;
        let end = (start + self.articles_per_page).min(total);
        let page_articles = if total > 0 {
            &filtered[start..end]
        } else {
            &[][..]
        };

        // ── Sticky Toolbar (top) ──
        egui::Frame::default()
            .inner_margin(egui::Margin::symmetric(24.0, 10.0))
            .fill(toolbar_bg)
            .stroke(egui::Stroke::new(0.5, separator_color))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    // Search box (left, capped at 400px)
                    let search_width = (ui.available_width() - 240.0).max(200.0).min(400.0);
                    ui.add(
                        egui::TextEdit::singleline(&mut self.articles_search)
                            .desired_width(search_width)
                            .hint_text("\u{1F50D}  Search articles..."),
                    );

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.add_space(6.0);

                        // "+ New Article" primary button
                        let new_btn = egui::Button::new(
                            egui::RichText::new("+ New Article").color(white),
                        )
                        .fill(blue)
                        .rounding(btn_rounding)
                        .min_size(egui::vec2(120.0, 30.0));
                        if ui.add(new_btn).clicked() {
                            self.editing_article = Some(EditingArticle {
                                title: String::new(),
                                tags: String::new(),
                                categories: String::new(),
                                content: String::new(),
                                draft: true,
                                original_path: None,
                            });
                            self.show_article_editor = true;
                        }
                    });
                });
            });

        // ── Full-bleed Data Table (manual fixed-width layout) ──
        let col_gap = 16.0;
        let row_h = 28.0;
        let header_h = 36.0;
        let footer_height = 40.0;

        // Pre-compute column widths BEFORE any layout
        let table_width = ui.available_width() - 48.0; // 24px padding each side
        let cw = Self::article_col_widths(table_width, col_gap);

        let header_color = egui::Color32::from_rgb(249, 250, 251);
        let stripe_color = egui::Color32::from_rgb(249, 250, 251);
        let weak = ui.visuals().weak_text_color();

        // ── Sticky Header Row (outside ScrollArea) ──
        egui::Frame::default()
            .inner_margin(egui::Margin::symmetric(24.0, 0.0))
            .fill(header_color)
            .stroke(egui::Stroke::new(0.5, separator_color))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = col_gap;
                    let header_labels = ["Title", "Date", "Categories", "Tags", "Status", "Actions"];
                    for (j, label) in header_labels.iter().enumerate() {
                        ui.allocate_ui(egui::vec2(cw[j], header_h), |ui| {
                            ui.set_min_size(egui::vec2(cw[j], header_h));
                            ui.add(egui::Label::new(
                                egui::RichText::new(*label).strong().color(weak),
                            ));
                        });
                    }
                });
            });

        // ── Scrollable Data Rows (middle, fills remaining minus footer) ──
        let available_height = ui.available_height();
        let table_height = (available_height - footer_height).max(60.0);

        ui.allocate_ui(egui::vec2(ui.available_width(), table_height), |ui| {
            egui::ScrollArea::vertical()
                .id_source("articles_table_scroll")
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    egui::Frame::default()
                        .inner_margin(egui::Margin::symmetric(24.0, 0.0))
                        .show(ui, |ui| {
                            if page_articles.is_empty() {
                                ui.add_space(20.0);
                                ui.label(
                                    egui::RichText::new("No articles found")
                                        .color(weak)
                                        .size(14.0),
                                );
                            } else {
                                for (i, article) in page_articles.iter().enumerate() {
                                    // Stripe every other row
                                    if i % 2 == 1 {
                                        let row_rect = ui.available_rect_before_wrap();
                                        let row_rect = egui::Rect::from_min_size(
                                            row_rect.min,
                                            egui::vec2(row_rect.width(), row_h),
                                        );
                                        ui.painter().rect_filled(row_rect, 0.0, stripe_color);
                                    }

                                    ui.horizontal(|ui| {
                                        ui.spacing_mut().item_spacing.x = col_gap;

                                        // Title
                                        ui.allocate_ui(egui::vec2(cw[0], row_h), |ui| {
                                            ui.set_min_size(egui::vec2(cw[0], row_h));
                                            ui.add(egui::Label::new(
                                                egui::RichText::new(&article.title).strong(),
                                            ).truncate());
                                        });
                                        // Date
                                        ui.allocate_ui(egui::vec2(cw[1], row_h), |ui| {
                                            ui.set_min_size(egui::vec2(cw[1], row_h));
                                            ui.add(egui::Label::new(
                                                egui::RichText::new(&article.date)
                                                    .small()
                                                    .color(weak),
                                            ).truncate());
                                        });
                                        // Categories
                                        ui.allocate_ui(egui::vec2(cw[2], row_h), |ui| {
                                            ui.set_min_size(egui::vec2(cw[2], row_h));
                                            ui.add(egui::Label::new(
                                                egui::RichText::new(&article.categories)
                                                    .small()
                                                    .color(weak),
                                            ).truncate());
                                        });
                                        // Tags
                                        ui.allocate_ui(egui::vec2(cw[3], row_h), |ui| {
                                            ui.set_min_size(egui::vec2(cw[3], row_h));
                                            ui.add(egui::Label::new(
                                                egui::RichText::new(article.tags.join(", "))
                                                    .small()
                                                    .background_color(tag_bg),
                                            ).truncate());
                                        });
                                        // Status
                                        ui.allocate_ui(egui::vec2(cw[4], row_h), |ui| {
                                            ui.set_min_size(egui::vec2(cw[4], row_h));
                                            ui.add(egui::Label::new(if article.draft {
                                                egui::RichText::new(" Draft ")
                                                    .small()
                                                    .color(egui::Color32::from_rgb(161, 98, 7))
                                                    .background_color(egui::Color32::from_rgb(254, 243, 199))
                                            } else {
                                                egui::RichText::new(" Published ")
                                                    .small()
                                                    .color(egui::Color32::from_rgb(21, 128, 61))
                                                    .background_color(egui::Color32::from_rgb(220, 252, 231))
                                            }));
                                        });
                                        // Actions
                                        ui.allocate_ui(egui::vec2(cw[5], row_h), |ui| {
                                            ui.set_min_size(egui::vec2(cw[5], row_h));
                                            ui.horizontal(|ui| {
                                                ui.spacing_mut().item_spacing.x = 12.0;
                                                if ui
                                                    .add(
                                                        egui::Button::new(
                                                            egui::RichText::new("\u{270F}")
                                                                .size(14.0)
                                                                .color(icon_gray),
                                                        )
                                                        .frame(false),
                                                    )
                                                    .on_hover_text("Edit")
                                                    .clicked()
                                                {
                                                    self.editing_article = Some(EditingArticle {
                                                        title: article.title.clone(),
                                                        tags: article.tags.join(", "),
                                                        categories: article.categories.clone(),
                                                        content: article.content.clone(),
                                                        draft: article.draft,
                                                        original_path: Some(article.path.clone()),
                                                    });
                                                    self.show_article_editor = true;
                                                }
                                                if ui
                                                    .add(
                                                        egui::Button::new(
                                                            egui::RichText::new("\u{1F441}")
                                                                .size(14.0)
                                                                .color(icon_gray),
                                                        )
                                                        .frame(false),
                                                    )
                                                    .on_hover_text("Preview")
                                                    .clicked()
                                                {
                                                    // Only open .md files within the project
                                                    let article_path = &article.path;
                                                    let is_md = article_path.extension()
                                                        .map_or(false, |ext| ext == "md");
                                                    let is_in_project = article_path
                                                        .canonicalize()
                                                        .ok()
                                                        .and_then(|p| {
                                                            self.project_path.canonicalize().ok().map(|base| p.starts_with(base))
                                                        })
                                                        .unwrap_or(false);
                                                    if is_md && is_in_project {
                                                        if let Some(path) = article_path.to_str() {
                                                            let _ = open::that(path);
                                                        }
                                                    }
                                                }
                                                if ui
                                                    .add(
                                                        egui::Button::new(
                                                            egui::RichText::new("\u{1F5D1}")
                                                                .size(14.0)
                                                                .color(icon_gray),
                                                        )
                                                        .frame(false),
                                                    )
                                                    .on_hover_text("Delete")
                                                    .clicked()
                                                {
                                                    self.show_delete_confirm = true;
                                                    self.delete_article_path =
                                                        Some(article.path.clone());
                                                    self.delete_article_title =
                                                        article.title.clone();
                                                }
                                            });
                                        });
                                    });
                                }
                            }
                        });
                });
        });

        // ── Sticky Pagination Footer ──
        egui::Frame::default()
            .inner_margin(egui::Margin::symmetric(24.0, 8.0))
            .fill(toolbar_bg)
            .stroke(egui::Stroke::new(0.5, separator_color))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    // Left: stats
                    if total > 0 {
                        ui.label(
                            egui::RichText::new(format!(
                                "Showing {}-{} of {} articles",
                                start + 1,
                                end,
                                total
                            ))
                            .small()
                            .color(ui.visuals().weak_text_color()),
                        );
                    } else {
                        ui.label(
                            egui::RichText::new("No articles")
                                .small()
                                .color(ui.visuals().weak_text_color()),
                        );
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let can_next = self.articles_page + 1 < total_pages;
                        let can_prev = self.articles_page > 0;

                        if ui
                            .add_enabled(can_next, egui::Button::new("\u{25B6}").rounding(btn_rounding))
                            .on_hover_text("Next page")
                            .clicked()
                        {
                            self.articles_page += 1;
                        }

                        ui.label(
                            egui::RichText::new(format!(
                                "{} / {}",
                                self.articles_page + 1,
                                total_pages
                            ))
                            .small(),
                        );

                        if ui
                            .add_enabled(can_prev, egui::Button::new("\u{25C0}").rounding(btn_rounding))
                            .on_hover_text("Previous page")
                            .clicked()
                        {
                            self.articles_page -= 1;
                        }
                    });
                });
            });

        // Modals
        if self.show_article_editor {
            self.show_article_editor_modal(ui);
        }
        if self.show_delete_confirm {
            self.show_delete_confirm_modal(ui);
        }
    }

    /// Compute proportional column widths for the articles table.
    fn article_col_widths(available: f32, gap: f32) -> [f32; 6] {
        let status_w = 64.0_f32;
        let actions_w = 90.0_f32;
        let flex_total = (available - gap * 5.0 - status_w - actions_w).max(100.0);
        // Title 5fr, Date 4fr, Categories 3fr, Tags 5fr = 17fr total
        let fr = flex_total / 17.0;
        [
            fr * 5.0,    // Title
            fr * 4.0,    // Date
            fr * 3.0,    // Categories
            fr * 5.0,    // Tags
            status_w,    // Status (fixed)
            actions_w,   // Actions (fixed)
        ]
    }

    fn show_article_editor_modal(&mut self, ui: &mut egui::Ui) {
        egui::Window::new("Article Editor")
            .collapsible(false)
            .resizable(true)
            .show(ui.ctx(), |ui| {
                if let Some(article) = &mut self.editing_article {
                    ui.horizontal(|ui| {
                        ui.label("Title:");
                        ui.text_edit_singleline(&mut article.title);
                    });

                    ui.horizontal(|ui| {
                        ui.label("Tags:");
                        ui.text_edit_singleline(&mut article.tags);
                        ui.label("(comma separated)");
                    });

                    ui.horizontal(|ui| {
                        ui.label("Categories:");
                        ui.text_edit_singleline(&mut article.categories);
                    });

                    ui.checkbox(&mut article.draft, "Draft");

                    ui.add_space(10.0);
                    ui.label("Content:");

                    egui::ScrollArea::vertical()
                        .max_height(400.0)
                        .show(ui, |ui| {
                            ui.add(
                                egui::TextEdit::multiline(&mut article.content)
                                    .desired_width(f32::INFINITY)
                                    .desired_rows(20),
                            );
                        });

                    ui.add_space(10.0);
                    ui.horizontal(|ui| {
                        if ui.button("Save").clicked() {
                            self.save_article();
                        }
                        if ui.button("Cancel").clicked() {
                            self.show_article_editor = false;
                            self.editing_article = None;
                        }
                    });
                }
            });
    }

    fn save_article(&mut self) {
        if let Some(article) = &self.editing_article {
            let tags: Vec<String> = article
                .tags
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();

            let result: anyhow::Result<()> = if let Some(path) = &article.original_path {
                self.article_manager.update_article(
                    path,
                    &article.title,
                    &tags,
                    &article.categories,
                    &article.content,
                    article.draft,
                )
            } else {
                self.article_manager
                    .create_article(
                        &article.title,
                        &tags,
                        &article.categories,
                        &article.content,
                        article.draft,
                    )
                    .map(|_| ())
            };

            match result {
                Ok(_) => {
                    self.status_message = "Article saved successfully".to_string();
                    self.show_article_editor = false;
                    self.editing_article = None;
                }
                Err(e) => {
                    self.show_error_with_message(&format!("Failed to save: {}", e));
                }
            }
        }
    }

    fn show_delete_confirm_modal(&mut self, ui: &mut egui::Ui) {
        egui::Window::new("Confirm Delete")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ui.ctx(), |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(10.0);
                    ui.label(format!("Are you sure you want to delete:"));
                    ui.add_space(5.0);
                    ui.colored_label(
                        egui::Color32::from_rgb(200, 50, 50),
                        &self.delete_article_title,
                    );
                    ui.add_space(5.0);
                    ui.label("The article will be moved to Recycle Bin.");
                    ui.add_space(15.0);

                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 20.0;

                        if ui.button("Cancel").clicked() {
                            self.show_delete_confirm = false;
                            self.delete_article_path = None;
                            self.delete_article_title.clear();
                        }

                        if ui.button("Delete").clicked() {
                            if let Some(path) = &self.delete_article_path {
                                match self.article_manager.delete_article(path) {
                                    Ok(_) => {
                                        self.status_message = format!(
                                            "Article '{}' deleted",
                                            self.delete_article_title
                                        );
                                    }
                                    Err(e) => {
                                        self.show_error_with_message(&format!(
                                            "Failed to delete: {}",
                                            e
                                        ));
                                    }
                                }
                            }
                            self.show_delete_confirm = false;
                            self.delete_article_path = None;
                            self.delete_article_title.clear();
                        }
                    });
                    ui.add_space(10.0);
                });
            });
    }

    fn show_themes(&mut self, ui: &mut egui::Ui) {
        let blue = egui::Color32::from_rgb(59, 130, 246);
        let white = egui::Color32::WHITE;
        let btn_rounding = egui::Rounding::same(6.0);

        ui.heading("Theme Gallery");
        ui.add_space(12.0);

        let themes = self.theme_manager.available_themes.clone();
        let active_theme = self.config.theme.clone();

        egui::ScrollArea::vertical().show(ui, |ui| {
            // Active theme card (highlighted)
            if let Some(theme) = themes.iter().find(|t| t.name == active_theme) {
                egui::Frame::default()
                    .inner_margin(16.0)
                    .rounding(8.0)
                    .fill(ui.visuals().window_fill)
                    .stroke(egui::Stroke::new(2.0, blue))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            // Thumbnail placeholder
                            let (rect, _) = ui.allocate_exact_size(
                                egui::vec2(80.0, 60.0),
                                egui::Sense::hover(),
                            );
                            ui.painter().rect_filled(
                                rect,
                                4.0,
                                egui::Color32::from_rgb(30, 58, 138),
                            );
                            ui.painter().text(
                                rect.center(),
                                egui::Align2::CENTER_CENTER,
                                "\u{1F3A8}",
                                egui::FontId::proportional(24.0),
                                white,
                            );

                            ui.vertical(|ui| {
                                ui.horizontal(|ui| {
                                    ui.label(
                                        egui::RichText::new(&theme.name).strong().size(16.0),
                                    );
                                    ui.label(
                                        egui::RichText::new(" Active ")
                                            .color(white)
                                            .background_color(blue),
                                    );
                                });
                                if !theme.version.is_empty() {
                                    ui.label(
                                        egui::RichText::new(format!("v{}", theme.version))
                                            .small()
                                            .color(ui.visuals().weak_text_color()),
                                    );
                                }
                                if !theme.author.is_empty() {
                                    ui.label(
                                        egui::RichText::new(format!("by {}", theme.author))
                                            .small()
                                            .color(ui.visuals().weak_text_color()),
                                    );
                                }
                                if !theme.description.is_empty() {
                                    ui.label(
                                        egui::RichText::new(&theme.description)
                                            .small()
                                            .color(ui.visuals().weak_text_color()),
                                    );
                                }

                                ui.add_space(6.0);
                                ui.horizontal(|ui| {
                                    if ui
                                        .add(
                                            egui::Button::new("Deactivate")
                                                .rounding(btn_rounding),
                                        )
                                        .clicked()
                                    {
                                        self.config.theme = String::new();
                                        if let Err(e) = self.config.save(&self.project_path) {
                                            self.show_error_with_message(&format!(
                                                "Failed to save config: {}",
                                                e
                                            ));
                                        } else {
                                            self.status_message =
                                                "Theme deactivated".to_string();
                                        }
                                    }

                                    // Color scheme selector as "Configure"
                                    egui::ComboBox::from_id_source("theme_color_scheme")
                                        .selected_text(format!(
                                            "Color: {}",
                                            &self.config.color_scheme
                                        ))
                                        .show_ui(ui, |ui| {
                                            ui.selectable_value(
                                                &mut self.config.color_scheme,
                                                "auto".to_string(),
                                                "Auto",
                                            );
                                            ui.selectable_value(
                                                &mut self.config.color_scheme,
                                                "light".to_string(),
                                                "Light",
                                            );
                                            ui.selectable_value(
                                                &mut self.config.color_scheme,
                                                "dark".to_string(),
                                                "Dark",
                                            );
                                        });
                                });
                            });
                        });
                    });
            }

            ui.add_space(16.0);
            ui.label(egui::RichText::new("Available Themes").strong().size(15.0));
            ui.add_space(8.0);

            // Grid of other themes + "Add New" card
            let card_width = 180.0_f32;
            let available = ui.available_width();
            let cols = ((available / (card_width + 12.0)) as usize).max(1);

            // Collect non-active themes
            let other_themes: Vec<_> =
                themes.iter().filter(|t| t.name != active_theme).collect();

            // Render in grid rows
            let total_items = other_themes.len() + 1; // +1 for "Add New"
            let rows = (total_items + cols - 1) / cols;

            for row in 0..rows {
                ui.horizontal(|ui| {
                    for col in 0..cols {
                        let idx = row * cols + col;

                        if idx < other_themes.len() {
                            let theme = &other_themes[idx];
                            ui.allocate_ui(egui::vec2(card_width, 170.0), |ui| {
                                Self::card_frame(ui).show(ui, |ui| {
                                    ui.set_min_size(egui::vec2(card_width - 32.0, 130.0));

                                    // Thumbnail placeholder
                                    let (rect, _) = ui.allocate_exact_size(
                                        egui::vec2(card_width - 40.0, 50.0),
                                        egui::Sense::hover(),
                                    );
                                    ui.painter().rect_filled(
                                        rect,
                                        4.0,
                                        egui::Color32::from_rgb(229, 231, 235),
                                    );
                                    ui.painter().text(
                                        rect.center(),
                                        egui::Align2::CENTER_CENTER,
                                        "\u{1F3A8}",
                                        egui::FontId::proportional(20.0),
                                        egui::Color32::from_rgb(156, 163, 175),
                                    );

                                    ui.add_space(4.0);
                                    ui.label(
                                        egui::RichText::new(&theme.name).strong(),
                                    );
                                    if !theme.version.is_empty() {
                                        ui.label(
                                            egui::RichText::new(format!(
                                                "v{}",
                                                theme.version
                                            ))
                                            .small()
                                            .color(ui.visuals().weak_text_color()),
                                        );
                                    }

                                    ui.add_space(4.0);
                                    ui.horizontal(|ui| {
                                        let activate_btn = egui::Button::new(
                                            egui::RichText::new("Activate").color(white),
                                        )
                                        .fill(blue)
                                        .rounding(btn_rounding);
                                        if ui.add(activate_btn).clicked() {
                                            self.config.theme = theme.name.clone();
                                            if let Err(e) =
                                                self.config.save(&self.project_path)
                                            {
                                                self.show_error_with_message(&format!(
                                                    "Failed to save: {}",
                                                    e
                                                ));
                                            } else {
                                                self.status_message = format!(
                                                    "Theme changed to {}",
                                                    theme.name
                                                );
                                            }
                                        }

                                        if ui
                                            .add(
                                                egui::Button::new(
                                                    egui::RichText::new("\u{1F5D1}")
                                                        .size(14.0),
                                                )
                                                .rounding(btn_rounding),
                                            )
                                            .on_hover_text("Delete theme")
                                            .clicked()
                                        {
                                            // Validate theme path is within project themes dir
                                            let themes_base = self.project_path.join("themes");
                                            let is_safe = theme.path.canonicalize().ok()
                                                .and_then(|tp| {
                                                    themes_base.canonicalize().ok().map(|tb| tp.starts_with(tb))
                                                })
                                                .unwrap_or(false);
                                            if !is_safe {
                                                self.show_error_with_message(
                                                    "Cannot delete: theme path is outside project themes directory"
                                                );
                                            } else {
                                                match trash::delete(&theme.path) {
                                                    Ok(_) => {
                                                        self.status_message = format!(
                                                            "Theme '{}' deleted",
                                                            theme.name
                                                        );
                                                        self.theme_manager =
                                                            ThemeManager::new(&self.project_path);
                                                    }
                                                    Err(e) => {
                                                        self.show_error_with_message(&format!(
                                                            "Failed to delete theme: {}",
                                                            e
                                                        ));
                                                    }
                                                }
                                            }
                                        }
                                    });
                                });
                            });
                        } else if idx == other_themes.len() {
                            // "Add New Theme" card with dashed border
                            ui.allocate_ui(egui::vec2(card_width, 170.0), |ui| {
                                let (rect, response) = ui.allocate_exact_size(
                                    egui::vec2(card_width - 4.0, 162.0),
                                    egui::Sense::click(),
                                );

                                // Dashed border effect (dotted rectangle)
                                let stroke =
                                    egui::Stroke::new(1.5, egui::Color32::from_rgb(156, 163, 175));
                                let rounding = egui::Rounding::same(8.0);
                                ui.painter().rect_stroke(rect, rounding, stroke);

                                // "+" icon
                                ui.painter().text(
                                    rect.center() - egui::vec2(0.0, 14.0),
                                    egui::Align2::CENTER_CENTER,
                                    "+",
                                    egui::FontId::proportional(36.0),
                                    egui::Color32::from_rgb(156, 163, 175),
                                );
                                // Label
                                ui.painter().text(
                                    rect.center() + egui::vec2(0.0, 20.0),
                                    egui::Align2::CENTER_CENTER,
                                    "Add New Theme",
                                    egui::FontId::proportional(13.0),
                                    egui::Color32::from_rgb(156, 163, 175),
                                );

                                if response.clicked() {
                                    // Open themes directory for user to add themes
                                    let themes_dir = self.project_path.join("themes");
                                    if themes_dir.exists() {
                                        let _ = open::that(&themes_dir);
                                    } else {
                                        let _ = std::fs::create_dir_all(&themes_dir);
                                        let _ = open::that(&themes_dir);
                                    }
                                    self.status_message =
                                        "Opened themes folder - add your theme here"
                                            .to_string();
                                }

                                if response.hovered() {
                                    ui.painter().rect_filled(
                                        rect,
                                        rounding,
                                        egui::Color32::from_rgba_premultiplied(59, 130, 246, 15),
                                    );
                                }
                            });
                        }
                    }
                });
            }
        });
    }

    fn show_settings(&mut self, ui: &mut egui::Ui) {
        let blue = egui::Color32::from_rgb(59, 130, 246);
        let white = egui::Color32::WHITE;
        let btn_rounding = egui::Rounding::same(6.0);

        // Header row: title (left) + save button (right)
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Site Settings").heading());
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let save_btn = egui::Button::new(
                    egui::RichText::new("\u{1F4BE}  Save Settings").color(white),
                )
                .fill(blue)
                .rounding(btn_rounding)
                .min_size(egui::vec2(140.0, 32.0));
                if ui.add(save_btn).clicked() {
                    match self.config.save(&self.project_path) {
                        Ok(_) => {
                            self.status_message = "Settings saved successfully".to_string();
                        }
                        Err(e) => {
                            self.show_error_with_message(&format!(
                                "Failed to save settings: {}",
                                e
                            ));
                        }
                    }
                }
            });
        });
        ui.add_space(12.0);

        egui::ScrollArea::vertical().show(ui, |ui| {
            // Basic Settings card
            Self::card_frame(ui).show(ui, |ui| {
                ui.label(egui::RichText::new("Basic Settings").strong().size(15.0));
                ui.add_space(8.0);

                egui::Grid::new("basic_settings_grid")
                    .num_columns(2)
                    .spacing([12.0, 8.0])
                    .show(ui, |ui| {
                        ui.label("Site Title:");
                        ui.add(
                            egui::TextEdit::singleline(&mut self.config.title)
                                .desired_width(300.0),
                        );
                        ui.end_row();

                        ui.label("Base URL:");
                        ui.add(
                            egui::TextEdit::singleline(&mut self.config.base_url)
                                .desired_width(300.0),
                        );
                        ui.end_row();

                        ui.label("Language:");
                        ui.add(
                            egui::TextEdit::singleline(&mut self.config.language_code)
                                .desired_width(300.0),
                        );
                        ui.end_row();
                    });
            });

            ui.add_space(12.0);

            // Sidebar card
            Self::card_frame(ui).show(ui, |ui| {
                ui.label(egui::RichText::new("Sidebar").strong().size(15.0));
                ui.add_space(8.0);

                egui::Grid::new("sidebar_settings_grid")
                    .num_columns(2)
                    .spacing([12.0, 8.0])
                    .show(ui, |ui| {
                        ui.label("Subtitle:");
                        ui.add(
                            egui::TextEdit::singleline(&mut self.config.sidebar_subtitle)
                                .desired_width(300.0),
                        );
                        ui.end_row();

                        ui.label("Emoji:");
                        ui.add(
                            egui::TextEdit::singleline(&mut self.config.sidebar_emoji)
                                .desired_width(300.0),
                        );
                        ui.end_row();
                    });
            });

            ui.add_space(12.0);

            // Article Settings card
            Self::card_frame(ui).show(ui, |ui| {
                ui.label(egui::RichText::new("Article Settings").strong().size(15.0));
                ui.add_space(8.0);

                ui.checkbox(&mut self.config.math_enabled, "Enable Math");
                ui.add_space(4.0);
                ui.checkbox(&mut self.config.toc_enabled, "Enable TOC");
                ui.add_space(4.0);
                ui.checkbox(&mut self.config.reading_time, "Show Reading Time");
            });
        });
    }

    fn start_hugo_server(&mut self) {
        let hugo_path = match self.find_hugo_exe() {
            Some(p) => p,
            None => {
                self.show_error_with_message(
                    "Hugo not found in system PATH. Please install Hugo first.",
                );
                return;
            }
        };
        let path = self.project_path.clone();

        std::thread::spawn(move || {
            let result = std::process::Command::new("cmd")
                .args(["/C", "start", "", &hugo_path, "server", "-D"])
                .current_dir(&path)
                .spawn();

            if let Err(e) = result {
                eprintln!("Failed to start hugo server: {}", e);
            } else {
                // 等待服务器启动后打开浏览器
                std::thread::sleep(std::time::Duration::from_secs(3));
                let _ = open::that("http://localhost:1313");
            }
        });

        self.status_message = "Hugo server starting in new window...".to_string();
    }

    fn find_hugo_exe(&self) -> Option<String> {
        // Only trust hugo from system PATH, never from project directory.
        // Must return an absolute path to prevent current_dir hijacking.
        if let Ok(output) = std::process::Command::new("where").arg("hugo").output() {
            if output.status.success() {
                let path_str = String::from_utf8_lossy(&output.stdout);
                for line in path_str.lines() {
                    let hugo_path = std::path::Path::new(line.trim());
                    if !hugo_path.is_absolute() {
                        continue;
                    }
                    // Use canonicalized Path comparison (case-insensitive on Windows)
                    let is_in_project = hugo_path
                        .canonicalize()
                        .ok()
                        .and_then(|hp| {
                            self.project_path.canonicalize().ok().map(|pp| hp.starts_with(pp))
                        })
                        .unwrap_or(false);
                    if !is_in_project {
                        return Some(line.trim().to_string());
                    }
                }
            }
        }

        // No safe hugo found — do NOT fall back to bare "hugo"
        // (Windows would resolve it via current_dir, enabling hijack)
        None
    }

    fn build_site(&mut self) {
        let hugo_path = match self.find_hugo_exe() {
            Some(p) => p,
            None => {
                self.show_error_with_message(
                    "Hugo not found in system PATH. Please install Hugo first.",
                );
                return;
            }
        };
        match std::process::Command::new(&hugo_path)
            .current_dir(&self.project_path)
            .output()
        {
            Ok(output) => {
                if output.status.success() {
                    self.status_message = "Site built successfully".to_string();
                } else {
                    let error = String::from_utf8_lossy(&output.stderr);
                    self.show_error_with_message(&format!("Build failed: {}", error));
                }
            }
            Err(e) => {
                self.show_error_with_message(&format!("Failed to run hugo: {}", e));
            }
        }
    }

    fn select_project_path(&mut self) {
        let folder = rfd::FileDialog::new()
            .set_title("Select Hugo Project Folder")
            .set_directory(&self.project_path)
            .pick_folder();

        if let Some(path) = folder {
            // 检查是否是有效的 Hugo 项目
            let is_valid = path.join("hugo.yaml").exists()
                || path.join("hugo.toml").exists()
                || path.join("hugo.json").exists()
                || path.join("config.yaml").exists()
                || path.join("config.toml").exists();

            if is_valid {
                self.project_path = path.clone();
                self.config = HugoConfig::load(&path).unwrap_or_default();
                self.article_manager = ArticleManager::new(&path);
                self.theme_manager = ThemeManager::new(&path);
                self.status_message = format!("Project loaded: {}", path.display());
            } else {
                // 即使没有配置文件也允许选择（可能是新建项目）
                self.project_path = path.clone();
                self.config = HugoConfig::default();
                self.article_manager = ArticleManager::new(&path);
                self.theme_manager = ThemeManager::new(&path);
                self.status_message =
                    format!("Folder selected (no Hugo config found): {}", path.display());
            }
        }
    }

    fn show_error_with_message(&mut self, message: &str) {
        self.error_message = message.to_string();
        self.show_error = true;
    }
}

impl eframe::App for HugoApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let tab_blue = egui::Color32::from_rgb(59, 130, 246);

        // Top panel with tabs
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                let tabs = [
                    (Tab::Dashboard, "\u{1F3E0} Dashboard"),
                    (Tab::Articles, "\u{1F4DD} Articles"),
                    (Tab::Themes, "\u{1F3A8} Themes"),
                    (Tab::Settings, "\u{2699} Settings"),
                ];
                for (tab, label) in tabs {
                    let is_active = self.current_tab == tab;
                    let btn = egui::Button::new(
                        egui::RichText::new(label).color(if is_active {
                            egui::Color32::WHITE
                        } else {
                            egui::Color32::BLACK
                        }),
                    )
                    .fill(if is_active {
                        tab_blue
                    } else {
                        egui::Color32::TRANSPARENT
                    })
                    .rounding(egui::Rounding::same(6.0));
                    if ui.add(btn).clicked() {
                        self.current_tab = tab;
                    }
                }
            });
            ui.add_space(4.0);
        });

        // Status bar
        let status_fill = ctx.style().visuals.window_fill();
        egui::TopBottomPanel::bottom("status_bar")
            .frame(
                egui::Frame::default()
                    .inner_margin(egui::Margin::symmetric(12.0, 6.0))
                    .fill(status_fill),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    if !self.status_message.is_empty() {
                        ui.colored_label(
                            egui::Color32::from_rgb(16, 185, 129),
                            "\u{2713}",
                        );
                        ui.label(&self.status_message);
                    }
                    ui.with_layout(
                        egui::Layout::right_to_left(egui::Align::Center),
                        |ui| {
                            ui.label(
                                egui::RichText::new(format!(
                                    "Project Path: {}",
                                    self.project_path.display()
                                ))
                                .small()
                                .color(ui.visuals().weak_text_color()),
                            );
                        },
                    );
                });
            });

        // Main content area
        let central_frame = if self.current_tab == Tab::Articles {
            egui::Frame::central_panel(&ctx.style()).inner_margin(egui::Margin::same(0.0))
        } else {
            egui::Frame::central_panel(&ctx.style())
        };
        egui::CentralPanel::default()
            .frame(central_frame)
            .show(ctx, |ui| match self.current_tab {
            Tab::Dashboard => self.show_dashboard(ui),
            Tab::Articles => self.show_articles(ui),
            Tab::Themes => self.show_themes(ui),
            Tab::Settings => self.show_settings(ui),
        });

        // Error dialog
        if self.show_error {
            egui::Window::new("Error")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.colored_label(egui::Color32::RED, &self.error_message);
                    if ui.button("OK").clicked() {
                        self.show_error = false;
                    }
                });
        }
    }
}
