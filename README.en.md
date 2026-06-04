<p align="center">
  <a href="README.md">한국어</a> | <strong>English</strong>
</p>

<p align="center">
  <img src="public/logo.png" width="120" alt="Agent Toast Logo">
</p>

<h1 align="center">Agent Toast</h1>

<p align="center">
  <strong>Stop watching the terminal while AI works</strong><br>
  Get notified the moment your agent is done
</p>

<p align="center">
  <a href="https://github.com/hopoduck/agent-toast/releases"><img src="https://img.shields.io/github/v/release/hopoduck/agent-toast?style=flat-square" alt="Release"></a>
  <a href="https://github.com/hopoduck/agent-toast/blob/master/LICENSE"><img src="https://img.shields.io/github/license/hopoduck/agent-toast?style=flat-square" alt="License"></a>
  <img src="https://img.shields.io/badge/platform-Windows-blue?style=flat-square" alt="Platform">
</p>

<p align="center">
  <img src=".github/media/intro.webp" width="720" alt="Agent Toast Preview">
</p>

## ✨ Features

- **Smart Notifications** - Click to activate terminal, auto-dismiss on focus return, skip if already focused
- **Agent Message Display** - Optionally show the agent's last message (or the tool description on permission requests) as the notification body
- **15 Hook Events** - Task completion, permission requests, input waiting, session start/end, and more
- **Remote Notifications** - Receive Claude Code hook notifications from remote Linux servers as desktop toasts
- **Multi-Monitor Support** - Display notifications on any corner of your preferred monitor with DPI scaling
- **Notification Sound** - System alert sound so you never miss an event (toggleable in settings)
- **Light/Dark Theme** - Toast design follows your system theme; hover to pause auto-dismiss
- **Multilingual UI** - Korean/English support
- **Auto Update** - New version notifications with one-click update

## 🔌 Supported Platforms

