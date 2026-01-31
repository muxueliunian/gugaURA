//! GugaURA 配置工具
//! 
//! 用于配置和管理 GugaURA DLL 注入

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod detector;
mod installer;
mod config;
mod theme;
mod embedded_dlls;

use eframe::egui;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([500.0, 650.0])
            .with_min_inner_size([400.0, 500.0])
            .with_resizable(true),
        ..Default::default()
    };

    eframe::run_native(
        "GugaURA 配置工具",
        options,
        Box::new(|cc| {
            // 加载中文字体
            setup_custom_fonts(&cc.egui_ctx);
            // 应用自定义主题
            theme::apply_theme(&cc.egui_ctx);
            
            Ok(Box::new(app::ConfigApp::new(cc)))
        }),
    )
}

/// 设置自定义字体以支持中文
fn setup_custom_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    
    // 尝试加载系统中文字体
    // Windows: 微软雅黑 (msyh.ttc)
    let font_paths = [
        "C:\\Windows\\Fonts\\msyh.ttc",      // 微软雅黑
        "C:\\Windows\\Fonts\\simsun.ttc",    // 宋体
        "C:\\Windows\\Fonts\\simhei.ttf",    // 黑体
    ];
    
    let mut font_loaded = false;
    for path in font_paths {
        if let Ok(font_data) = std::fs::read(path) {
            fonts.font_data.insert(
                "chinese_font".to_owned(),
                egui::FontData::from_owned(font_data).into(),
            );
            
            // 将中文字体添加到所有字体族的首选列表
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
            
            font_loaded = true;
            break;
        }
    }
    
    if font_loaded {
        ctx.set_fonts(fonts);
    }
}
