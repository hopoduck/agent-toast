# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Requirements

- Node.js 18+
- pnpm
- Rust (MSVC toolchain on Windows)

## Build & Dev Commands

```bash
pnpm install                # Install dependencies
pnpm tauri dev              # Full dev mode with Vite hot reload (port 1420)
pnpm tauri build            # Production build → src-tauri/target/release/agent-toast.exe
pnpm build                  # vue-tsc --noEmit (type check) + vite build (frontend only)
```

### Lint & Format

```bash
# Rust
cd src-tauri && cargo fmt          # Code formatting
cd src-tauri && cargo fmt --check  # Format check (for CI)
cd src-tauri && cargo clippy       # Lint check

# TypeScript
pnpm vue-tsc --noEmit              # Type check only
```

## Architecture

**Single-instance Tauri v2 desktop app** for Windows that shows smart notifications for Claude Code events.

### Process Model

- First invocation (`--pid --event`): tries Named Pipe → if no listener, launches Tauri app
- Subsequent invocations: connects to pipe `\\.\pipe\agent-toast`, sends JSON, exits immediately
- No args or `--daemon`: starts app without initial notification
- Singleton enforced via `CreateMutexW("agent-toast-singleton")`

### Backend ↔ Frontend IPC

- **Rust → Frontend**: emits `notification-data` event with `NotificationData` struct
- **Frontend → Rust**: `invoke("close_notify", { id })`, `invoke("activate_source", { hwnd, id })`
- **Initial load**: Frontend calls `invoke("get_notification_data")` on mount (event may arrive before listener is ready)

### Window Routing

Single `index.html` + `main.ts` serves both notification and setup windows. Window label (`setup` vs `notify-*`) determines which Vue component mounts — no router needed.

## Rust Backend (src-tauri/src/)

| Module            | Purpose                                                                                             |
| ----------------- | --------------------------------------------------------------------------------------------------- |
| `main.rs`         | CLI entry, single-instance routing via pipe, parent PID auto-detection                              |
| `lib.rs`          | Tauri app setup, command registration, tray icon                                                    |
| `cli.rs`          | clap arg parsing, `NotifyRequest` struct                                                            |
| `pipe.rs`         | Named Pipe server/client (Windows-only, stubs for other OS)                                         |
| `notification.rs` | Notification lifecycle, window creation, 4-corner positioning with DPI scaling                      |
| `win32.rs`        | Process tree walking, focus detection, window activation (Windows-only)                             |
| `setup.rs`        | Settings file I/O (`~/.claude/settings.json`), hook config builder, preserves non-agent-toast hooks |
| `sound.rs`        | System notification sound via `PlaySoundW`                                                          |
| `updater.rs`      | Auto-update check via GitHub API (12-hour interval), update notification                            |

### Critical Win32 Logic

- **Process tree**: walks parent chain from `--pid` up to 20 levels to find the terminal window. Tree is resolved eagerly in `main.rs` before pipe send (avoids race if CLI process exits).
- **FR-2**: skip notification if source already focused (`is_hwnd_focused`)
- **FR-3**: auto-close on focus return via `SetWinEventHook(EVENT_SYSTEM_FOREGROUND)` + mpsc channel
- **Window activation**: uses `SendInput` Alt-key simulation to bypass `SetForegroundWindow` restriction; restores minimized windows via `IsIconic` check


### Thread Model

- Main thread: Tauri runtime + GUI event loop
- Pipe server thread: infinite loop accepting Named Pipe connections
- Foreground listener thread: `SetWinEventHook` message loop → mpsc → foreground change handler

## Frontend (src/)

Vue 3 + TypeScript + Composition API. UI components use shadcn-vue (`src/components/ui/`).

| File                             | Purpose                                                                        |
| -------------------------------- | ------------------------------------------------------------------------------ |
| `App.vue`                        | Notification window UI with event-type color coding, auto-dismiss progress bar |
| `Setup.vue`                      | Settings window with tab navigation (general / hooks)                          |
| `components/GeneralSettings.vue` | Position, auto-dismiss, sound settings                                         |
| `components/HookSettings.vue`    | Per-event hook enable/message config for 15 Claude Code hook events            |
| `components/AboutSettings.vue`   | About tab with version info and links                                          |
| `i18n.ts`                        | Internationalization (Korean/English) — reactive locale switching              |
| `types.ts`                       | Shared TypeScript interfaces (`HookConfig`, `NotificationData`, etc.)          |

## CLI Usage

```bash
agent-toast.exe --pid 1234 --event task_complete --message "Build done"
agent-toast.exe --daemon          # Run in background without notification
agent-toast.exe --setup           # Open settings window
agent-toast.exe --codex           # Codex CLI integration notification
```

Events: `task_complete`, `user_input_required`, `error`

`CLAUDE_PROJECT_DIR` env var is used as `title_hint` for window matching when `--title` is not provided.

## Configuration Files

### App Config (src-tauri/)

- `tauri.conf.json`: Tauri app settings (window size, permissions, build config)
- `capabilities/default.json`: Default Tauri permission settings

### User Settings

- `~/.claude/settings.json`: Claude Code hook settings (read/written by setup.rs, preserves non-agent-toast hooks)
- `~/.codex/config.toml`: Codex CLI notification hook settings
