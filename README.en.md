<p align="center">
  <a href="README.md">한국어</a> | <strong>English</strong>
</p>

<p align="center">
  <img src="public/logo.png" width="120" alt="Agent Toast Logo">
</p>

<h1 align="center">Agent Toast</h1>

<p align="center">
  <strong>No more babysitting the terminal</strong><br>
  Agent Toast pings you the instant your agent needs you, then clicks you right back
</p>

<p align="center">
  <a href="https://github.com/hopoduck/agent-toast/releases"><img src="https://img.shields.io/github/v/release/hopoduck/agent-toast?style=flat-square" alt="Release"></a>
  <a href="https://github.com/hopoduck/agent-toast/blob/master/LICENSE"><img src="https://img.shields.io/github/license/hopoduck/agent-toast?style=flat-square" alt="License"></a>
  <img src="https://img.shields.io/badge/platform-Windows-blue?style=flat-square" alt="Platform">
  <img src="https://img.shields.io/endpoint?url=https%3A%2F%2Fagent-toast-stats.hopoduck.com%2Fv1%2Fbadge&style=flat-square" alt="Toasts shown worldwide">
</p>

<p align="center">
  <img src=".github/media/intro.en.webp" width="720" alt="Agent Toast Preview">
</p>

## ✨ Features

- **One click** activates the exact terminal window that raised the notification
- **Auto-dismiss** when you return to the terminal, and no toast at all if you're already looking at it
- Shows the **agent's last message** in the notification body (tool description on permission requests)
- **15 hook events** including task completion, permission requests, input waiting, and session start/end
- Receives Claude Code hook notifications from **remote Linux servers** as desktop toasts
- Pick any corner of any monitor in a **multi-monitor** setup, with DPI scaling
- **System alert sound** so you never miss an event (toggleable in settings)
- Follows the system **light/dark theme**, and hovering pauses auto-dismiss
- Tune bar, border, background, effects, density (comfortable/compact), and sans/mono system fonts with a **live preview** (D2Coding bundled)
- **Stats and insights** aggregated from shown, clicked, and auto-dismissed events
- **Korean/English** UI
- New version notifications with **one-click auto-update**

## 🖼️ Screenshots

|                               Toast Design Customization                               |                                    Notification Stats                                    |
| :-------------------------------------------------------------------------------------: | :-----------------------------------------------------------------------------------------: |
| <img src=".github/media/settings-design.en.png" width="380" alt="Design settings tab"> | <img src=".github/media/settings-stats.en.png" width="380" alt="Notification stats tab"> |

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

> 💡 By default the notification body shows the agent's last message (or the tool description on permission requests). Turn off **Use Agent's Message** in the General tab to show each hook's fixed text instead.

| Platform    | Config File               |
| ----------- | ------------------------- |
| Claude Code | `~/.claude/settings.json` |
| Codex CLI   | `~/.codex/config.toml`    |

## ⚙️ How It Works

- The first launch starts the app; subsequent CLI calls just send JSON through a Named Pipe and exit immediately (single instance)
- Detects focus changes in real time via Win32 API to auto-dismiss notifications
- Walks the process tree up from `--pid` to find the terminal window that raised the notification

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

agent-toast-send init --url http://<desktop-ip>:38787 --dynamic [--hostname "prod"]
```

- `<desktop-ip>` is the address reachable from the server to your desktop (Tailscale, LAN, SSH `-R`). Network reachability is the user's responsibility and is not managed by the app.
- `--dynamic` shows the agent's last message (or the tool description on permission requests) as the notification body (omit for fixed text).
- `--hostname` sets the label shown in the toast (omit to auto-detect via `hostname(1)`).
- By default, the **Stop** (task completion) and **Notification** (permission request) hooks are registered. For finer customization, edit `~/.claude/settings.json` on the server directly.

To uninstall, run `agent-toast-send uninstall`. It only removes agent-toast related hooks and preserves everything else.

</details>

## 🌍 Global stats (anonymous)

The app anonymously uploads notification counters (shown/clicked/closed counts) to an aggregation server and shows worldwide totals in the stats tab and the badge above.

- The only data sent is cumulative per-event/source counters and a random ID generated at install
- Hostnames, file paths, message contents, and anything else identifying are **never sent**
- Turn it off with the **Share anonymous stats** toggle in Settings → Stats tab

## 🤔 Why a custom notification window?

An OS-native toast's job ends the moment it pops up. Agent Toast's notifications also handle the trip back to your terminal.

<p align="center">
  <img src=".github/media/toast.en.png" width="452" alt="Agent Toast notification toast">
</p>

- Clicking the toast activates the exact terminal window that raised it
- The toast closes itself when focus returns to the terminal
- If you're already looking at that terminal, no toast appears in the first place

This behavior requires knowing which window raised each notification, which is why Agent Toast uses its own notification window instead of native toasts.

## 🔍 Comparison with Other Notification Tools

|                                  | **Agent Toast**                 | [**Toasty**](https://github.com/shanselman/toasty) | [**claude-code-notification**](https://github.com/wyattjoh/claude-code-notification) | **PowerShell Script** | [**ntfy.sh**](https://ntfy.sh) |
| -------------------------------- | ------------------------------- | -------------------------------------------------- | ------------------------------------------------------------------------------------ | --------------------- | ------------------------------ |
| **Notification Style**           | Custom notification window      | OS native toast                                    | OS native toast                                                                      | OS native toast       | HTTP push notification         |
| **Platform**                     | Windows                         | Windows                                            | Windows · macOS · Linux                                                              | Windows               | All (incl. mobile)             |
| **Installation**                 | Installer / Portable            | CLI binary                                         | CLI binary                                                                           | Copy script           | One-line curl                  |
| **GUI Settings**                 | ✅ Settings window               | ❌ CLI only                                         | ❌ CLI only                                                                           | ❌ Manual edit         | ❌ Manual edit                  |
| **Design Customization**         | ✅ Bar, fonts, density, etc.     | ❌                                                  | ❌                                                                                    | ❌                     | ❌                              |
| **Notification Stats**           | ✅                               | ❌                                                  | ❌                                                                                    | ❌                     | ❌                              |
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
