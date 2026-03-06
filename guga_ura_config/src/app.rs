//! GUI 应用主界面 — 现代简约重构版

use eframe::egui::{self, Align, Color32, Layout, RichText};
use std::path::PathBuf;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use crate::config::Config;
use crate::detector::{
    detect_game_version, is_valid_game_dir, scan_installed_games, DetectedGame, GameVersion,
};
use crate::installer::{check_install_status, install_dll, uninstall_dll, InstallStatus};
use crate::theme::colors;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MainTab {
    Overview,
    GameDeploy,
    ForwardPerf,
    Debug,
    Console,
}

impl MainTab {
    const ALL: [MainTab; 5] = [
        MainTab::Overview,
        MainTab::GameDeploy,
        MainTab::ForwardPerf,
        MainTab::Debug,
        MainTab::Console,
    ];

    fn label(self) -> &'static str {
        match self {
            MainTab::Overview => "总览",
            MainTab::GameDeploy => "游戏与部署",
            MainTab::ForwardPerf => "转发与性能",
            MainTab::Debug => "调试",
            MainTab::Console => "控制台",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StatusLevel {
    Info,
    Success,
    Warning,
    Error,
}

impl StatusLevel {
    fn color(self) -> Color32 {
        match self {
            StatusLevel::Info => colors::INFO,
            StatusLevel::Success => colors::SUCCESS,
            StatusLevel::Warning => colors::WARNING,
            StatusLevel::Error => colors::ERROR,
        }
    }

    fn background(self) -> Color32 {
        match self {
            StatusLevel::Info => colors::INFO_BG,
            StatusLevel::Success => colors::SUCCESS_BG,
            StatusLevel::Warning => colors::WARNING_BG,
            StatusLevel::Error => colors::ERROR_BG,
        }
    }

    fn label(self) -> &'static str {
        match self {
            StatusLevel::Info => "信息",
            StatusLevel::Success => "成功",
            StatusLevel::Warning => "提示",
            StatusLevel::Error => "错误",
        }
    }
}

#[derive(Debug, Clone, Default)]
struct UiValidationState {
    game_path_valid: bool,
    notifier_host_valid: bool,
    timeout_valid: bool,
    custom_fps_valid: bool,
    install_disabled_reason: Option<String>,
    uninstall_disabled_reason: Option<String>,
    save_disabled_reason: Option<String>,
}

#[derive(Debug, Clone)]
struct UiLogEntry {
    at: SystemTime,
    level: StatusLevel,
    message: String,
}

impl UiLogEntry {
    fn new(level: StatusLevel, message: String) -> Self {
        Self {
            at: SystemTime::now(),
            level,
            message,
        }
    }

    fn time_hms(&self) -> String {
        use chrono::{Local, TimeZone};
        let ms = self
            .at
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();
        match Local.timestamp_millis_opt(ms as i64).single() {
            Some(dt) => dt.format("%H:%M:%S").to_string(),
            None => "??:??:??".to_string(),
        }
    }
}

/// 应用状态
pub struct ConfigApp {
    game_path: String,
    detected_version: GameVersion,
    config: Config,
    install_status: InstallStatus,
    receiver_status: String,
    status_message: Option<(String, StatusLevel)>,
    detected_games: Vec<DetectedGame>,
    show_game_selector: bool,
    status_time: Option<Instant>,
    active_tab: MainTab,
    validation: UiValidationState,
    ui_logs: Vec<UiLogEntry>,
    timeout_input: String,
    custom_fps_input: String,
    fans_output_dir_input: String,
}

impl ConfigApp {
    pub fn new(_cc: &eframe::CreationContext<'_>, receiver_status: String) -> Self {
        let config = Config::load_from_exe_dir();
        let mut app = Self {
            game_path: String::new(),
            detected_version: GameVersion::Unknown,
            config,
            install_status: InstallStatus::Unknown,
            receiver_status,
            status_message: None,
            detected_games: Vec::new(),
            show_game_selector: false,
            status_time: None,
            active_tab: MainTab::Overview,
            validation: UiValidationState::default(),
            ui_logs: Vec::new(),
            timeout_input: String::new(),
            custom_fps_input: String::new(),
            fans_output_dir_input: String::new(),
        };
        app.sync_inputs_from_config();
        app.validate_inputs();
        app
    }

    fn set_status(&mut self, msg: impl Into<String>, level: StatusLevel) {
        let message = msg.into();
        self.status_message = Some((message.clone(), level));
        self.status_time = Some(Instant::now());
        self.ui_logs.push(UiLogEntry::new(level, message));
        if self.ui_logs.len() > 40 {
            let drain_count = self.ui_logs.len() - 40;
            self.ui_logs.drain(0..drain_count);
        }
    }

    fn sync_inputs_from_config(&mut self) {
        self.timeout_input = self.config.timeout_ms.to_string();
        if is_custom_fps(self.config.target_fps) {
            self.custom_fps_input = self.config.target_fps.to_string();
        } else {
            self.custom_fps_input.clear();
        }
        self.fans_output_dir_input = self.current_fans_output_dir();
    }

