//! GUI 应用主界面

use eframe::egui::{self, RichText, Color32, Layout, Align};
use std::path::PathBuf;

use crate::config::Config;
use crate::detector::{detect_game_version, is_valid_game_dir, scan_installed_games, DetectedGame, GameVersion};
use crate::installer::{check_install_status, install_dll, uninstall_dll, InstallStatus};
use crate::theme::colors;

/// 应用状态
pub struct ConfigApp {
    /// 游戏路径
    game_path: String,
    /// 检测到的游戏版本
    detected_version: GameVersion,
    /// 配置
    config: Config,
    /// 安装状态
    install_status: InstallStatus,
    /// 状态消息
    status_message: Option<(String, bool)>, // (message, is_error)
    /// 扫描到的游戏列表
    detected_games: Vec<DetectedGame>,
    /// 是否显示游戏选择弹窗
    show_game_selector: bool,
}

impl ConfigApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            game_path: String::new(),
            detected_version: GameVersion::Unknown,
            config: Config::default(),
            install_status: InstallStatus::Unknown,
            status_message: None,
            detected_games: Vec::new(),
            show_game_selector: false,
        }
    }
    
    /// 刷新状态
    fn refresh_status(&mut self) {
        if self.game_path.is_empty() {
            self.detected_version = GameVersion::Unknown;
            self.install_status = InstallStatus::Unknown;
            return;
        }
        
        let path = PathBuf::from(&self.game_path);
        if !is_valid_game_dir(&path) {
            self.detected_version = GameVersion::Unknown;
            self.install_status = InstallStatus::Unknown;
            return;
        }
        
        self.detected_version = detect_game_version(&path);
        self.install_status = check_install_status(&path, self.detected_version);
        self.config = Config::load_from(&path);
    }
    
    /// 选择游戏目录
    fn select_game_folder(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .set_title("选择游戏目录 (包含 umamusume.exe)")
            .pick_folder()
        {
            self.game_path = path.to_string_lossy().to_string();
            self.refresh_status();
            
            if self.detected_version == GameVersion::Unknown {
                self.status_message = Some(("未找到游戏文件，请确认选择正确的目录".to_string(), true));
            } else {
                self.status_message = Some((
                    format!("检测到 {}", self.detected_version.display_name()),
                    false
                ));
            }
        }
    }
    
    /// 自动扫描游戏
    fn scan_games(&mut self) {
        self.detected_games = scan_installed_games();
        
        if self.detected_games.is_empty() {
            self.status_message = Some(("未检测到已安装的游戏".to_string(), true));
            self.show_game_selector = false;
        } else if self.detected_games.len() == 1 {
            // 只有一个游戏，直接选择
            let game_path = self.detected_games[0].path.to_string_lossy().to_string();
            let game_version = self.detected_games[0].version;
            let game_display = self.detected_games[0].path.display().to_string();
            self.game_path = game_path;
            self.refresh_status();
            self.status_message = Some((
                format!("检测到 {} - {}", game_version.display_name(), game_display),
                false
            ));
            self.show_game_selector = false;
        } else {
            // 多个游戏，显示选择界面
            self.show_game_selector = true;
            self.status_message = Some((
                format!("检测到 {} 个游戏安装，请选择一个", self.detected_games.len()),
                false
            ));
        }
    }
    
    /// 选择检测到的游戏
    fn select_detected_game(&mut self, index: usize) {
        if index < self.detected_games.len() {
            let game_path = self.detected_games[index].path.to_string_lossy().to_string();
            let game_version = self.detected_games[index].version;
            self.game_path = game_path;
            self.refresh_status();
            self.status_message = Some((
                format!("已选择 {}", game_version.display_name()),
                false
            ));
            self.show_game_selector = false;
        }
    }
    
    /// 安装 DLL
    fn do_install(&mut self) {
        let path = PathBuf::from(&self.game_path);
        
        match install_dll(&path, self.detected_version) {
            Ok(()) => {
                self.status_message = Some(("安装成功！".to_string(), false));
                // 保存配置
                let _ = self.config.save_to(&path);
            }
            Err(e) => {
                self.status_message = Some((format!("安装失败: {}", e), true));
            }
        }
        
        self.refresh_status();
    }
    
    /// 卸载 DLL
    fn do_uninstall(&mut self) {
        let path = PathBuf::from(&self.game_path);
        
        match uninstall_dll(&path, self.detected_version) {
            Ok(()) => {
                self.status_message = Some(("卸载成功！".to_string(), false));
            }
            Err(e) => {
                self.status_message = Some((format!("卸载失败: {}", e), true));
            }
        }
        
        self.refresh_status();
    }
    
    /// 保存配置
    fn do_save_config(&mut self) {
        let path = PathBuf::from(&self.game_path);
        
        match self.config.save_to(&path) {
            Ok(()) => {
                self.status_message = Some(("配置已保存".to_string(), false));
            }
            Err(e) => {
                self.status_message = Some((format!("保存失败: {}", e), true));
            }
        }
    }
}

