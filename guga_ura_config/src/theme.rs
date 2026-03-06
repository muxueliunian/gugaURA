use eframe::egui;

/// 现代简约浅色主题
pub fn apply_theme(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::light();

    visuals.panel_fill = colors::BG_APP;
    visuals.window_fill = colors::BG_APP;
    visuals.extreme_bg_color = colors::SURFACE;
    visuals.faint_bg_color = colors::SURFACE_SUBTLE;
    visuals.code_bg_color = colors::SURFACE_SUBTLE;

    visuals.widgets.noninteractive.bg_fill = colors::SURFACE;
    visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, colors::TEXT_SECONDARY);
    visuals.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0, colors::BORDER);

    visuals.widgets.inactive.bg_fill = colors::BUTTON_SECONDARY_BG;
    visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, colors::TEXT_PRIMARY);
    visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, colors::BORDER);

    visuals.widgets.hovered.bg_fill = colors::BUTTON_SECONDARY_HOVER;
    visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.0, colors::TEXT_PRIMARY);
    visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.0, colors::BORDER_STRONG);

    visuals.widgets.active.bg_fill = colors::TAB_ACTIVE_BG;
    visuals.widgets.active.fg_stroke = egui::Stroke::new(1.0, colors::TEXT_PRIMARY);
    visuals.widgets.active.bg_stroke = egui::Stroke::new(1.0, colors::BORDER_STRONG);

    visuals.selection.bg_fill = colors::PRIMARY;
    visuals.selection.stroke = egui::Stroke::new(1.0, colors::PRIMARY_HOVER);

    let corner_radius = egui::CornerRadius::same(8);
    visuals.widgets.noninteractive.corner_radius = corner_radius;
    visuals.widgets.inactive.corner_radius = corner_radius;
    visuals.widgets.hovered.corner_radius = corner_radius;
    visuals.widgets.active.corner_radius = corner_radius;

    visuals.widgets.hovered.expansion = 0.0;
    visuals.widgets.active.expansion = 0.0;
    visuals.window_shadow = egui::Shadow::NONE;

    ctx.set_visuals(visuals);

    let mut style = (*ctx.style()).clone();
    style.spacing.item_spacing = egui::vec2(8.0, 8.0);
    style.spacing.window_margin = egui::Margin::same(16);
    style.spacing.button_padding = egui::vec2(12.0, 8.0);
    style.visuals.hyperlink_color = colors::PRIMARY;
    ctx.set_style(style);
}

/// 现代简约配色（无绿色）
pub mod colors {
    use eframe::egui::Color32;

    pub const BG_APP: Color32 = Color32::from_rgb(246, 247, 249); // #F6F7F9
    pub const SURFACE: Color32 = Color32::from_rgb(255, 255, 255); // #FFFFFF
    pub const SURFACE_SUBTLE: Color32 = Color32::from_rgb(250, 251, 252);
    pub const SURFACE_STRONG: Color32 = Color32::from_rgb(243, 244, 246);

    pub const BORDER: Color32 = Color32::from_rgb(229, 231, 235); // #E5E7EB
    pub const BORDER_STRONG: Color32 = Color32::from_rgb(203, 213, 225);

    pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(17, 24, 39); // #111827
    pub const TEXT_SECONDARY: Color32 = Color32::from_rgb(107, 114, 128); // #6B7280
    pub const TEXT_MUTED: Color32 = Color32::from_rgb(148, 163, 184);

    pub const PRIMARY: Color32 = Color32::from_rgb(37, 99, 235); // #2563EB
    pub const PRIMARY_HOVER: Color32 = Color32::from_rgb(29, 78, 216); // #1D4ED8
    pub const PRIMARY_ACTIVE: Color32 = Color32::from_rgb(30, 64, 175);
    pub const PRIMARY_TEXT: Color32 = Color32::from_rgb(255, 255, 255);

    pub const BUTTON_SECONDARY_BG: Color32 = SURFACE;
    pub const BUTTON_SECONDARY_HOVER: Color32 = Color32::from_rgb(249, 250, 251);
    pub const BUTTON_DANGER_BG: Color32 = Color32::from_rgb(254, 242, 242);
    pub const BUTTON_DANGER_HOVER: Color32 = Color32::from_rgb(254, 226, 226);
    pub const BUTTON_DANGER_TEXT: Color32 = Color32::from_rgb(185, 28, 28);

    pub const TAB_ACTIVE_BG: Color32 = Color32::from_rgb(239, 246, 255);
    pub const TAB_ACTIVE_BORDER: Color32 = Color32::from_rgb(191, 219, 254);
    pub const TAB_TEXT_ACTIVE: Color32 = PRIMARY;
    pub const TAB_TEXT_INACTIVE: Color32 = TEXT_SECONDARY;

    pub const INPUT_BG: Color32 = SURFACE;

    pub const INFO: Color32 = Color32::from_rgb(14, 165, 233); // #0EA5E9
    pub const WARNING: Color32 = Color32::from_rgb(217, 119, 6); // #D97706
    pub const ERROR: Color32 = Color32::from_rgb(220, 38, 38); // #DC2626
    pub const SUCCESS: Color32 = Color32::from_rgb(22, 163, 74);

    pub const INFO_BG: Color32 = Color32::from_rgb(240, 249, 255);
    pub const WARNING_BG: Color32 = Color32::from_rgb(255, 251, 235);
    pub const ERROR_BG: Color32 = Color32::from_rgb(254, 242, 242);
    pub const SUCCESS_BG: Color32 = Color32::from_rgb(240, 253, 244);

    pub const STATUS_BAR_BG: Color32 = Color32::from_rgb(248, 250, 252);
}