    fn has_valid_game_context(&self) -> bool {
        self.validation.game_path_valid && self.detected_version != GameVersion::Unknown
    }

    fn validate_inputs(&mut self) {
        self.validation.game_path_valid = if self.game_path.trim().is_empty() {
            false
        } else {
            is_valid_game_dir(&PathBuf::from(&self.game_path))
        };

        let notifier = self.config.notifier_host.trim();
        self.validation.notifier_host_valid =
            notifier.starts_with("http://") || notifier.starts_with("https://");

        self.validation.timeout_valid = self
            .timeout_input
            .trim()
            .parse::<u64>()
            .map(|v| v > 0)
            .unwrap_or(false);

        self.validation.custom_fps_valid =
            if is_custom_fps(self.config.target_fps) || !self.custom_fps_input.trim().is_empty() {
                self.custom_fps_input
                    .trim()
                    .parse::<i32>()
                    .map(|v| v > 0)
                    .unwrap_or(false)
            } else {
                true
            };

        let has_game = self.has_valid_game_context();

        self.validation.install_disabled_reason = if !has_game {
            Some("请先选择并检测有效游戏目录".to_string())
        } else if self.install_status == InstallStatus::Installed {
            Some("当前已安装，如需重装请先卸载".to_string())
        } else {
            None
        };

        self.validation.uninstall_disabled_reason = if !has_game {
            Some("请先选择并检测有效游戏目录".to_string())
        } else if self.install_status != InstallStatus::Installed {
            Some("当前未安装，无需卸载".to_string())
        } else {
            None
        };

        self.validation.save_disabled_reason = if !has_game {
            Some("请先选择并检测有效游戏目录".to_string())
        } else if !self.validation.notifier_host_valid {
            Some("转发地址必须以 http:// 或 https:// 开头".to_string())
        } else if !self.validation.timeout_valid {
            Some("超时必须是大于 0 的整数".to_string())
        } else if !self.validation.custom_fps_valid {
            Some("自定义 FPS 必须是正整数".to_string())
        } else {
            None
        };
    }

    fn commit_inputs_to_config(&mut self) -> Result<(), String> {
        let timeout = self
            .timeout_input
            .trim()
            .parse::<u64>()
            .map_err(|_| "超时必须是整数".to_string())?;
        if timeout == 0 {
            return Err("超时必须大于 0".to_string());
        }
        self.config.timeout_ms = timeout;

        if is_custom_fps(self.config.target_fps) || !self.custom_fps_input.trim().is_empty() {
            let fps = self
                .custom_fps_input
                .trim()
                .parse::<i32>()
                .map_err(|_| "自定义 FPS 必须是正整数".to_string())?;
            if fps <= 0 {
                return Err("自定义 FPS 必须是正整数".to_string());
            }
            self.config.target_fps = fps;
        }

        let fans_output_dir = self.fans_output_dir_input.trim();
        self.config.fans_output_dir = if fans_output_dir.is_empty() {
            None
        } else {
            Some(fans_output_dir.to_string())
        };

        Ok(())
    }

    fn refresh_status(&mut self) {
        if self.game_path.trim().is_empty() {
            self.detected_version = GameVersion::Unknown;
            self.install_status = InstallStatus::Unknown;
            self.validate_inputs();
            return;
        }

        let path = PathBuf::from(&self.game_path);
        if !is_valid_game_dir(&path) {
            self.detected_version = GameVersion::Unknown;
            self.install_status = InstallStatus::Unknown;
            self.validate_inputs();
            return;
        }

        self.detected_version = detect_game_version(&path);
        self.install_status = check_install_status(&path, self.detected_version);
        let game_config_exists = Config::config_path(&path).exists();
        let game_config_has_fans_enabled = Config::game_config_has_key(&path, "fans_enabled");
        self.config = Config::load_from(&path);
        let exe_config = Config::load_from_exe_dir();
        if !game_config_exists || !game_config_has_fans_enabled {
            self.config.fans_enabled = exe_config.fans_enabled;
        }
        if self
            .config
            .fans_output_dir
            .as_ref()
            .map(|v| v.trim().is_empty())
            .unwrap_or(true)
        {
            self.config.fans_output_dir = exe_config.fans_output_dir;
        }
        self.sync_inputs_from_config();
        self.validate_inputs();
    }

    fn select_game_folder(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .set_title("选择游戏目录 (包含 umamusume.exe)")
            .pick_folder()
        {
            self.game_path = path.to_string_lossy().to_string();
            self.refresh_status();
            if self.detected_version == GameVersion::Unknown {
                self.set_status("未找到游戏文件，请确认目录", StatusLevel::Error);
            } else {
                self.set_status(
                    format!("检测到 {}", self.detected_version.display_name()),
                    StatusLevel::Success,
                );
            }
        }
    }

