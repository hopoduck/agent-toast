// 인트로 영상 다국어 문자열. ko = 기존(원문), en = 영문 버전.
// 씬은 locale prop 으로 STRINGS[locale] 을 읽어 렌더한다.

export type Locale = "ko" | "en";

/** Scene4 리포트 한 줄. color 는 로케일 무관(구조) — 두 로케일 모두 같은 색을 유지. */
export interface ReportLine {
  text: string;
  color?: string;
}

export interface IntroStrings {
  /** Scene1 캡션 */
  cap1: string;
  /** Scene2 캡션 (ko 는 \n 으로 2줄) */
  cap2: string;
  /** Scene3 캡션 */
  cap3: string;
  /** 엔딩 태그라인 */
  tagline: string;
  /** 터미널 사용자 입력 (Scene1 & Scene4 공용) */
  userInput: string;
  /** 터미널 응답 첫 줄 (Scene1 & Scene4 공용) */
  response: string;
  /** Explore 툴콜 줄 (Scene1 & Scene4 공용) */
  exploreCall: string;
  /** 노트앱 사이드바 라벨 */
  notesLabel: string;
  /** 노트앱 목록 제목 (첫 항목이 active) */
  notes: string[];
  /** 노트앱 본문 제목 */
  noteTitle: string;
  /** 노트앱 본문 (Scene2 타이핑 / Scene3 완성본) */
  noteBody: string;
  /** 토스트 이벤트 라벨 */
  toastLabel: string;
  /** 토스트 큰 제목(window_title) */
  toastTitle: string;
  /** 토스트 메시지 */
  toastMessage: string;
  /** 토스트 "보기" 버튼 */
  toastView: string;
  /** 토스트 "닫기" 버튼 */
  toastDismiss: string;
  /** Scene4 리팩터링 리포트 (파일명/줄수/식별자는 원문 유지, 산문만 번역) */
  report: ReportLine[];
}

