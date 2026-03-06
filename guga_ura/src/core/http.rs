//! HTTP客户端模块

use std::time::Duration;

/// POST字节数据到指定URL
pub fn post_bytes(url: &str, data: &[u8], timeout_ms: u64) {
    // 使用单独的线程发送，避免阻塞游戏
    let url = url.to_string();
    let data = data.to_vec();
    let timeout = Duration::from_millis(timeout_ms.max(1));

    std::thread::spawn(move || {
        let agent = ureq::AgentBuilder::new()
            .timeout_connect(timeout)
            .timeout_read(timeout)
            .timeout_write(timeout)
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