    fn scan_games(&mut self) {
        self.detected_games = scan_installed_games();
        if self.detected_games.is_empty() {
            self.set_status("未检测到已安装的游戏", StatusLevel::Warning);
            self.show_game_selector = false;
        } else if self.detected_games.len() == 1 {
            let game = self.detected_games[0].clone();
            self.game_path = game.path.to_string_lossy().to_string();
            self.refresh_status();
            self.set_status(
                format!("{} — {}", game.version.display_name(), game.path.display()),
                StatusLevel::Success,
            );
            self.show_game_selector = false;
        } else {
            self.show_game_selector = true;
            self.set_status(
                format!("检测到 {} 个游戏，请选择", self.detected_games.len()),
                StatusLevel::Info,
            );
        }
    }

    fn select_detected_game(&mut self, index: usize) {
        if let Some(game) = self.detected_games.get(index).cloned() {
            let selected_path = game.path.to_string_lossy().to_string();
            let selected_version = game.version;
            self.game_path = selected_path;
            self.refresh_status();
            self.set_status(
                format!("已选择 {}", selected_version.display_name()),
                StatusLevel::Success,
            );
            self.show_game_selector = false;
        }
    }

    fn do_install(&mut self) {
        let path = PathBuf::from(&self.game_path);
        match install_dll(&path, self.detected_version) {
            Ok(()) => {
                self.apply_debug_output_dir();
                self.ensure_fans_output_dir();
                match self.save_config_to_targets(&path) {
                    Ok(()) => self.set_status("安装成功", StatusLevel::Success),
                    Err(e) => self.set_status(
                        format!("安装成功，但配置同步失败: {}", e),
                        StatusLevel::Warning,
                    ),
                }
            }
            Err(e) => self.set_status(format!("安装失败: {}", e), StatusLevel::Error),
        }
        self.refresh_status();
    }

    fn do_uninstall(&mut self) {
        let path = PathBuf::from(&self.game_path);
        match uninstall_dll(&path, self.detected_version) {
            Ok(()) => self.set_status("已卸载", StatusLevel::Success),
            Err(e) => self.set_status(format!("卸载失败: {}", e), StatusLevel::Error),
        }
        self.refresh_status();
    }

    fn do_save_config(&mut self) {
        if let Err(e) = self.commit_inputs_to_config() {
            self.set_status(format!("保存失败: {}", e), StatusLevel::Error);
            self.validate_inputs();
            return;
        }

        self.apply_debug_output_dir();
        self.ensure_fans_output_dir();
        let path = PathBuf::from(&self.game_path);
        match self.save_config_to_targets(&path) {
            Ok(()) => self.set_status("配置已保存", StatusLevel::Success),
            Err(e) => self.set_status(format!("保存失败: {}", e), StatusLevel::Error),
        }
        self.validate_inputs();
    }

    fn save_config_to_targets(&self, game_dir: &std::path::Path) -> Result<(), String> {
        self.config.save_to(game_dir)?;
        self.config.save_to_exe_dir()?;
        Ok(())
    }

    fn apply_debug_output_dir(&mut self) {
        if let Ok(mut exe_dir) = std::env::current_exe() {
            exe_dir.pop();
            let debug_dir = exe_dir.join("debug");
            self.config.debug_output_dir = Some(debug_dir.to_string_lossy().to_string());
        }
    }

    fn ensure_fans_output_dir(&mut self) {
        let value = self.fans_output_dir_input.trim();
        if value.is_empty() {
            self.config.fans_output_dir =
                Some(self.default_fans_output_dir().display().to_string());
            self.fans_output_dir_input = self.default_fans_output_dir().display().to_string();
        } else {
            self.config.fans_output_dir = Some(value.to_string());
        }
    }

    fn current_debug_output_dir(&self) -> String {
        if let Some(dir) = &self.config.debug_output_dir {
            return dir.clone();
        }
        if let Ok(mut exe_dir) = std::env::current_exe() {
            exe_dir.pop();
            return exe_dir.join("debug").display().to_string();
        }
        "debug".to_string()
    }

    fn default_fans_output_dir(&self) -> PathBuf {
        if let Ok(mut exe_dir) = std::env::current_exe() {
            exe_dir.pop();
            return exe_dir.join("fans");
        }
        PathBuf::from("fans")
    }

    fn current_fans_output_dir(&self) -> String {
        if let Some(dir) = &self.config.fans_output_dir {
            if !dir.trim().is_empty() {
                return dir.clone();
            }
        }
        self.default_fans_output_dir().display().to_string()
    }

    fn install_status_summary(&self) -> (&'static str, StatusLevel) {
        match self.install_status {
            InstallStatus::Installed => ("已安装 · 版本一致", StatusLevel::Success),
            InstallStatus::NotInstalled => ("未安装", StatusLevel::Warning),
            InstallStatus::NeedsUpdate => ("需要更新", StatusLevel::Warning),
            InstallStatus::Unknown => ("等待检测", StatusLevel::Info),
        }
    }