const ko: IntroStrings = {
  cap1: "맡긴다",
  cap2: "다른 일을\n한다",
  cap3: "알린다",
  tagline: "터미널 감시는 이제 그만",
  userInput: "ChatPanel 컴포넌트 리팩터링좀 해줘.",
  response:
    "ChatPanel 컴포넌트 현황 파악부터 하겠습니다. Explore 에이전트로 구조랑 의존성 쫙 훑어볼게요.",
  exploreCall: "● Explore(ChatPanel 컴포넌트 구조 탐색)",
  notesLabel: "📝 메모",
  notes: ["주간 정리", "회의록 — 03/12", "아이디어 노트", "독서 메모"],
  noteTitle: "주간 정리",
  noteBody:
    "이번 주에 정리할 내용\n\n- 리팩토링 결과 검토\n- 다음 스프린트 계획\n- 회의 일정 조율",
  toastLabel: "작업 완료",
  toastTitle: "Claude Code",
  toastMessage: "작업이 완료되었습니다",
  toastView: "보기",
  toastDismiss: "닫기",
  report: [
    {
      text: "ChatPanel 컴포넌트 현황 파악부터 하겠습니다. Explore 에이전트로 구조랑 의존성 쫙 훑어볼게요.",
      color: "#a3a3a3",
    },
    { text: "" },
    { text: "● Explore(ChatPanel 컴포넌트 구조 탐색)" },
    { text: "" },
    { text: "ChatPanel 리팩터링 작업 완료", color: "#4ade80" },
    { text: "" },
    {
      text: "요청하신 ChatPanel 컴포넌트 리팩터링을 마쳤습니다. 작업 내역 정리하면:",
      color: "#a3a3a3",
    },
    { text: "" },
    { text: "📁 파일 구조 변경", color: "#fbbf24" },
    { text: "" },
    { text: "신규 추가 (5개)", color: "#a3a3a3" },
    { text: "src/components/ChatPanel/", color: "#a3a3a3" },
    { text: "├── ChatInput/", color: "#a3a3a3" },
    { text: "│   ├── index.tsx                  (312줄, 기존 809줄에서 분리)", color: "#a3a3a3" },
    { text: "│   ├── FileUploadSection.tsx      (148줄)", color: "#a3a3a3" },
    { text: "│   ├── AttachmentPreview.tsx      (94줄)", color: "#a3a3a3" },
    { text: "│   └── useChatInputState.ts       (87줄, 7개 useState 통합)", color: "#a3a3a3" },
    { text: "├── shared/", color: "#a3a3a3" },
    { text: "│   └── AutocompleteDropdown.tsx   (132줄, Macro/Mention 통합)", color: "#a3a3a3" },
    { text: "└── MessageBubble/", color: "#a3a3a3" },
    { text: "    ├── index.tsx                  (218줄, 기존 414줄에서 분리)", color: "#a3a3a3" },
    { text: "    └── AttachmentItem.tsx         (96줄)", color: "#a3a3a3" },
    { text: "" },
    { text: "삭제", color: "#a3a3a3" },
    { text: "- MacroList.tsx, MentionList.tsx → AutocompleteDropdown으로 통합", color: "#f87171" },
    { text: "" },
    { text: "🔧 주요 변경사항", color: "#fbbf24" },
    { text: "" },
    { text: "1. ChatInput 분할 (809줄 → 312줄, 약 61% 감소)", color: "#a3a3a3" },
    { text: "- 파일 업로드 + 진행률 UI → FileUploadSection으로 추출", color: "#a3a3a3" },
    { text: "- 첨부 파일/링크 미리보기 → AttachmentPreview로 추출", color: "#a3a3a3" },
    { text: "- 7개 useState를 useChatInputState 커스텀 훅으로 캡슐화", color: "#a3a3a3" },
    { text: "" },
    { text: "2. Prop drilling 해소 (21개 → 6개)", color: "#a3a3a3" },
    {
      text: "- 도메인별로 props 객체화: inputState, attachments, handlers, permissions, conversation, variant",
      color: "#a3a3a3",
    },
    { text: "- setter 함수들은 handlers 객체로 묶어서 전달", color: "#a3a3a3" },
    { text: "" },
    { text: "3. AutocompleteDropdown 제네릭화", color: "#a3a3a3" },
    { text: "- MacroList/MentionList 중복 제거", color: "#a3a3a3" },
    { text: "- 키보드 네비게이션, 외부 클릭 감지 로직 통합", color: "#a3a3a3" },
    { text: "- 숫자 단축키(1~9) 처리도 공통화", color: "#a3a3a3" },
    { text: "" },
    { text: "4. 유틸 정리", color: "#a3a3a3" },
    {
      text: "- formatTimeOnly, formatMessageTimestamp → src/utils/dateFormat.ts로 이동 및 통합",
      color: "#a3a3a3",
    },
    { text: "- 4곳에서 사용하던 중복 코드 제거", color: "#a3a3a3" },
    { text: "" },
    { text: "✅ 검증 결과", color: "#fbbf24" },
    { text: "" },
    { text: "pnpm typecheck   ✓ (에러 없음)", color: "#4ade80" },
    { text: "pnpm lint        ✓ (warning 0)", color: "#4ade80" },
    { text: "pnpm test        ✓ (47 passed, 0 failed)", color: "#4ade80" },
    { text: "" },
    {
      text: "기존 동작은 모두 동일하게 유지하면서 코드 가독성과 유지보수성이 크게 개선되었습니다. ChatInput을 수정할 때 800줄짜리 파일을 헤집을 필요 없이, 관심사별로 분리된 작은 파일만 건드리면 되도록 작업 완료하였습니다.",
      color: "#a3a3a3",
    },
  ],
};

