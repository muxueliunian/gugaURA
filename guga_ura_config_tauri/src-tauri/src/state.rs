//! Tauri 应用共享状态

use guga_ura_config_core::receiver::ReceiverRuntimeInfo;

/// 启动期共享状态
pub struct AppState {
    receiver_runtime: ReceiverRuntimeInfo,
}

impl AppState {
    /// 创建应用状态
    pub fn new(receiver_runtime: ReceiverRuntimeInfo) -> Self {
        Self { receiver_runtime }
    }

    /// 获取接收器状态文案
    pub fn receiver_status(&self) -> &str {
        &self.receiver_runtime.status
    }

    /// 获取接收器是否已就绪
    pub fn receiver_ready(&self) -> bool {
        self.receiver_runtime.ready
    }

    /// 获取接收器当前监听地址
    pub fn receiver_listen_addr(&self) -> &str {
        &self.receiver_runtime.listen_addr
    }

    /// 获取配置中的监听地址
    pub fn receiver_configured_listen_addr(&self) -> &str {
        &self.receiver_runtime.configured_listen_addr
    }

    /// 获取监听地址来源
    pub fn receiver_listen_addr_source(&self) -> &str {
        self.receiver_runtime.source.as_str()
    }
}