    fn render_header(&self, ui: &mut egui::Ui) {
        surface_card(ui, |ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label(
                        RichText::new("GugaURA Control Center")
                            .size(20.0)
                            .strong()
                            .color(colors::TEXT_PRIMARY),
                    );
                    ui.label(
                        RichText::new("现代简约配置界面 · 保持完整功能兼容")
                            .size(12.0)
                            .color(colors::TEXT_SECONDARY),
                    );
                });

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let (status_text, level) = self.install_status_summary();
                    status_badge(ui, level, status_text);
                    ui.add_space(8.0);
                    ui.label(
                        RichText::new(format!("v{}", env!("CARGO_PKG_VERSION")))
                            .size(12.0)
                            .color(colors::TEXT_SECONDARY),
                    );
                });
            });
        });
    }

    fn render_global_action_bar(&mut self, ui: &mut egui::Ui) {
        surface_card(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label(
                    RichText::new("全局操作")
                        .size(14.0)
                        .strong()
                        .color(colors::TEXT_PRIMARY),
                );
                ui.add_space(8.0);

                let save_enabled = self.validation.save_disabled_reason.is_none();
                let save_resp = ui.add_enabled(save_enabled, primary_button("保存配置"));
                if save_resp.clicked() {
                    self.do_save_config();
                }
                if !save_enabled {
                    if let Some(reason) = &self.validation.save_disabled_reason {
                        save_resp.on_hover_text(reason);
                    }
                }

                let refresh_resp = ui.add(secondary_button("刷新状态"));
                if refresh_resp.clicked() {
                    self.refresh_status();
                    self.set_status("已刷新", StatusLevel::Info);
                }
            });
        });
    }

    fn render_tab_bar(&mut self, ui: &mut egui::Ui) {
        surface_card(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                for tab in MainTab::ALL {
                    let selected = self.active_tab == tab;
                    let resp = ui.add(tab_button(tab.label(), selected));
                    if resp.clicked() {
                        self.active_tab = tab;
                    }
                }
            });
        });
    }

    fn render_overview_tab(&mut self, ui: &mut egui::Ui) {
        section_card(
            ui,
            "系统状态总览",
            "接收器、版本与部署状态",
            |ui| {
                let (install_text, install_level) = self.install_status_summary();
                kv_row(
                    ui,
                    "游戏版本",
                    match self.detected_version {
                        GameVersion::Steam => "Steam 版",
                        GameVersion::DMM => "DMM 版",
                        GameVersion::Unknown => "未检测",
                    },
                    colors::TEXT_PRIMARY,
                );
                kv_row(ui, "DLL 状态", install_text, install_level.color());
                kv_row(
                    ui,
                    "本地接收器",
                    &self.receiver_status,
                    colors::TEXT_PRIMARY,
                );
                kv_row(ui, "配置文件", "guga_ura_config.json", colors::TEXT_PRIMARY);
                kv_row(
                    ui,
                    "Fans 保存",
                    if self.config.fans_enabled {
                        "开启"
                    } else {
                        "关闭"
                    },
                    if self.config.fans_enabled {
                        colors::SUCCESS
                    } else {
                        colors::WARNING
                    },
                );
                kv_row(
                    ui,
                    "Fans 输出目录",
                    &self.current_fans_output_dir(),
                    colors::TEXT_SECONDARY,
                );
                if !self.game_path.trim().is_empty() {
                    kv_row(ui, "当前目录", &self.game_path, colors::TEXT_SECONDARY);
                }
            },
        );

        ui.add_space(10.0);

        section_card(
            ui,
            "推荐操作流程",
            "首次使用建议按顺序执行",
            |ui| {
                for step in [
                    "1. 进入“游戏与部署”选择或自动检测目录",
                    "2. 确认版本与 DLL 部署状态",
                    "3. 在“转发与性能”设置地址、超时、FPS、VSync",
                    "4. 点击“保存配置”，必要时执行安装/卸载",
                ] {
                    ui.label(RichText::new(step).size(12.0).color(colors::TEXT_SECONDARY));
                }
            },
        );

        ui.add_space(10.0);

        section_card(
            ui,
            "最近状态消息",
            "保留最近 40 条会话日志",
            |ui| {
                if self.ui_logs.is_empty() {
                    ui.label(
                        RichText::new("暂无日志")
                            .size(12.0)
                            .color(colors::TEXT_MUTED),
                    );
                } else {
                    egui::ScrollArea::vertical()
                        .max_height(180.0)
                        .show(ui, |ui| {
                            for (idx, item) in self.ui_logs.iter().rev().take(10).enumerate() {
                                ui.horizontal_wrapped(|ui| {
                                    ui.label(
                                        RichText::new(item.time_hms())
                                            .size(11.0)
                                            .color(colors::TEXT_MUTED),
                                    );
                                    ui.label(
                                        RichText::new(item.level.label())
                                            .size(11.0)
                                            .color(item.level.color())
                                            .strong(),
                                    );
                                });
                                ui.add(
                                    egui::Label::new(
                                        RichText::new(&item.message)
                                            .size(12.0)
                                            .color(colors::TEXT_SECONDARY),
                                    )
                                    .wrap(),
                                );
                                if idx < 9 {
                                    ui.add_space(4.0);
                                    ui.separator();
                                    ui.add_space(4.0);
                                }
                            }
                        });
                }
            },
        );
    }

    fn render_game_deploy_tab(&mut self, ui: &mut egui::Ui) {
        section_card(
            ui,
            "游戏目录检测",
            "支持自动扫描和手动选择，保留 0/1/N 检测分支",
            |ui| {
                field_label(ui, "游戏目录");
                input_frame(ui, |ui| {
                    let input_width = (ui.available_width() - 110.0).max(120.0);
                    ui.horizontal(|ui| {
                        ui.add_sized(
                            egui::vec2(input_width, 36.0),
                            egui::TextEdit::singleline(&mut self.game_path)
                                .hint_text("选择游戏目录...")
                                .text_color(colors::TEXT_PRIMARY),
                        );
                        if ui.add(secondary_button("浏览目录")).clicked() {
                            self.select_game_folder();
                        }
                    });
                });

                ui.horizontal_wrapped(|ui| {
                    if ui.add(secondary_button("自动检测")).clicked() {
                        self.scan_games();
                    }
                    if ui.add(secondary_button("手动选择")).clicked() {
                        self.select_game_folder();
                    }
                    if ui.add(secondary_button("刷新检测")).clicked() {
                        self.refresh_status();
                        self.set_status("路径状态已刷新", StatusLevel::Info);
                    }
                });

                if self.show_game_selector && !self.detected_games.is_empty() {
                    ui.add_space(6.0);
                    ui.label(
                        RichText::new("检测到多个安装目录，请选择：")
                            .size(12.0)
                            .color(colors::TEXT_SECONDARY),
                    );
                    input_frame(ui, |ui| {
                        egui::ScrollArea::vertical()
                            .max_height(96.0)
                            .show(ui, |ui| {
                                let games = self.detected_games.clone();
                                for (idx, game) in games.iter().enumerate() {
                                    let text = format!(
                                        "[{}] {}",
                                        game.version.display_name(),
                                        game.path.display()
                                    );
                                    if ui.selectable_label(false, text).clicked() {
                                        self.select_detected_game(idx);
                                    }
                                }
                            });
                    });
                }

                ui.add_space(4.0);
                let (hint, level) = match self.detected_version {
                    GameVersion::Steam => {
                        ("检测结果：Steam 版已识别，路径有效", StatusLevel::Success)
                    }
                    GameVersion::DMM => ("检测结果：DMM 版已识别，路径有效", StatusLevel::Success),
                    GameVersion::Unknown => ("检测结果：等待检测或路径无效", StatusLevel::Warning),
                };
                ui.label(RichText::new(hint).size(12.0).color(level.color()));
            },
        );

        ui.add_space(10.0);

        section_card(
            ui,
            "部署操作",
            "安装/卸载逻辑与原实现一致，入口更清晰",
            |ui| {
                let install_enabled = self.validation.install_disabled_reason.is_none();
                let install_resp = ui.add_enabled(install_enabled, secondary_button("安装 DLL"));
                if install_resp.clicked() {
                    self.do_install();
                }
                if !install_enabled {
                    if let Some(reason) = &self.validation.install_disabled_reason {
                        install_resp.on_hover_text(reason);
                    }
                }

                let uninstall_enabled = self.validation.uninstall_disabled_reason.is_none();
                let uninstall_resp = ui.add_enabled(uninstall_enabled, danger_button("卸载"));
                if uninstall_resp.clicked() {
                    self.do_uninstall();
                }
                if !uninstall_enabled {
                    if let Some(reason) = &self.validation.uninstall_disabled_reason {
                        uninstall_resp.on_hover_text(reason);
                    }
                }

                ui.add_space(8.0);
                ui.label(
                    RichText::new("说明：卸载会恢复原始 DLL 并清理配置/数据目录。")
                        .size(12.0)
                        .color(colors::TEXT_SECONDARY),
                );
            },
        );
    }

    fn render_forward_perf_tab(&mut self, ui: &mut egui::Ui) {
        section_card(
            ui,
            "转发服务设置",
            "保留 notifier_host 与 timeout_ms",
            |ui| {
                field_label(ui, "地址");
                input_frame(ui, |ui| {
                    ui.add_sized(
                        egui::vec2(ui.available_width(), 36.0),
                        egui::TextEdit::singleline(&mut self.config.notifier_host)
                            .hint_text("http://127.0.0.1:4693")
                            .text_color(colors::TEXT_PRIMARY),
                    );
                });
                if !self.validation.notifier_host_valid {
                    ui.label(
                        RichText::new("地址必须以 http:// 或 https:// 开头")
                            .size(12.0)
                            .color(colors::ERROR),
                    );
                }

                ui.add_space(8.0);
                field_label(ui, "超时 (ms)");
                input_frame(ui, |ui| {
                    ui.add_sized(
                        egui::vec2(120.0, 36.0),
                        egui::TextEdit::singleline(&mut self.timeout_input)
                            .hint_text("100")
                            .text_color(colors::TEXT_PRIMARY),
                    );
                });
                if !self.validation.timeout_valid {
                    ui.label(
                        RichText::new("超时必须是大于 0 的整数")
                            .size(12.0)
                            .color(colors::ERROR),
                    );
                }

                ui.add_space(8.0);
                field_label(ui, "Fans 数据保存");
                input_frame(ui, |ui| {
                    ui.checkbox(
                        &mut self.config.fans_enabled,
                        "启用 Fans 聚合保存（按 viewer_id 覆盖更新）",
                    );
                    if !self.config.fans_enabled {
                        ui.label(
                            RichText::new("关闭后仍会保存 debug 包，但不写 fans 日文件")
                                .size(12.0)
                                .color(colors::TEXT_SECONDARY),
                        );
                    }
                });

                ui.add_space(8.0);
                field_label(ui, "Fans 输出目录");
                input_frame(ui, |ui| {
                    ui.add_enabled_ui(self.config.fans_enabled, |ui| {
                        let input_width = (ui.available_width() - 110.0).max(120.0);
                        ui.horizontal(|ui| {
                            ui.add_sized(
                                egui::vec2(input_width, 36.0),
                                egui::TextEdit::singleline(&mut self.fans_output_dir_input)
                                    .hint_text("默认: EXE 同级 fans/")
                                    .text_color(colors::TEXT_PRIMARY),
                            );
                            if ui.add(secondary_button("选择目录")).clicked() {
                                if let Some(path) = rfd::FileDialog::new()
                                    .set_title("选择 Fans 输出目录")
                                    .pick_folder()
                                {
                                    self.fans_output_dir_input = path.display().to_string();
                                }
                            }
                        });
                    });
                    if !self.config.fans_enabled {
                        ui.add_space(4.0);
                        ui.label(
                            RichText::new("Fans 保存已关闭，此目录暂不生效")
                                .size(12.0)
                                .color(colors::TEXT_MUTED),
                        );
                    }
                });
            },
        );

        ui.add_space(10.0);

        section_card(
            ui,
            "性能参数",
            "保留 FPS / VSync 逻辑与取值语义",
            |ui| {
                field_label(ui, "目标 FPS");
                ui.horizontal_wrapped(|ui| {
                    for (fps, label) in [(-1, "默认"), (60, "60"), (120, "120"), (144, "144")] {
                        let selected = self.config.target_fps == fps;
                        if ui.add(chip_button(label, selected)).clicked() {
                            self.config.target_fps = fps;
                            self.custom_fps_input.clear();
                        }
                    }
                });

                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("自定义 FPS")
                            .size(12.0)
                            .color(colors::TEXT_SECONDARY),
                    );
                    if ui
                        .add_sized(
                            egui::vec2(90.0, 32.0),
                            egui::TextEdit::singleline(&mut self.custom_fps_input)
                                .hint_text("例如 90")
                                .text_color(colors::TEXT_PRIMARY),
                        )
                        .changed()
                    {
                        if let Ok(v) = self.custom_fps_input.trim().parse::<i32>() {
                            if v > 0 {
                                self.config.target_fps = v;
                            }
                        }
                    }
                });
                if !self.validation.custom_fps_valid {
                    ui.label(
                        RichText::new("自定义 FPS 必须是正整数")
                            .size(12.0)
                            .color(colors::ERROR),
                    );
                }

                ui.add_space(6.0);
                field_label(ui, "VSync");
                ui.horizontal_wrapped(|ui| {
                    for (vsync, label) in [(-1, "默认"), (0, "关闭"), (1, "开启")] {
                        let selected = self.config.vsync_count == vsync;
                        if ui.add(chip_button(label, selected)).clicked() {
                            self.config.vsync_count = vsync;
                        }
                    }
                });

                ui.add_space(8.0);
                let save_enabled = self.validation.save_disabled_reason.is_none();
                let save_resp = ui.add_enabled(save_enabled, primary_button("保存配置"));
                if save_resp.clicked() {
                    self.do_save_config();
                }
                if !save_enabled {
                    if let Some(reason) = &self.validation.save_disabled_reason {
                        save_resp.on_hover_text(reason);
                    }
                }
            },
        );
    }

    fn render_debug_tab(&mut self, ui: &mut egui::Ui) {
        section_card(
            ui,
            "Debug 模式",
            "开启后，payload 会解码并写入本地 debug/ 目录",
            |ui| {
                let can_toggle = self.has_valid_game_context();
                let toggle_resp = ui.add_enabled(
                    can_toggle,
                    egui::Checkbox::new(&mut self.config.debug_mode, "开启 Debug 模式并自动保存"),
                );
                if toggle_resp.changed() {
                    self.apply_debug_output_dir();
                    self.ensure_fans_output_dir();
                    let path = PathBuf::from(&self.game_path);
                    match self.save_config_to_targets(&path) {
                        Ok(()) => self.set_status(
                            format!(
                                "Debug 模式已{}并保存",
                                if self.config.debug_mode {
                                    "开启"
                                } else {
                                    "关闭"
                                }
                            ),
                            StatusLevel::Success,
                        ),
                        Err(e) => self
                            .set_status(format!("保存 Debug 模式失败: {}", e), StatusLevel::Error),
                    }
                }
                if !can_toggle {
                    toggle_resp.on_hover_text("请先检测游戏目录，再切换 Debug 模式");
                }

                ui.add_space(6.0);
                ui.add(
                    egui::Label::new(
                        RichText::new(format!("输出目录：{}", self.current_debug_output_dir()))
                            .size(12.0)
                            .color(colors::TEXT_SECONDARY),
                    )
                    .wrap(),
                );
                ui.label(
                    RichText::new(
                        "注意：仅保留基础 hover 反馈，不启用额外动画。状态消息会在 6 秒后自动消失。",
                    )
                    .size(12.0)
                    .color(colors::TEXT_SECONDARY),
                );
            },
        );
    }

    fn render_console_tab(&mut self, ui: &mut egui::Ui) {
        section_card(
            ui,
            "内置接收器控制台",
            "查看接收、解码与 Fans 聚合日志",
            |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.label(
                        RichText::new(format!("状态：{}", self.receiver_status))
                            .size(12.0)
                            .color(colors::TEXT_SECONDARY),
                    );
                    if ui.add(secondary_button("清空日志")).clicked() {
                        crate::receiver::clear_logs();
                        self.set_status("已清空内置接收器日志", StatusLevel::Info);
                    }
                });

                ui.add_space(8.0);
                let lines = crate::receiver::snapshot_logs(600);
                if lines.is_empty() {
                    ui.label(
                        RichText::new("暂无接收器日志")
                            .size(12.0)
                            .color(colors::TEXT_MUTED),
                    );
                    return;
                }

                input_frame(ui, |ui| {
                    egui::ScrollArea::vertical()
                        .max_height(420.0)
                        .show(ui, |ui| {
                            for line in lines {
                                ui.label(
                                    RichText::new(line)
                                        .size(11.0)
                                        .monospace()
                                        .color(colors::TEXT_SECONDARY),
                                );
                            }
                        });
                });
            },
        );
    }

    fn render_status_bar(&self, ui: &mut egui::Ui) {
        egui::Frame::default()
            .fill(colors::STATUS_BAR_BG)
            .stroke(egui::Stroke::new(1.0, colors::BORDER))
            .inner_margin(egui::Margin::symmetric(12, 10))
            .show(ui, |ui| {
                if let Some((msg, level)) = &self.status_message {
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(level.label())
                                .size(12.0)
                                .color(level.color())
                                .strong(),
                        );
                        ui.label(RichText::new(msg).size(12.0).color(colors::TEXT_PRIMARY));
                    });
                } else {
                    ui.label(
                        RichText::new("状态：就绪")
                            .size(12.0)
                            .color(colors::TEXT_SECONDARY),
                    );
                }
            });
    }
}

