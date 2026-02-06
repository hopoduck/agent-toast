<p align="center">
  <strong>한국어</strong> | <a href="README.en.md">English</a>
</p>

<p align="center">
  <img src="public/logo.png" width="120" alt="Agent Toast Logo">
</p>

<h1 align="center">Agent Toast</h1>

<p align="center">
  <strong>Windows용 스마트 데스크톱 알림 앱</strong><br>
  AI 코딩 어시스턴트의 이벤트를 놓치지 마세요
</p>

<p align="center">
  <a href="https://github.com/hopoduck/agent-toast/releases"><img src="https://img.shields.io/github/v/release/hopoduck/agent-toast?style=flat-square" alt="Release"></a>
  <a href="https://github.com/hopoduck/agent-toast/blob/master/LICENSE"><img src="https://img.shields.io/github/license/hopoduck/agent-toast?style=flat-square" alt="License"></a>
  <img src="https://img.shields.io/badge/platform-Windows-blue?style=flat-square" alt="Platform">
</p>

---

## ✨ 주요 기능

- **이벤트 알림** - 작업 완료, 사용자 입력 필요, 오류 발생 등
- **자동 포커스 이동** - 알림 클릭 시 터미널 창으로 자동 전환
- **포커스 시 자동 닫힘** - 터미널 창에 포커스하면 알림 자동 닫힘
- **멀티 모니터 지원** - 원하는 모니터에 알림 표시
- **다국어 지원** - 한국어/영어 UI
- **자동 업데이트** - 새 버전 알림 및 원클릭 업데이트

## 📸 스크린샷

<p align="center">
  <img src=".github/screenshots/notifications.png" width="400" alt="알림 예시">
</p>

## 🔌 지원 플랫폼

| 플랫폼                                               | 설명                               |
| ---------------------------------------------------- | ---------------------------------- |
| [Claude Code](https://www.anthropic.com/claude-code) | Anthropic의 AI 코딩 어시스턴트     |
| [Codex CLI](https://openai.com/codex/)               | OpenAI의 터미널 기반 코딩 에이전트 |

## 📥 설치

### Releases에서 다운로드

[**📦 최신 버전 다운로드**](https://github.com/hopoduck/agent-toast/releases/latest)

### 직접 빌드

```bash
# 요구사항: Node.js 18+, pnpm, Rust (MSVC 툴체인)

pnpm install
pnpm tauri build
```

## 🚀 사용법

### 1. 설정 창 열기

```bash
agent-toast.exe --setup
```

또는 시스템 트레이 아이콘 우클릭 → 설정

### 2. 훅 설정

설정 창에서 원하는 이벤트를 활성화하면 자동으로 훅이 등록됩니다.

| 플랫폼      | 설정 파일                 |
| ----------- | ------------------------- |
| Claude Code | `~/.claude/settings.json` |
| Codex CLI   | `~/.codex/config.toml`    |

## ⚙️ 작동 원리

Agent Toast는 백그라운드에서 Named Pipe 서버로 실행됩니다. Claude Code나 Codex CLI에서 이벤트가 발생하면 훅이 실행되어 Agent Toast에 알림을 전송합니다. 알림을 클릭하면 프로세스 트리를 추적하여 원래 터미널 창을 찾아 포커스를 이동시킵니다.

## 🛠️ 기술 스택

<p>
  <img src="https://img.shields.io/badge/Rust-000000?style=flat-square&logo=rust&logoColor=white" alt="Rust">
  <img src="https://img.shields.io/badge/Tauri-24C8D8?style=flat-square&logo=tauri&logoColor=white" alt="Tauri">
  <img src="https://img.shields.io/badge/Vue.js-4FC08D?style=flat-square&logo=vue.js&logoColor=white" alt="Vue.js">
  <img src="https://img.shields.io/badge/TypeScript-3178C6?style=flat-square&logo=typescript&logoColor=white" alt="TypeScript">
</p>

## 📄 라이선스

[MIT License](LICENSE)
