# Progress Reporter (Tauri)

Linux 桌面上的进程 & 媒体上报工具，使用 Tauri 重构，支持 X11 与 KDE Wayland（通过 `qdbus` 获取活跃窗口）。前端是简洁面板，后端复用了 Rust 采集与上报逻辑。

## 运行

1) 安装依赖（Ubuntu/Debian 示例）

```bash
sudo apt update && sudo apt install -y libdbus-1-dev pkg-config libssl-dev gcc qdbus-qt5
cargo install tauri-cli --version '^2.0.0' --locked
```

确保 `XDG_SESSION_TYPE` 正确（`x11` 或 `wayland`），KDE Wayland 需要 `qdbus` 可用。

2) 配置 `.env.process`（放在运行目录或 UI 中指定路径）

```ini
API_URL=https://api.example.cn/api/v2/fn/ps/update
API_KEY=your_key
WATCH_TIME=5
MEDIA_ENABLE=true
LOG_ENABLE=true
```

3) 本地启动

```bash
cd app
cargo tauri dev
```

4) 打包

```bash
cd app
cargo tauri build
```

## 功能简述

- 后端：
	- X11：使用 `x11rb` 直接读取 `_NET_ACTIVE_WINDOW` / `WM_CLASS`。
	- KDE Wayland：调用 `qdbus org.kde.KWin /KWin supportInformation` 解析当前窗口类名。
	- 媒体：通过 MPRIS（D-Bus）扫描 `org.mpris.MediaPlayer2.*` 读取 `xesam:title/artist`。
	- 上报：统一 JSON 结构 `{ key, process, timestamp, media? }` 发送到 `API_URL`。

- 前端：
	- 指定配置文件路径、启动/停止 watcher、实时查看运行状态、最近进程/媒体、最后一次上报时间。

## 已知限制

- Wayland 下依赖 KDE 的 `qdbus` 输出；其他 Wayland 合成器暂未适配。
- 活跃窗口名取自类名/资源名，可能与窗口标题不同。
- 上报失败会静默重试（日志仅在终端输出）。