impl eframe::App for ConfigApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.status_time.is_some() {
            ctx.request_repaint_after(Duration::from_millis(250));
        }
        if self.active_tab == MainTab::Console {
            ctx.request_repaint_after(Duration::from_millis(300));
        }
        if let Some(t) = self.status_time {
            if t.elapsed() > Duration::from_secs(6) {
                self.status_message = None;
                self.status_time = None;
            }
        }

        self.validate_inputs();

        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            self.render_status_bar(ui);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            self.render_header(ui);
            ui.add_space(10.0);
            self.render_global_action_bar(ui);
            ui.add_space(10.0);
            self.render_tab_bar(ui);
            ui.add_space(10.0);

            egui::ScrollArea::vertical().show(ui, |ui| match self.active_tab {
                MainTab::Overview => self.render_overview_tab(ui),
                MainTab::GameDeploy => self.render_game_deploy_tab(ui),
                MainTab::ForwardPerf => self.render_forward_perf_tab(ui),
                MainTab::Debug => self.render_debug_tab(ui),
                MainTab::Console => self.render_console_tab(ui),
            });
        });
    }
}

fn is_custom_fps(v: i32) -> bool {
    !matches!(v, -1 | 60 | 120 | 144) && v > 0
}

fn surface_card<R>(ui: &mut egui::Ui, add_contents: impl FnOnce(&mut egui::Ui) -> R) -> R {
    egui::Frame::default()
        .fill(colors::SURFACE)
        .stroke(egui::Stroke::new(1.0, colors::BORDER))
        .corner_radius(10)
        .inner_margin(egui::Margin::same(14))
        .show(ui, |ui| add_contents(ui))
        .inner
}

