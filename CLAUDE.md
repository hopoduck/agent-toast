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
pnpm tauri build            # Production build → target/release/agent-toast.exe (workspace root)
pnpm build                  # vue-tsc --noEmit (type check) + vite build (frontend only)
pnpm release                # Release build with updater artifacts (requires TAURI_SIGNING_PRIVATE_KEY in .env)
```

### Lint & Format (workspace root)

```bash
# Rust (workspace)
cargo fmt --check --all                                    # Format check
cargo clippy --workspace --all-targets -- -D warnings      # Lint check (CI enforces -D warnings)
cargo test --workspace                                     # Run all tests
cargo test -p <crate> <test_name>                          # Run a specific test in a specific crate

# TypeScript
pnpm vue-tsc --noEmit                                      # Type check only
```

### CI Checks (GitHub Actions)

On push/PR to `master`, the `check.yml` workflow runs on `windows-latest`: `cargo fmt --check --all`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace`, `pnpm vue-tsc --noEmit`. A second `check-send-linux` job on `ubuntu-latest` verifies `agent-toast-send` builds for `x86_64-unknown-linux-musl` and runs the `agent-toast-send` / `agent-toast-core` tests on Linux. All must pass before merge.

`release.yml` runs on `v*` tag push (or manual `workflow_dispatch`): builds the signed updater artifacts and publishes a GitHub Release. The release body's changelog is delimited by `<!-- changelog:start -->` / `<!-- changelog:end -->` markers, which the in-app updater (`changelog.rs`) extracts to show "what's new".

## Architecture

**Single-instance Tauri v2 desktop app** for Windows that shows smart notifications for Claude Code events.

### Process Model

- First invocation (`--pid --event`): tries Named Pipe → if no listener, launches Tauri app
- Subsequent invocations: connects to pipe `\\.\pipe\agent-toast`, sends JSON, exits immediately
- No args or `--daemon`: starts app without initial notification
- Singleton enforced via `CreateMutexW("agent-toast-singleton")`
- **Dev/prod coexist**: debug builds (`cfg(debug_assertions)`) use `-dev`-suffixed names — mutex `agent-toast-singleton-dev` and pipe `\\.\pipe\agent-toast-dev` — so a dev instance runs alongside an installed production one. Dev builds also show a "DEV" badge on the taskbar/tray icon, title, and tooltip.

### Backend ↔ Frontend IPC

- **Rust → Frontend**: emits `notification-data` event with `NotificationData` struct
- **Frontend → Rust**: `invoke("close_notify", { id })`, `invoke("activate_source", { hwnd, id })`
- **Initial load**: Frontend calls `invoke("get_notification_data")` on mount (event may arrive before listener is ready)

### Window Routing

Single `index.html` + `main.ts` serves both notification and setup windows. Window label (`setup` vs `notify-*`) determines which Vue component mounts — no router needed.

## Cargo Workspace Layout

```
Cargo.toml                          # workspace root
crates/
  agent-toast-core/                 # shared types + hook-config JSON merge
    src/lib.rs
    src/wire.rs                     # NotifyRequest (+ hostname), WIRE_VERSION
    src/hook_config.rs              # merge_agent_toast_hooks, HookEntry, is_agent_toast_cmd
    src/dynamic.rs                  # --dynamic: derive toast body from hook stdin JSON (tool_input.description → last_assistant_message → static --message)
  agent-toast-desktop/              # Windows-only Tauri app (was src-tauri/)
    src/main.rs, lib.rs, cli.rs, pipe.rs, http_server.rs,
    notification.rs, win32.rs, setup.rs, sound.rs, updater.rs,
    changelog.rs, fonts.rs
    tauri.conf.json, tauri.release.conf.json, icons/, capabilities/
  agent-toast-send/                 # cross-platform CLI for remote Linux servers
    src/main.rs                     # send / init / uninstall subcommands
    tests/send_integration.rs, tests/init_integration.rs
src/                                # Vue 3 + TypeScript frontend (unchanged)
```

### Rust Backend Modules (`crates/agent-toast-desktop/src/`)