| Platform                                             | Description                          |
| ---------------------------------------------------- | ------------------------------------ |
| [Claude Code](https://www.anthropic.com/claude-code) | Anthropic's AI coding assistant      |
| [Codex CLI](https://openai.com/codex/)               | OpenAI's terminal-based coding agent |

## 📥 Installation

### Download from Releases

[**📦 Download Latest Version**](https://github.com/hopoduck/agent-toast/releases/latest)

### Build from Source

```bash
# Requirements: Node.js 18+, pnpm, Rust (MSVC toolchain)

pnpm install
pnpm tauri build
```

## 🚀 Usage

### 1. Open Settings

```bash
agent-toast.exe --setup
```

Or right-click the system tray icon → Settings

### 2. Configure Hooks

Enable desired events in the settings window to automatically register hooks.

> 💡 Turn on **Use Agent's Message** in the General tab to show the agent's last message (or the tool description on permission requests) as the notification body instead of each hook's fixed text.

| Platform    | Config File               |
| ----------- | ------------------------- |
| Claude Code | `~/.claude/settings.json` |
| Codex CLI   | `~/.codex/config.toml`    |

## ⚙️ How It Works

- Single-instance management via Named Pipe — first launch starts the app, subsequent CLI calls send JSON through the pipe and exit immediately
- Real-time focus detection via Win32 API for automatic notification dismissal
- Process tree traversal from `--pid` for improved terminal window detection accuracy

## 🌐 Remote Notifications (Linux Servers)

Receive Claude Code hook notifications from a remote Linux server as desktop toasts.

<details>
<summary><strong>Setup instructions</strong></summary>

### 1. Desktop: Enable HTTP Receiver

Settings window → **Remote Notifications** → toggle **Enable HTTP receiver** ON. The default port is `38787` (changeable in settings); the bind address is always `0.0.0.0`.

Windows Firewall may prompt for permission on first use. If you're using Tailscale or SSH port forwarding, allowing **private networks** only is sufficient.

### 2. Server: Install `agent-toast-send` + Register Hooks

```bash
curl -L https://github.com/hopoduck/agent-toast/releases/latest/download/agent-toast-send-linux-$(uname -m) \
  -o ~/.local/bin/agent-toast-send
chmod +x ~/.local/bin/agent-toast-send

agent-toast-send init --url http://<desktop-ip>:38787 [--hostname "prod"]
```

- `<desktop-ip>` is the address reachable from the server to your desktop (Tailscale, LAN, SSH `-R`). Network reachability is the user's responsibility and is not managed by the app.
- `--hostname` sets the label shown in the toast (omit to auto-detect via `hostname(1)`).
- Default hooks registered: **Stop** (task completion), **Notification** (permission request). For finer customization, edit `~/.claude/settings.json` on the server directly.

To uninstall, run `agent-toast-send uninstall` — only removes agent-toast related hooks; all other hooks are preserved.

</details>

## 🔍 Comparison with Other Notification Tools

|                                  | **Agent Toast**                 | [**Toasty**](https://github.com/shanselman/toasty) | [**claude-code-notification**](https://github.com/wyattjoh/claude-code-notification) | **PowerShell Script** | [**ntfy.sh**](https://ntfy.sh) |
| -------------------------------- | ------------------------------- | -------------------------------------------------- | ------------------------------------------------------------------------------------ | --------------------- | ------------------------------ |
| **Notification Style**           | Custom notification window      | OS native toast                                    | OS native toast                                                                      | OS native toast       | HTTP push notification         |
| **Platform**                     | Windows                         | Windows                                            | Windows · macOS · Linux                                                              | Windows               | All (incl. mobile)             |
| **Installation**                 | Installer / Portable            | CLI binary                                         | CLI binary                                                                           | Copy script           | One-line curl                  |
| **GUI Settings**                 | ✅ Settings window               | ❌ CLI only                                         | ❌ CLI only                                                                           | ❌ Manual edit         | ❌ Manual edit                  |
| **Smart Notifications**¹         | ✅                               | ❌                                                  | ❌                                                                                    | ❌                     | ❌                              |
| **Click → Activate Terminal**    | ✅                               | ❌                                                  | ❌                                                                                    | ❌                     | ❌                              |
| **Multi-Monitor · Position**     | ✅ 4 corners + monitor           | ❌                                                  | ❌                                                                                    | ❌                     | ❌                              |
| **DPI Scaling**                  | ✅                               | ❌                                                  | ❌                                                                                    | ❌                     | ❌                              |
| **Notification Sound**           | ✅                               | ❌                                                  | ✅                                                                                    | ❌                     | ✅                              |
| **Auto Update**                  | ✅                               | ❌                                                  | ❌                                                                                    | ❌                     | ❌                              |
| **Remote Server Notifications**² | ✅ Dedicated CLI + HTTP receiver | ❌                                                  | ❌                                                                                    | ❌                     | ✅                              |
| **Mobile Notifications**         | ❌                               | ✅ (via ntfy)                                       | ❌                                                                                    | ❌                     | ✅                              |
| **Multi AI Tool Support**        | Claude Code · Codex CLI         | Claude · Copilot · Gemini · Codex, etc.            | Claude Code                                                                          | Claude Code           | Universal                      |
| **Language**                     | Rust + TypeScript               | C++                                                | Rust                                                                                 | PowerShell            | Shell (curl)                   |

> ¹ **Smart Notifications**: Skip notification if terminal is already focused + auto-dismiss when terminal regains focus
>
> ² **Remote Server Notifications**: Agent hooks running on a remote Linux server show toasts on your desktop (Toasty's ntfy integration is desktop→mobile outbound only)

## 🛠️ Tech Stack

<p>
  <img src="https://img.shields.io/badge/Rust-000000?style=flat-square&logo=rust&logoColor=white" alt="Rust">
  <img src="https://img.shields.io/badge/Tauri-24C8D8?style=flat-square&logo=tauri&logoColor=white" alt="Tauri">
  <img src="https://img.shields.io/badge/Vue.js-4FC08D?style=flat-square&logo=vue.js&logoColor=white" alt="Vue.js">
  <img src="https://img.shields.io/badge/TypeScript-3178C6?style=flat-square&logo=typescript&logoColor=white" alt="TypeScript">
</p>

## 📄 License

[MIT License](LICENSE)