fn section_card<R>(
    ui: &mut egui::Ui,
    title: &str,
    subtitle: &str,
    add_contents: impl FnOnce(&mut egui::Ui) -> R,
) -> R {
    surface_card(ui, |ui| {
        ui.label(
            RichText::new(title)
                .size(16.0)
                .strong()
                .color(colors::TEXT_PRIMARY),
        );
        ui.label(
            RichText::new(subtitle)
                .size(12.0)
                .color(colors::TEXT_SECONDARY),
        );
        ui.add_space(8.0);
        add_contents(ui)
    })
}

fn input_frame<R>(ui: &mut egui::Ui, add_contents: impl FnOnce(&mut egui::Ui) -> R) -> R {
    egui::Frame::default()
        .fill(colors::INPUT_BG)
        .stroke(egui::Stroke::new(1.0, colors::BORDER))
        .corner_radius(8)
        .inner_margin(egui::Margin::symmetric(10, 6))
        .show(ui, |ui| add_contents(ui))
        .inner
}

fn field_label(ui: &mut egui::Ui, text: &str) {
    ui.label(RichText::new(text).size(12.0).color(colors::TEXT_SECONDARY));
}

fn kv_row(ui: &mut egui::Ui, key: &str, value: &str, value_color: Color32) {
    ui.horizontal(|ui| {
        ui.add_sized(
            egui::vec2(88.0, 0.0),
            egui::Label::new(RichText::new(key).size(12.0).color(colors::TEXT_SECONDARY)),
        );
        ui.add_sized(
            egui::vec2(ui.available_width(), 0.0),
            egui::Label::new(RichText::new(value).size(12.0).color(value_color)).wrap(),
        );
    });
    ui.add_space(4.0);
}

