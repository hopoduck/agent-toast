import { AbsoluteFill, useCurrentFrame, interpolate, staticFile } from "remotion";
import { FakeTerminal, type TerminalLine } from "../components/FakeTerminal";
import { FakeNoteApp } from "../components/FakeNoteApp";

const FULL_BODY = "이번 주에 정리할 내용\n\n- 리팩토링 결과 검토\n- 다음 스프린트 계획\n- 회의 일정 조율";

const FULL_REPORT_LINES: { text: string; color?: string }[] = [
  // 응답 시작 (Scene1/2/3 와 연결되는 사전 작업 요약)
  { text: "ChatPanel 컴포넌트 현황 파악부터 하겠습니다. Explore 에이전트로 구조랑 의존성 쫙 훑어볼게요.", color: "#a3a3a3" },
  { text: "" },
  { text: "● Explore(ChatPanel 컴포넌트 구조 탐색)" },
  { text: "" },
  { text: "ChatPanel 리팩터링 작업 완료", color: "#4ade80" },
  { text: "" },
  { text: "요청하신 ChatPanel 컴포넌트 리팩터링을 마쳤습니다. 작업 내역 정리하면:", color: "#a3a3a3" },
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
  { text: "- 도메인별로 props 객체화: inputState, attachments, handlers, permissions, conversation, variant", color: "#a3a3a3" },
  { text: "- setter 함수들은 handlers 객체로 묶어서 전달", color: "#a3a3a3" },
  { text: "" },
  { text: "3. AutocompleteDropdown 제네릭화", color: "#a3a3a3" },
  { text: "- MacroList/MentionList 중복 제거", color: "#a3a3a3" },
  { text: "- 키보드 네비게이션, 외부 클릭 감지 로직 통합", color: "#a3a3a3" },
  { text: "- 숫자 단축키(1~9) 처리도 공통화", color: "#a3a3a3" },
  { text: "" },
  { text: "4. 유틸 정리", color: "#a3a3a3" },
  { text: "- formatTimeOnly, formatMessageTimestamp → src/utils/dateFormat.ts로 이동 및 통합", color: "#a3a3a3" },
  { text: "- 4곳에서 사용하던 중복 코드 제거", color: "#a3a3a3" },
  { text: "" },
  { text: "✅ 검증 결과", color: "#fbbf24" },
  { text: "" },
  { text: "pnpm typecheck   ✓ (에러 없음)", color: "#4ade80" },
  { text: "pnpm lint        ✓ (warning 0)", color: "#4ade80" },
  { text: "pnpm test        ✓ (47 passed, 0 failed)", color: "#4ade80" },
  { text: "" },
  { text: "기존 동작은 모두 동일하게 유지하면서 코드 가독성과 유지보수성이 크게 개선되었습니다. ChatInput을 수정할 때 800줄짜리 파일을 헤집을 필요 없이, 관심사별로 분리된 작은 파일만 건드리면 되도록 작업 완료하였습니다.", color: "#a3a3a3" },
];

// Base context lines that were already visible at the end of Scene3
const BASE_LINES: TerminalLine[] = [
  { prefix: "❯", text: "ChatPanel 컴포넌트 리팩터링좀 해줘." },
];

export const Scene4Return: React.FC = () => {
  const frame = useCurrentFrame();

  // Phase 1 (0-72): note app fades out, terminal un-dims
  const noteOpacity = interpolate(frame, [0, 72], [1, 0], { extrapolateRight: "clamp" });
  const noteScale = interpolate(frame, [0, 72], [1, 0.96], { extrapolateRight: "clamp" });
  const terminalDim = frame < 72;

  // Full report — shown immediately (no progressive reveal)
  const lines: TerminalLine[] = [...BASE_LINES, ...FULL_REPORT_LINES];

  // Phase 3: logo + tagline fade in, terminal fades out
  const endingOpacity = interpolate(frame, [320, 400], [0, 1], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });
  const endingScale = interpolate(frame, [320, 440], [0.95, 1], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });
  // 종료값 0 으로 — 잔상이 남으면 로고 위에 어두운 패치가 보임
  const terminalFadeOut = interpolate(frame, [280, 380], [1, 0], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });

  return (
    <AbsoluteFill style={{ background: "#0f0f10" }}>
      {/* Terminal — comes forward */}
      <AbsoluteFill style={{ alignItems: "center", justifyContent: "center", opacity: terminalFadeOut }}>
        <FakeTerminal title="claude" lines={lines} dimmed={terminalDim} />
      </AbsoluteFill>

      {/* Note app fading out */}
      {noteOpacity > 0.01 && (
        <AbsoluteFill style={{ alignItems: "center", justifyContent: "center", opacity: noteOpacity, transform: `scale(${noteScale})` }}>
          <FakeNoteApp
            notes={[
              { title: "주간 정리", active: true },
              { title: "회의록 — 03/12" },
              { title: "아이디어 노트" },
              { title: "독서 메모" },
            ]}
            title="주간 정리"
            body={FULL_BODY}
          />
        </AbsoluteFill>
      )}

      {/* Ending — logo + tagline */}
      <AbsoluteFill
        style={{
          alignItems: "center",
          justifyContent: "center",
          opacity: endingOpacity,
          transform: `scale(${endingScale})`,
        }}
      >
        <div style={{ display: "flex", flexDirection: "column", alignItems: "center", gap: 18 }}>
          {/* Simple wordmark — actual logo image not loaded here, use text */}
          <div
            style={{
              fontSize: 80,
              fontWeight: 800,
              color: "#ffffff",
              letterSpacing: -1,
              display: "flex",
              alignItems: "center",
              gap: 22,
            }}
          >
            <img
              src={staticFile("logo.png")}
              alt=""
              style={{ width: 90, height: 90, objectFit: "contain" }}
            />
            agent-toast
          </div>
          <div
            style={{
              fontSize: 28,
              color: "rgba(255,255,255,0.7)",
              letterSpacing: 0.5,
            }}
          >
            기다리지 마세요
          </div>
        </div>
      </AbsoluteFill>
    </AbsoluteFill>
  );
};