const en: IntroStrings = {
  cap1: "Delegate.",
  cap2: "Step away.",
  cap3: "Get pinged.",
  tagline: "No more babysitting the terminal.",
  userInput: "Refactor the ChatPanel component for me.",
  response:
    "Let me start by understanding the current ChatPanel component. I'll sweep through its structure and dependencies with the Explore agent.",
  exploreCall: "● Explore(Scanning ChatPanel component structure)",
  notesLabel: "📝 Notes",
  notes: ["Weekly notes", "Meeting — 03/12", "Ideas", "Reading log"],
  noteTitle: "Weekly notes",
  noteBody:
    "Things to wrap up this week\n\n- Review the refactoring results\n- Plan the next sprint\n- Coordinate meeting schedules",
  toastLabel: "Task complete",
  toastTitle: "Claude Code",
  toastMessage: "Your task is complete.",
  toastView: "View",
  toastDismiss: "Dismiss",
  report: [
    {
      text: "Let me start by understanding the current ChatPanel component. I'll sweep through its structure and dependencies with the Explore agent.",
      color: "#a3a3a3",
    },
    { text: "" },
    { text: "● Explore(Scanning ChatPanel component structure)" },
    { text: "" },
    { text: "ChatPanel refactoring complete", color: "#4ade80" },
    { text: "" },
    {
      text: "I've finished refactoring the ChatPanel component as requested. Here's a summary:",
      color: "#a3a3a3",
    },
    { text: "" },
    { text: "📁 File structure changes", color: "#fbbf24" },
    { text: "" },
    { text: "Added (5 files)", color: "#a3a3a3" },
    { text: "src/components/ChatPanel/", color: "#a3a3a3" },
    { text: "├── ChatInput/", color: "#a3a3a3" },
    { text: "│   ├── index.tsx                  (312 lines, split from 809)", color: "#a3a3a3" },
    { text: "│   ├── FileUploadSection.tsx      (148 lines)", color: "#a3a3a3" },
    { text: "│   ├── AttachmentPreview.tsx      (94 lines)", color: "#a3a3a3" },
    { text: "│   └── useChatInputState.ts       (87 lines, 7 useStates merged)", color: "#a3a3a3" },
    { text: "├── shared/", color: "#a3a3a3" },
    { text: "│   └── AutocompleteDropdown.tsx   (132 lines, Macro/Mention merged)", color: "#a3a3a3" },
    { text: "└── MessageBubble/", color: "#a3a3a3" },
    { text: "    ├── index.tsx                  (218 lines, split from 414)", color: "#a3a3a3" },
    { text: "    └── AttachmentItem.tsx         (96 lines)", color: "#a3a3a3" },
    { text: "" },
    { text: "Removed", color: "#a3a3a3" },
    { text: "- MacroList.tsx, MentionList.tsx → merged into AutocompleteDropdown", color: "#f87171" },
    { text: "" },
    { text: "🔧 Key changes", color: "#fbbf24" },
    { text: "" },
    { text: "1. Split ChatInput (809 → 312 lines, ~61% smaller)", color: "#a3a3a3" },
    { text: "- File upload + progress UI → extracted to FileUploadSection", color: "#a3a3a3" },
    { text: "- Attachment/link previews → extracted to AttachmentPreview", color: "#a3a3a3" },
    { text: "- Encapsulated 7 useStates into the useChatInputState custom hook", color: "#a3a3a3" },
    { text: "" },
    { text: "2. Eliminated prop drilling (21 → 6 props)", color: "#a3a3a3" },
    {
      text: "- Grouped props by domain: inputState, attachments, handlers, permissions, conversation, variant",
      color: "#a3a3a3",
    },
    { text: "- Bundled setter functions into a handlers object", color: "#a3a3a3" },
    { text: "" },
    { text: "3. Made AutocompleteDropdown generic", color: "#a3a3a3" },
    { text: "- Removed MacroList/MentionList duplication", color: "#a3a3a3" },
    { text: "- Unified keyboard navigation and outside-click detection", color: "#a3a3a3" },
    { text: "- Shared the number-shortcut (1–9) handling too", color: "#a3a3a3" },
    { text: "" },
    { text: "4. Tidied utilities", color: "#a3a3a3" },
    {
      text: "- formatTimeOnly, formatMessageTimestamp → moved & merged into src/utils/dateFormat.ts",
      color: "#a3a3a3",
    },
    { text: "- Removed duplicate code used in 4 places", color: "#a3a3a3" },
    { text: "" },
    { text: "✅ Verification", color: "#fbbf24" },
    { text: "" },
    { text: "pnpm typecheck   ✓ (no errors)", color: "#4ade80" },
    { text: "pnpm lint        ✓ (0 warnings)", color: "#4ade80" },
    { text: "pnpm test        ✓ (47 passed, 0 failed)", color: "#4ade80" },
    { text: "" },
    {
      text: "Behavior is fully preserved while readability and maintainability improved significantly. Editing ChatInput no longer means digging through an 800-line file — you now touch only the small, concern-separated files.",
      color: "#a3a3a3",
    },
  ],
};

export const STRINGS: Record<Locale, IntroStrings> = { ko, en };
