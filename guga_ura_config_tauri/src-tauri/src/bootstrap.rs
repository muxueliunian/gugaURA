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
        let runtime = state.receiver_runtime();

        Self {
            app_version: app_version.to_string(),
            receiver_ready: runtime.ready,
            receiver_status: runtime.status,
            receiver_listen_addr: runtime.listen_addr,
            receiver_configured_listen_addr: runtime.configured_listen_addr,
            receiver_listen_addr_source: runtime.source.as_str().to_string(),
        }
    }
}
