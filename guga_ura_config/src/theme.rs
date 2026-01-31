use eframe::egui;

/// 配置应用的自定义主题
pub fn apply_theme(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();
    
    // 背景色 - 午夜黑
    visuals.panel_fill = egui::Color32::from_rgb(10, 15, 28);
    visuals.window_fill = egui::Color32::from_rgb(10, 15, 28);
    
    // 组件背景 - 深石板蓝
    visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(30, 41, 59);
    visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(51, 65, 85);
    visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(71, 85, 105);
    visuals.widgets.active.bg_fill = egui::Color32::from_rgb(15, 23, 42);
    
    // 选中状态 - 亮青色
    visuals.selection.bg_fill = egui::Color32::from_rgb(34, 211, 238);
    visuals.selection.stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(34, 211, 238));
    
    // 文字颜色 - 浅灰/白
    visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(203, 213, 225));
    visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(148, 163, 184));
    visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.0, egui::Color32::WHITE);
    visuals.widgets.active.fg_stroke = egui::Stroke::new(1.0, egui::Color32::WHITE);
    
    // 边框圆角 - eframe 0.31 使用 u8 作为半径
    // visuals.window_corner_radius = egui::CornerRadius::same(12); // 如果支持
    
    ctx.set_visuals(visuals);
    
    // 样式调整
    let mut style = (*ctx.style()).clone();
    
    // 间距和大小
    style.spacing.item_spacing = egui::vec2(10.0, 10.0);
    // Margin 使用 i8
    style.spacing.window_margin = egui::Margin::same(24);
    style.spacing.button_padding = egui::vec2(12.0, 8.0);
    
    ctx.set_style(style);
}

// 颜色定义常量
pub mod colors {
    use eframe::egui::Color32;
    
    pub const BACKGROUND: Color32 = Color32::from_rgb(10, 15, 28);      // #0A0F1C
    pub const CARD_BG: Color32 = Color32::from_rgb(30, 41, 59);         // #1E293B
    pub const ACCENT: Color32 = Color32::from_rgb(34, 211, 238);        // #22D3EE
    pub const ACCENT_HOVER: Color32 = Color32::from_rgb(6, 182, 212);   // #06B6D4
    pub const TEXT_PRIMARY: Color32 = Color32::WHITE;                   // #FFFFFF
    pub const TEXT_SECONDARY: Color32 = Color32::from_rgb(148, 163, 184); // #94A3B8
    pub const SUCCESS: Color32 = Color32::from_rgb(52, 211, 153);       // #34D399
    pub const SUCCESS_BG: Color32 = Color32::from_rgb(6, 78, 59);       // #064E3B
    pub const ERROR: Color32 = Color32::from_rgb(248, 113, 113);        // #F87171
    pub const INPUT_BG: Color32 = Color32::from_rgb(15, 23, 42);        // #0F172A
}