// 辅助函数：绘制卡片
fn ui_card<R>(ui: &mut egui::Ui, title: &str, add_contents: impl FnOnce(&mut egui::Ui) -> R) -> R {
    let frame = egui::Frame::default()
        .fill(colors::CARD_BG)
        .corner_radius(12)
        .inner_margin(16);
        
    frame.show(ui, |ui| {
        ui.label(
            RichText::new(title)
                .size(12.0)
                .color(colors::TEXT_SECONDARY)
        );
        ui.add_space(8.0);
        add_contents(ui)
    }).inner
}

// 辅助函数：主按钮
fn primary_button(ui: &mut egui::Ui, text: &str) -> egui::Response {
    let btn = egui::Button::new(
        RichText::new(text).color(colors::BACKGROUND).strong()
    )
    .fill(colors::ACCENT)
    .min_size(egui::vec2(80.0, 36.0));
    ui.add(btn)
}

// 辅助函数：次要按钮
fn secondary_button(ui: &mut egui::Ui, text: &str) -> egui::Response {
    let btn = egui::Button::new(
        RichText::new(text).color(colors::TEXT_SECONDARY)
    )
    .fill(egui::Color32::from_rgb(51, 65, 85))
    .min_size(egui::vec2(80.0, 36.0));
    ui.add(btn)
}

// 带宽度的Label辅助函数
fn label_sized(ui: &mut egui::Ui, text: &str, width: f32) {
    ui.add_sized(
        [width, 20.0], 
        egui::Label::new(RichText::new(text).color(colors::TEXT_SECONDARY))
    );
}

