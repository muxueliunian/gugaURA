//! 启动期 DTO

use crate::state::AppState;
use serde::Serialize;

/// 启动页所需的基础状态
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BootstrapStateDto {
    pub app_version: String,
    pub receiver_ready: bool,
    pub receiver_status: String,
    pub receiver_listen_addr: String,
    pub receiver_configured_listen_addr: String,
    pub receiver_listen_addr_source: String,
}

impl BootstrapStateDto {
    /// 基于共享状态创建启动信息
    pub fn from_state(app_version: &str, state: &AppState) -> Self {
        Self {
            app_version: app_version.to_string(),
            receiver_ready: state.receiver_ready(),
            receiver_status: state.receiver_status().to_string(),
            receiver_listen_addr: state.receiver_listen_addr().to_string(),
            receiver_configured_listen_addr: state.receiver_configured_listen_addr().to_string(),
            receiver_listen_addr_source: state.receiver_listen_addr_source().to_string(),
        }
    }
}