fn status_badge(ui: &mut egui::Ui, level: StatusLevel, text: &str) {
    egui::Frame::default()
        .fill(level.background())
        .stroke(egui::Stroke::new(1.0, colors::BORDER))
        .corner_radius(8)
        .inner_margin(egui::Margin::symmetric(10, 6))
        .show(ui, |ui| {
            ui.label(RichText::new(text).size(12.0).color(level.color()).strong());
        });
}

fn primary_button<'a>(text: &'a str) -> egui::Button<'a> {
    egui::Button::new(
        RichText::new(text)
            .color(colors::PRIMARY_TEXT)
            .strong()
            .size(12.0),
    )
    .fill(colors::PRIMARY)
    .stroke(egui::Stroke::new(1.0, colors::PRIMARY_HOVER))
    .corner_radius(8)
    .min_size(egui::vec2(0.0, 38.0))
}

fn secondary_button<'a>(text: &'a str) -> egui::Button<'a> {
    egui::Button::new(RichText::new(text).color(colors::TEXT_PRIMARY).size(12.0))
        .fill(colors::BUTTON_SECONDARY_BG)
        .stroke(egui::Stroke::new(1.0, colors::BORDER_STRONG))
        .corner_radius(8)
        .min_size(egui::vec2(0.0, 38.0))
}

