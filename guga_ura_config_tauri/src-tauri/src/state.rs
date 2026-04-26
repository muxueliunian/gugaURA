//! Tauri 应用共享状态

use guga_ura_config_core::receiver::{
    EmbeddedReceiverHandle, ReceiverRuntimeInfo, StartedEmbeddedReceiver,
};
use std::sync::Mutex;

/// 启动期共享状态
pub struct AppState {
    receiver_runtime: Mutex<ReceiverRuntimeInfo>,
    receiver_handle: Mutex<Option<EmbeddedReceiverHandle>>,
}

impl AppState {
    /// 创建应用状态
    pub fn new(receiver: StartedEmbeddedReceiver) -> Self {
        Self {
            receiver_runtime: Mutex::new(receiver.runtime),
            receiver_handle: Mutex::new(receiver.handle),
        }
    }

    /// 获取接收器运行时快照
    pub fn receiver_runtime(&self) -> ReceiverRuntimeInfo {
        match self.receiver_runtime.lock() {
            Ok(guard) => guard.clone(),
            Err(poisoned) => poisoned.into_inner().clone(),
        }
    }

    /// 更新接收器运行时快照
    pub fn update_receiver_runtime(&self, receiver_runtime: ReceiverRuntimeInfo) {
        match self.receiver_runtime.lock() {
            Ok(mut guard) => *guard = receiver_runtime,
            Err(poisoned) => *poisoned.into_inner() = receiver_runtime,
        }
    }

    /// 使用新接收器替换当前接收器
    pub fn replace_receiver(
        &self,
        receiver: StartedEmbeddedReceiver,
    ) -> Option<EmbeddedReceiverHandle> {
        self.update_receiver_runtime(receiver.runtime);
        match self.receiver_handle.lock() {
            Ok(mut guard) => guard.replace(receiver.handle?),
            Err(poisoned) => poisoned.into_inner().replace(receiver.handle?),
        }
    }
}
