## 致谢

本项目参考了以下开源项目：
- [Hachimi](https://github.com/Hachimi-Hachimi/Hachimi)
- [Hachimi/Cellar](https://github.com/Hachimi-Hachimi/Cellar)
- [UmamusumeResponseAnalyzer](https://github.com/EtherealAO/UmamusumeResponseAnalyzer)

# GugaURA - Uma Musume Data Capture Tool

精简版数据抓包工具，用于捕获游戏HTTP请求/响应数据。

## 功能

-  **单一职责**：只做数据抓包转发，不修改任何游戏数据
-  **轻量级**：代码量约1000行
-  **解耦设计**：通过HTTP转发数据，与后端服务完全解耦

## 工作原理

```
游戏 ────> GugaURA (DLL) ────HTTP───> 你的后端服务
              │                            │
              │ Hook                       │ 接收并解析
              ▼                            ▼
     CompressRequest              POST /notify/request
     DecompressResponse           POST /notify/response
```

## 编译

```bash
# 安装Rust工具链
rustup target add x86_64-pc-windows-msvc

# 编译Release版本（同时编译两个DLL）
cargo build --release

# 输出文件:
# - target/release/UnityPlayer.dll  (主体)
# - target/release/apphelp.dll      (反检测)
```

## 安装

1. 备份游戏目录下的 `UnityPlayer.dll`
2. 复制以下文件到游戏目录：
   - `target/release/UnityPlayer.dll`
   - `target/release/apphelp.dll`
3. 确保你的后端服务在监听 `http://127.0.0.1:4693`

> ⚠️ **重要**：必须同时安装两个DLL。`apphelp.dll` 负责绕过游戏的反检测机制。

## 配置

首次运行后会在游戏目录生成 `guga_ura_config.json`：

```json
{
  "notifier_host": "http://127.0.0.1:4693",
  "timeout_ms": 100
}
```

## 数据格式

GugaURA会将原始的msgpack二进制数据POST到你的服务：

- `POST /notify/request` - 游戏发送的请求数据
- `POST /notify/response` - 游戏收到的响应数据

## 项目结构

```
gugaURA/
├── Cargo.toml          # Workspace配置
├── guga_ura/           # 主项目 → UnityPlayer.dll
│   ├── Cargo.toml
│   └── src/
└── cellar/             # 反检测 → apphelp.dll
    ├── Cargo.toml
    └── src/
```

## 许可证

GPL-3.0 (基于Hachimi项目)


