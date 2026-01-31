# GugaURA - Uma Musume Data Capture Tool

精简版数据抓包工具，用于捕获游戏HTTP请求/响应数据。

## 更新日志

### v0.2.0 (2025-01-31)

**新功能：**
- FPS 解锁：支持自定义帧率 (60/120/144 或任意值)
- VSync 控制：支持开启/关闭垂直同步
- 图形化配置工具：一键安装、配置管理
- DLL 内嵌：配置工具内嵌 DLL，支持单文件分发

**修复：**
- 修复 FPS 设置不生效的问题

### v0.1.0

- 初始版本，HTTP 数据抓包功能

---

## 功能

- 数据抓包：捕获游戏HTTP请求/响应（msgpack格式）
- FPS 解锁：突破游戏默认30帧限制
- VSync 控制：手动控制垂直同步
- 轻量级：代码量约1500行
- 解耦设计：通过HTTP转发数据，与后端服务完全解耦

## 工作原理

```
游戏 ───> GugaURA (DLL) ───HTTP──> 你的后端服务
              │                            │
              │ Hook                       │ 接收并解析
              ▼                            ▼
     CompressRequest              POST /notify/request
     DecompressResponse           POST /notify/response
     set_targetFrameRate          (FPS控制)
     set_vSyncCount               (VSync控制)
```

## 编译

```bash
# 安装Rust工具链
rustup target add x86_64-pc-windows-msvc

# 编译Release版本（全部组件）
cargo build --release

# 输出文件:
# - target/release/UnityPlayer.dll    (主体DLL)
# - target/release/apphelp.dll        (反检测)
# - target/release/GugaURA_Config.exe (配置工具，内嵌DLL)
```

## 安装

### 方式1：使用配置工具（推荐）

1. 运行 `GugaURA_Config.exe`
2. 选择游戏目录
3. 点击"安装"按钮（DLL已内嵌，无需额外文件）
4. 配置 FPS/VSync 设置
5. 点击"保存"

### 方式2：手动安装

1. 备份游戏目录下的 `UnityPlayer.dll`
2. 复制以下文件到游戏目录：
   - `UnityPlayer.dll`
   - `apphelp.dll`
3. 确保你的后端服务在监听 `http://127.0.0.1:4693`

**重要**：必须同时安装两个DLL。`apphelp.dll` 负责绕过游戏的反检测机制。

## 配置

首次运行后会在游戏目录生成 `guga_ura_config.json`：

```json
{
  "notifier_host": "http://127.0.0.1:4693",
  "timeout_ms": 100,
  "target_fps": 60,
  "vsync_count": 0
}
```

| 配置项 | 说明 |
|--------|------|
| `notifier_host` | 后端服务地址 |
| `timeout_ms` | HTTP超时时间（毫秒） |
| `target_fps` | 目标帧率，-1=游戏默认 |
| `vsync_count` | VSync，-1=默认，0=关闭，1=开启 |

## 数据格式

GugaURA会将原始的msgpack二进制数据POST到你的服务：

- `POST /notify/request` - 游戏发送的请求数据
- `POST /notify/response` - 游戏收到的响应数据

## 项目结构

```
gugaURA/
├── Cargo.toml           # Workspace配置
├── guga_ura/            # 主项目 -> UnityPlayer.dll
│   ├── Cargo.toml
│   └── src/
├── guga_ura_config/     # 配置工具 -> GugaURA_Config.exe
│   ├── Cargo.toml
│   └── src/
└── cellar/              # 反检测 -> apphelp.dll
    ├── Cargo.toml
    └── src/
```

## 致谢

本项目参考了以下开源项目：
- [Hachimi](https://github.com/Hachimi-Hachimi/Hachimi)
- [Hachimi/Cellar](https://github.com/Hachimi-Hachimi/Cellar)
- [UmamusumeResponseAnalyzer](https://github.com/EtherealAO/UmamusumeResponseAnalyzer)

## 许可证

GPL-3.0 (基于Hachimi项目)
