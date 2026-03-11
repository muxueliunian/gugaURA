# gugaURA

赛马娘（Umamusume）数据抓取、配置与本地转发工具。

借鉴与感谢：

- Hachimi: [https://github.com/Hachimi-Hachimi/Hachimi](https://github.com/Hachimi-Hachimi/Hachimi)
- URA / UmamusumeResponseAnalyzer: [https://github.com/EtherealAO/UmamusumeResponseAnalyzer](https://github.com/EtherealAO/UmamusumeResponseAnalyzer)

本项目在注入链路、数据接收与本地分析配套思路上参考了 Hachimi 与 URA 的公开方案，感谢这两个项目的开源工作。

当前唯一受支持的桌面配置工具入口是 `guga_ura_config_tauri`。

## 当前产物

- `guga_ura`
  - 游戏侧 DLL 负载
  - 负责请求/响应抓取、FPS 和 VSync 控制
- `guga_ura_config_tauri`
  - 当前主配置工具
  - 负责目录检测、安装/卸载、配置保存、Receiver 与 Relay 管理
- `guga_ura_config_core`
  - 配置工具复用的核心能力层
- `guga_ura_receiver`
  - 可独立运行的本地接收器

当前推荐分发物：

- `target/release/bundle/nsis/gugaURA_installer.exe`

开发态裸 EXE：

- `target/release/guga_ura_config_tauri.exe`
  - 仅适合本机验证
  - 仍要求系统已具备 WebView2

## 主要能力

- 抓取游戏 HTTP 请求与响应数据，原始内容为 msgpack
- 将原始数据发送到自定义 `notifier_host`
- 在本地 Receiver 侧保存调试 JSON
- 支持 Receiver 监听地址与 Relay 目标地址分离配置
- 支持接收后继续二次转发
- 支持 FPS 解锁与 VSync 控制

## 工作方式

```text
游戏 -> gugaURA DLL -> notifier_host
                     -> POST /notify/request
                     -> POST /notify/response

Receiver -> 保存 debug JSON
         -> 可选 fans 聚合
         -> 可选 relay 到 relay_target_host
```

## 构建

```bash
# 安装 Rust 目标
rustup target add x86_64-pc-windows-msvc

# 构建 DLL 负载
cargo build --release -p guga_ura -p cellar

# 构建 Tauri 配置工具与安装器
cd guga_ura_config_tauri
pnpm install
pnpm build:windows-installer
```

关键产物：

- `target/release/bundle/nsis/gugaURA_installer.exe`
- `target/release/guga_ura_config_tauri.exe`
- `target/release/UnityPlayer.dll`
- `target/release/apphelp.dll`

## 安装与使用

### 方式 1：安装器分发

1. 分发 `target/release/bundle/nsis/gugaURA_installer.exe`
2. 安装后启动 `gugaURA`
3. 选择游戏目录
4. 点击“安装 DLL”
5. 按需保存注入、Receiver、Relay、FPS 与 VSync 配置

说明：

- 安装器是默认分发形态
- 若目标机器缺少 WebView2 Runtime，安装器会通过 Microsoft bootstrapper 联网下载安装
- 前端资源和配置相关 DLL 已内嵌到配置工具链路

### 方式 2：直接运行裸 EXE

1. 在已安装 WebView2 的 Windows 机器上运行 `target/release/guga_ura_config_tauri.exe`
2. 进入配置工具后完成安装与配置

说明：

- 更适合开发和本机验证
- 不建议替代安装器作为默认发布方式

### 方式 3：手动安装 DLL

1. 备份游戏目录下的 `UnityPlayer.dll`
2. 将 `UnityPlayer.dll` 与 `apphelp.dll` 复制到游戏目录
3. 确保游戏目录配置中的 `notifier_host` 指向一个可达的 HTTP 接收端

## 配置说明

当前有三类地址：

- `receiver_listen_addr`
  - Receiver 实际监听地址
- `notifier_host`
  - DLL 将原始 msgpack 发送到的目标基地址
- `relay_target_host`
  - Receiver 收到后再转发到的目标基地址

配置文件仍为 `guga_ura_config.json`，会写到游戏目录和配置工具 EXE 同级目录，对应作用不同：

- 游戏目录配置主要影响 DLL 发送侧
- EXE 同级配置主要影响 Receiver、Relay 与 Fans 行为

示例：

```json
{
  "receiver_listen_addr": "127.0.0.1:4693",
  "notifier_host": "http://127.0.0.1:4693",
  "relay_enabled": false,
  "relay_target_host": null,
  "timeout_ms": 100,
  "target_fps": 60,
  "vsync_count": 0,
  "fans_enabled": true,
  "fans_output_dir": null
}
```

字段说明：

| 配置项 | 说明 |
| --- | --- |
| `receiver_listen_addr` | Receiver 监听地址，默认 `127.0.0.1:4693` |
| `notifier_host` | DLL 发送目标基地址，实际会自动拼接 `/notify/request` 和 `/notify/response` |
| `relay_enabled` | 是否开启 Receiver 二次转发 |
| `relay_target_host` | Receiver 的二次转发目标基地址 |
| `timeout_ms` | HTTP 超时时间，单位毫秒 |
| `target_fps` | 目标帧率，`-1` 表示游戏默认 |
| `vsync_count` | `-1 = 默认`，`0 = 关闭`，`1 = 开启` |
| `fans_enabled` | 是否启用 Receiver 侧 fans 聚合保存 |
| `fans_output_dir` | fans 输出目录；为空时默认 EXE 同级 `fans/` |

## 接收与路由

DLL 会把原始 msgpack 二进制 POST 到目标地址：

- `POST /notify/request`
- `POST /notify/response`

内置 Receiver 默认监听：

- `127.0.0.1:4693`

兼容路由：

- `POST /notify/request`
- `POST /notify/response`
- `POST /*`

Relay 规则：

- 仅在 `relay_enabled = true` 且 `relay_target_host` 有值时触发
- 透传原始 body、原始路径、`Content-Type` 与 `x-plugin-name`
- 自动增加 `x-gugaura-relayed: 1`
- 自环目标会被阻止
- relay 失败不会影响本地保存和 fans 聚合

独立接收器示例：

```bash
cargo run -p guga_ura_receiver --release -- --host 127.0.0.1 --port 4700 --output-dir C:\\temp\\uma_debug
```

## 项目结构

```text
gugaURA/
├── guga_ura/               # DLL 负载
├── guga_ura_config_core/   # 配置核心能力
├── guga_ura_config_tauri/  # 当前主配置工具
├── guga_ura_receiver/      # 独立本地接收器
└── cellar/                 # 反检测相关 DLL
```

## 许可证

GPL-3.0