fn danger_button<'a>(text: &'a str) -> egui::Button<'a> {
    egui::Button::new(
        RichText::new(text)
            .color(colors::BUTTON_DANGER_TEXT)
            .size(12.0),
    )
    .fill(colors::BUTTON_DANGER_BG)
    .stroke(egui::Stroke::new(1.0, colors::ERROR))
    .corner_radius(8)
    .min_size(egui::vec2(0.0, 38.0))
}

fn tab_button<'a>(text: &'a str, active: bool) -> egui::Button<'a> {
    let fill = if active {
        colors::TAB_ACTIVE_BG
    } else {
        colors::SURFACE
    };
    let stroke = if active {
        colors::TAB_ACTIVE_BORDER
    } else {
        colors::BORDER
    };
    let text_color = if active {
        colors::TAB_TEXT_ACTIVE
    } else {
        colors::TAB_TEXT_INACTIVE
    };

    egui::Button::new(RichText::new(text).size(12.0).color(text_color).strong())
        .fill(fill)
        .stroke(egui::Stroke::new(1.0, stroke))
        .corner_radius(8)
        .min_size(egui::vec2(108.0, 36.0))
}

fn chip_button<'a>(text: &'a str, active: bool) -> egui::Button<'a> {
    if active {
        egui::Button::new(
            RichText::new(text)
                .size(12.0)
                .color(colors::PRIMARY)
                .strong(),
        )
        .fill(colors::TAB_ACTIVE_BG)
        .stroke(egui::Stroke::new(1.0, colors::TAB_ACTIVE_BORDER))
        .corner_radius(8)
        .min_size(egui::vec2(64.0, 32.0))
    } else {
        egui::Button::new(RichText::new(text).size(12.0).color(colors::TEXT_SECONDARY))
            .fill(colors::SURFACE_STRONG)
            .stroke(egui::Stroke::new(1.0, colors::BORDER))
            .corner_radius(8)
            .min_size(egui::vec2(64.0, 32.0))
    }
}
