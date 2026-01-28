//! HTTP客户端模块

use std::time::Duration;
use once_cell::sync::Lazy;

/// HTTP Agent (复用连接)
static AGENT: Lazy<ureq::Agent> = Lazy::new(|| {
    ureq::AgentBuilder::new()
        .timeout_connect(Duration::from_millis(100))
        .timeout_read(Duration::from_millis(100))
        .timeout_write(Duration::from_millis(100))
        .build()
});

/// POST字节数据到指定URL
pub fn post_bytes(url: &str, data: &[u8], timeout_ms: u64) {
    // 使用单独的线程发送，避免阻塞游戏
    let url = url.to_string();
    let data = data.to_vec();
    
    std::thread::spawn(move || {
        let agent = ureq::AgentBuilder::new()
            .timeout_connect(Duration::from_millis(timeout_ms))
            .timeout_read(Duration::from_millis(timeout_ms))
            .timeout_write(Duration::from_millis(timeout_ms))
            .build();
        
        match agent.post(&url).send_bytes(&data) {
            Ok(_) => {
                // 发送成功，静默
            }
            Err(e) => {
                // 发送失败，只在调试模式下打印
                #[cfg(debug_assertions)]
                warn!("Failed to send data to {}: {}", url, e);
            }
        }
    });
}