| Module            | Purpose                                                                                             |
| ----------------- | --------------------------------------------------------------------------------------------------- |
| `main.rs`         | CLI entry, single-instance routing via pipe, parent PID auto-detection                              |
| `lib.rs`          | Tauri app setup, command registration, tray icon, HTTP server wiring                                |
| `cli.rs`          | clap arg parsing (re-exports `NotifyRequest` from `agent-toast-core`)                               |
| `pipe.rs`         | Named Pipe server/client (Windows-only, stubs for other OS) — local transport                       |
| `http_server.rs`  | `tiny_http` receiver for remote notifications — runs when `http_enabled=true`                       |
| `notification.rs` | Notification lifecycle, window creation, 4-corner positioning with DPI scaling, `RateLimiter` (10/s burst 10) |
| `win32.rs`        | Process tree walking, focus detection, window activation (Windows-only)                             |
| `setup.rs`        | Settings file I/O (`~/.claude/settings.json`), hook config builder (delegates JSON merge to core)   |
| `sound.rs`        | System notification sound via `PlaySoundW`                                                          |
| `updater.rs`      | Auto-update check via GitHub API (`CHECK_INTERVAL_MINUTES = 60`, i.e. hourly; 24h snooze), update notification with snooze/dedupe/sticky |
| `changelog.rs`    | Extracts changelog from release body markers; `ReleaseInfo` payload sent to frontend                |
| `fonts.rs`        | Enumerates installed system fonts via GDI `EnumFontFamiliesExW` for the toast font picker           |

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
| `App.vue`                        | Notification window shell — auto-dismiss progress bar, dynamic window height, light/dark theme; delegates card rendering to `ToastCard.vue` |
| `components/ToastCard.vue`       | The toast card itself — event-type color coding, inline markdown body (markdown-it, escaped), Claude/OpenAI logo, applies `ToastStyle` (font + design) |
| `Setup.vue`                      | Settings window with tab navigation (general / hooks / remote / design / howto / about) |
| `components/GeneralSettings.vue` | Position, auto-dismiss, sound settings                                         |
| `components/HookSettings.vue`    | Per-event hook enable/message config for 15 Claude Code hook events            |
| `components/RemoteSettings.vue`  | Remote HTTP receiver settings + `agent-toast-send` setup guide                 |
| `components/DesignSettings.vue`  | Toast appearance — sans/mono system-font picker (`ToastStyle`), bundled D2Coding, live preview |
| `components/HowtoSettings.vue`   | Usage guide tab                                                                |
| `components/AboutSettings.vue`   | About tab with version info and links                                          |
| `components/SlidingTabs.vue`     | Animated tab switcher used by `Setup.vue`                                      |
| `components/CodeBlock.vue`       | Syntax-highlighted code block used in the howto/setup guides                   |
| `i18n.ts`                        | vue-i18n setup — locale strings live in `src/locales/{ko,en}.json`             |
| `types.ts`                       | Shared TypeScript interfaces (`HookConfig`, `NotificationData`, `ToastStyle`, etc.) |

## CLI Usage

```bash
agent-toast.exe --pid 1234 --event task_complete --message "Build done"
agent-toast.exe --daemon          # Run in background without notification
agent-toast.exe --setup           # Open settings window
agent-toast.exe --codex           # Codex CLI integration notification
agent-toast.exe --pid 1234 --event task_complete --dynamic   # Derive body from hook stdin JSON (falls back to --message)
```

Events: `task_complete`, `user_input_required`, `error`

`CLAUDE_PROJECT_DIR` env var is used as `title_hint` for window matching when `--title` is not provided.

## Configuration Files

### App Config (`crates/agent-toast-desktop/`)

- `tauri.conf.json`: Tauri app settings (window size, permissions, build config)
- `tauri.release.conf.json`: Release-only overrides (enables `createUpdaterArtifacts`)
- `capabilities/default.json`: Default Tauri permission settings

### User Settings

- `~/.claude/settings.json`: Claude Code hook settings (read/written by setup.rs, preserves non-agent-toast hooks)
- `~/.codex/config.toml`: Codex CLI notification hook settings

## Intro Video (`video/`)

Separate Remotion project (own `package.json`, not part of the pnpm/Cargo workspaces) that renders the README intro video (`.github/media/intro.webp`). See `video/CLAUDE.md` for render commands, fps-relative timing rules, and the mp4 → animated webp conversion guide.
