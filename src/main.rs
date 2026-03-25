#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod models;
mod utils;

use eframe::egui;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_min_inner_size([800.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Hugo Blog Manager",
        options,
        Box::new(|cc| {
            // 配置中文字体支持
            configure_fonts(&cc.egui_ctx);
            Ok(Box::new(app::HugoApp::new(cc)))
        }),
    )
}

fn configure_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    
    // 尝试加载 Windows 系统中文字体
    let font_paths = [
        r"C:\Windows\Fonts\msyh.ttc",      // 微软雅黑
        r"C:\Windows\Fonts\msyh.ttf",      // 微软雅黑
        r"C:\Windows\Fonts\simsun.ttc",    // 宋体
        r"C:\Windows\Fonts\simhei.ttf",    // 黑体
    ];
    
    for font_path in &font_paths {
        if std::path::Path::new(font_path).exists() {
            if let Ok(font_data) = std::fs::read(font_path) {
                fonts.font_data.insert(
                    "chinese_font".to_owned(),
                    egui::FontData::from_owned(font_data),
                );
                
                // 将中文字体添加到字体族的开头，作为首选
                fonts
                    .families
                    .entry(egui::FontFamily::Proportional)
                    .or_default()
                    .insert(0, "chinese_font".to_owned());
                
                fonts
                    .families
                    .entry(egui::FontFamily::Monospace)
                    .or_default()
                    .insert(0, "chinese_font".to_owned());
                
                break;
            }
        }
    }
    
    ctx.set_fonts(fonts);
}
