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
    src/wire.rs                     # NotifyRequest (+ hostname, alt_title_hint), WIRE_VERSION
    src/hook_config.rs              # merge_agent_toast_hooks, HookEntry, is_agent_toast_cmd
    src/ide.rs                      # read_ide_project_name: JetBrains `.idea/.name` lookup
    src/dynamic.rs                  # --dynamic: derive toast body from hook stdin JSON (tool_input.description → last_assistant_message → static --message)
  agent-toast-desktop/              # Windows-only Tauri app (was src-tauri/)
    src/main.rs, lib.rs, cli.rs, pipe.rs, http_server.rs,
    notification.rs, win32.rs, setup.rs, sound.rs, updater.rs,
    changelog.rs, fonts.rs, orca.rs
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
| `sound.rs`        | Notification sound — custom file via WinRT `MediaPlayer` (fallback to system sound), default via `PlaySoundW`; preview play/stop |
| `updater.rs`      | Auto-update check via GitHub API (`CHECK_INTERVAL_MINUTES = 60`, i.e. hourly; 24h snooze), update notification with snooze/dedupe/sticky |
| `changelog.rs`    | Extracts changelog from release body markers; `ReleaseInfo` payload sent to frontend                |
| `fonts.rs`        | Enumerates installed system fonts via GDI `EnumFontFamiliesExW` for the toast font picker           |
| `orca.rs`         | Orca integration: app PID and tab switching over the runtime's named pipe (`orca-runtime.json`), plus the on-screen tab read from its persisted state (`profiles/<active profile>/orca-data.json`) |
| `watchdog.rs`     | Main-thread stall detection and recovery — heartbeat ACK, per-thread breadcrumbs, health file, `WM_QUIT` unstick, restart escalation |

### Critical Win32 Logic

- **Process tree**: walks parent chain from `--pid` up to 20 levels to find the terminal window. Tree is resolved eagerly in `main.rs` before pipe send (avoids race if CLI process exits).
- **Window title matching**: `select_source_window` scores every candidate against each hint in `title_hints` (folder name, then the JetBrains project name) and keeps the best; only when nothing matches does it fall back to process-tree distance and z-order. `score_title_match` normalizes en/em dash separators first, since JetBrains titles read `api – Foo.java [api]` while VS Code uses `Foo.ts - folder - Visual Studio Code`. Every frame of one JetBrains IDE shares a single PID, so the title is the only thing that can tell those windows apart.
- **FR-2**: skip notification if source already focused (`is_hwnd_focused`)
- **FR-3**: auto-close on focus return via `SetWinEventHook(EVENT_SYSTEM_FOREGROUND)` + mpsc channel
- **Window activation**: uses `SendInput` Alt-key simulation to bypass `SetForegroundWindow` restriction; restores minimized windows via `IsIconic` check
- **Orca sessions bypass all of the above**: the shell's parent is a detached `orca-terminal-daemon.exe`, so the process tree reaches no window at all, and every session is a tab in one window titled `Orca`. When `ORCA_TERMINAL_HANDLE` is present (`orca.rs`), the window comes from the runtime's own app PID, and the tab drives the per-tab decisions: skip and auto-close only when Orca's recorded `activeTabId` equals this session's `ORCA_TAB_ID`, and switch to the session on click via `terminal.focus`
- **The on-screen tab is not on the runtime pipe**: `terminal.resolveActive` reads like the method for it, but without a worktree argument it walks the tab map and returns the first entry no matter what the UI shows. The runtime reads the real value from its own store and exposes it only to plugins, and the RPC envelope permits exactly three frames (success, failure, keepalive), so nothing can be subscribed to either. `orca.rs` therefore reads `activeTabId` out of Orca's persisted state, re-parsing that file only when its mtime or length moves

### Thread Model

- Main thread: Tauri runtime + GUI event loop
- Pipe server thread: infinite loop accepting Named Pipe connections
- Foreground listener thread: `SetWinEventHook` message loop → mpsc → foreground change handler
- Orca tab watcher thread: polls Orca's recorded active tab every 400ms while Orca toasts are on screen. Switching tabs inside Orca fires no OS focus event, so it is the only way to notice the user returning to a session
- Watchdog thread: asks the main thread for an ACK every 5s and never calls anything that needs it

### Main-Thread Stall and Recovery (`watchdog.rs`)

`wry` waits for WebView2 controller creation in `webview2_com::wait_with_pump`, an unbounded `GetMessage`/`DispatchMessage` loop with no timeout. When that callback never arrives the main thread stays inside it forever: Win32 messages keep being dispatched, so the process still answers `WM_NULL` and looks alive to Windows, but tao's user-event queue is never drained. Window creation, show, destroy, reposition and tray menu commands all pile up unexecuted. Observed live on 2026-09-09: 15 toasts logged as created over two hours with no window ever appearing.

- **Detection**: a worker thread posts `run_on_main_thread` ACKs every 5s. Nothing else can distinguish this state, because the main thread is pumping messages the whole time.
- **Guard**: toast windows are built inside `run_on_main_thread` so the build is bracketed by main-thread breadcrumbs. Recovery only fires `WM_QUIT` when the main breadcrumb is `notify-build-window` or `setup-build-window`. Posting it while the main thread is merely slow would reach tao's own loop and quit the app.
- **Unstick**: `PostThreadMessageW(main_tid, WM_QUIT)` makes the nested `GetMessage` return 0, so `wry` fails that one window with `TaskCanceled` and the event loop resumes. Verified live.
- **Escalation**: after `MAX_UNSTICKS` within `UNSTICK_RESET`, or when the stall is not in a webview build, the watchdog thread restarts the process itself (`current_exe` + `exit`), since Tauri's own `restart()` routes through the dead main thread. The replacement instance is marked via `AGENT_TOAST_RECOVERED_MS` and shows one toast explaining the gap.
- **Singleton wait**: the replacement starts while the old process is still exiting, so `main.rs` retries the mutex for `SINGLETON_WAIT` instead of giving up immediately. Without it a slow shutdown leaves nothing running.

## Frontend (src/)

Vue 3 + TypeScript + Composition API. UI components use shadcn-vue (`src/components/ui/`).

**Styling: Tailwind CSS v4 (utility-first) + shadcn-vue.** Compose with utility classes — avoid hand-written `<style>`/raw CSS. Use arbitrary-value utilities when needed (`bg-[radial-gradient(...)]`, `[stop-color:var(--chart-1)]`), `tw-animate-css` for entrance/motion (`animate-in fade-in slide-in-from-bottom-2`, `motion-reduce:animate-none`), and the existing design tokens (CSS vars in `src/global.css`: shadcn tokens, `--event-*` event colors shared with the toast, `--chart-*`, `--font-mono` = D2Coding). Every screen must work in both light and dark themes (toggle lives in `Setup.vue`).

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

`ORCA_TERMINAL_HANDLE` and `ORCA_TAB_ID` env vars (exported by Orca into every managed terminal) are forwarded as `orca_terminal_handle` and `orca_tab_id`. The handle identifies the terminal and is what a toast click hands back to `terminal.focus`; the tab id is what skip and auto-close compare against Orca's recorded active tab. Only the local Windows path sends them; `agent-toast-send` leaves both unset because a remote host can reach neither the desktop's Orca runtime nor its state.

`CLAUDE_PROJECT_DIR` env var is used as `title_hint` for window matching when `--title` is not provided. Its `.idea/.name` (the JetBrains project name, present when it differs from the folder name) is sent as `alt_title_hint`, a matching-only hint — the toast still displays `title_hint`.

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