impl eframe::App for ConfigApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // 标题栏
            ui.horizontal(|ui| {
                ui.heading("GugaURA 配置工具");
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.label(RichText::new("v0.1.0").size(12.0).color(colors::TEXT_SECONDARY));
                });
            });
            ui.add_space(20.0);
            
            egui::ScrollArea::vertical().show(ui, |ui| {
                // 游戏路径卡片
                ui_card(ui, "游戏路径", |ui| {
                    ui.horizontal(|ui| {
                        // 路径输入框 - 使用 available_width 减去按钮宽度 (2*90 = 180 + spacing)
                        let btn_width = 190.0;
                        let path_width = ui.available_width() - btn_width;
                        
                        egui::ScrollArea::horizontal()
                            .id_salt("path_scroll")
                            .max_width(path_width)
                            .show(ui, |ui| {
                                ui.add(
                                    egui::TextEdit::singleline(&mut self.game_path)
                                        .hint_text("选择游戏目录...")
                                        .desired_width(f32::INFINITY)
                                        .text_color(colors::ACCENT)
                                );
                            });
                            
                        if secondary_button(ui, "选择").clicked() {
                            self.select_game_folder();
                        }
                        if secondary_button(ui, "🔍 检测").clicked() {
                            self.scan_games();
                        }
                    });
                    
                    // 游戏检测列表
                    if self.show_game_selector && !self.detected_games.is_empty() {
                        ui.add_space(8.0);
                        ui.label("检测到的游戏:");
                        egui::Frame::default()
                            .fill(colors::INPUT_BG)
                            .corner_radius(8)
                            .inner_margin(8)
                            .show(ui, |ui| {
                                egui::ScrollArea::vertical().max_height(80.0).show(ui, |ui| {
                                    let games = self.detected_games.clone();
                                    for (i, game) in games.iter().enumerate() {
                                        let text = format!("[{}] {}", 
                                            game.version.display_name(), 
                                            game.path.display()
                                        );
                                        if ui.selectable_label(false, &text).clicked() {
                                            self.select_detected_game(i);
                                        }
                                    }
                                });
                            });
                    }
                    
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("检测结果:").color(colors::TEXT_SECONDARY));
                        match self.detected_version {
                            GameVersion::Steam => {
                                ui.label(RichText::new("● Steam 版").color(colors::ACCENT));
                            },
                            GameVersion::DMM => {
                                ui.label(RichText::new("● DMM 版").color(Color32::from_rgb(200, 100, 0)));
                            },
                            GameVersion::Unknown => {
                                ui.label(RichText::new("○ 未检测").color(colors::TEXT_SECONDARY));
                            },
                        }
                    });
                });
                
                ui.add_space(16.0);
                
                // 服务器设置
                ui_card(ui, "服务器设置", |ui| {
                    ui.horizontal(|ui| {
                        label_sized(ui, "转发地址:", 60.0);
                        ui.add(
                            egui::TextEdit::singleline(&mut self.config.notifier_host)
                                .desired_width(ui.available_width())
                                .text_color(colors::ACCENT)
                        );
                    });
                    
                    ui.horizontal(|ui| {
                        label_sized(ui, "超时时间:", 60.0);
                        let mut timeout_str = self.config.timeout_ms.to_string();
                        if ui.add(
                            egui::TextEdit::singleline(&mut timeout_str)
                                .desired_width(80.0)
                        ).changed() {
                            if let Ok(v) = timeout_str.parse() {
                                self.config.timeout_ms = v;
                            }
                        }
                        ui.label(RichText::new("ms").color(colors::TEXT_SECONDARY));
                    });
                });
                
                ui.add_space(16.0);
                
                // 帧数设置
                ui_card(ui, "帧数设置", |ui| {
                    // 目标帧数
                    ui.horizontal(|ui| {
                        label_sized(ui, "目标帧数:", 60.0);
                        
                        let fps_options = [
                            (-1, "默认"),
                            (60, "60"),
                            (120, "120"),
                            (144, "144"),
                        ];
                        
                        for (fps, label) in fps_options {
                            let selected = self.config.target_fps == fps;
                            if ui.add(egui::Button::new(label).selected(selected).min_size(egui::vec2(40.0, 30.0))).clicked() {
                                self.config.target_fps = fps;
                            }
                        }
                    });
                    
                    ui.horizontal(|ui| {
                        ui.add_space(68.0); // spacer (60 + spacing)
                        // 自定义FPS - 只有当target_fps不是预设值时才显示
                        let preset_values = [-1, 60, 120, 144];
                        let is_custom = !preset_values.contains(&self.config.target_fps);
                        
                        let mut fps_str = if is_custom && self.config.target_fps > 0 {
                            self.config.target_fps.to_string()
                        } else {
                            String::new()
                        };
                        
                        ui.label(RichText::new("自定义:").size(12.0).color(colors::TEXT_SECONDARY));
                        if ui.add(egui::TextEdit::singleline(&mut fps_str).desired_width(60.0).hint_text("FPS")).changed() {
                            if let Ok(v) = fps_str.parse::<i32>() {
                                if v > 0 { self.config.target_fps = v; }
                            }
                            // 如果输入为空，不做任何事情，让用户手动点预设按钮
                        }
                    });
                    
                    ui.add_space(8.0);
                    
                    // 垂直同步
                    ui.horizontal(|ui| {
                        label_sized(ui, "垂直同步:", 60.0);
                        
                        let vsync_options = [
                            (-1, "默认"),
                            (0, "关闭"),
                            (1, "开启"),
                        ];
                        
                        for (vsync, label) in vsync_options {
                            let selected = self.config.vsync_count == vsync;
                            if ui.add(egui::Button::new(label).selected(selected).min_size(egui::vec2(40.0, 30.0))).clicked() {
                                self.config.vsync_count = vsync;
                            }
                        }
                    });
                });
                
                ui.add_space(16.0);
                
                // 底部状态和按钮
                ui.vertical_centered(|ui| {
                    // DLL 状态
                    egui::Frame::default()
                        .fill(colors::CARD_BG)
                        .corner_radius(12)
                        .inner_margin(12)
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("DLL 状态:").color(colors::TEXT_SECONDARY));
                                // Spacer
                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    let (text, bg_color, fg_color) = match self.install_status {
                                        InstallStatus::Installed => ("✓ 已安装", colors::SUCCESS_BG, colors::SUCCESS),
                                        InstallStatus::NotInstalled => ("未安装", Color32::from_rgb(70, 20, 20), colors::ERROR),
                                        InstallStatus::NeedsUpdate => ("需更新", Color32::from_rgb(80, 50, 0), Color32::YELLOW),
                                        InstallStatus::Unknown => ("? 未知", colors::INPUT_BG, colors::TEXT_SECONDARY),
                                    };
                                    
                                    ui.label(
                                        RichText::new(text)
                                            .color(fg_color)
                                            .background_color(bg_color)
                                            .strong()
                                    );
                                });
                            });
                        });
                    
                    ui.add_space(20.0);
                    
                    // 操作按钮组 - 使用 Horizontal layout 但允许换行
                    ui.horizontal_wrapped(|ui| {
                        ui.spacing_mut().item_spacing.x = 12.0;
                        ui.spacing_mut().button_padding = egui::vec2(16.0, 10.0);
                        
                        let has_game = self.detected_version != GameVersion::Unknown;
                        
                        ui.add_enabled_ui(has_game && self.install_status != InstallStatus::Installed, |ui| {
                            if secondary_button(ui, "安装 DLL").clicked() {
                                self.do_install();
                            }
                        });
                        
                        ui.add_enabled_ui(has_game && self.install_status == InstallStatus::Installed, |ui| {
                            if secondary_button(ui, "卸载 DLL").clicked() {
                                self.do_uninstall();
                            }
                        });
                        
                        ui.add_enabled_ui(has_game, |ui| {
                            if primary_button(ui, "保存配置").clicked() {
                                self.do_save_config();
                            }
                        });
                        
                        if secondary_button(ui, "刷新").clicked() {
                            self.refresh_status();
                            self.status_message = Some(("状态已刷新".to_string(), false));
                        }
                    });
                    
                    ui.add_space(12.0);
                    
                    // 状态消息
                    if let Some((msg, is_error)) = &self.status_message {
                        let color = if *is_error { colors::ERROR } else { colors::SUCCESS };
                        ui.label(RichText::new(msg).color(color));
                    }
                    
                    // 底部提示
                    ui.label(RichText::new("提示: 点击「检测」自动扫描已安装的游戏").size(11.0).color(colors::TEXT_SECONDARY));
                });
            });
        });
    }
}
