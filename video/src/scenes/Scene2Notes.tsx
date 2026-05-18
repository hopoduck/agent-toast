import { AbsoluteFill, useCurrentFrame, interpolate } from "remotion";
import { FakeTerminal } from "../components/FakeTerminal";
import { FakeNoteApp } from "../components/FakeNoteApp";

const BODY_FULL = "이번 주에 정리할 내용\n\n- 리팩토링 결과 검토\n- 다음 스프린트 계획\n- 회의 일정 조율";
const BODY_TYPING_START = 60;
const BODY_TYPING_END = 400;

export const Scene2Notes: React.FC = () => {
  const frame = useCurrentFrame();

  const typedCount = Math.floor(
    interpolate(frame, [BODY_TYPING_START, BODY_TYPING_END], [0, BODY_FULL.length], {
      extrapolateLeft: "clamp",
      extrapolateRight: "clamp",
    }),
  );
  const typedBody = BODY_FULL.slice(0, typedCount);

  const noteAppOpacity = interpolate(frame, [0, 48], [0, 1], { extrapolateRight: "clamp" });
  const noteAppTranslate = interpolate(frame, [0, 48], [20, 0], { extrapolateRight: "clamp" });

  return (
    <AbsoluteFill style={{ background: "#0f0f10" }}>
      {/* Dimmed terminal in background */}
      <AbsoluteFill style={{ alignItems: "center", justifyContent: "center" }}>
        <FakeTerminal
          title="claude"
          lines={[
            { prefix: "❯", text: "ChatPanel 컴포넌트 리팩터링좀 해줘." },
            { text: "현황 파악 중...", color: "#a3a3a3" },
            { text: "● Explore(ChatPanel 컴포넌트 구조 탐색)" },
            { text: "ChatInput.tsx 분할 중... (809줄 → 312줄)", color: "#a3a3a3" },
            { text: "FileUploadSection.tsx 추출 중...", color: "#a3a3a3" },
            { text: "AttachmentPreview.tsx 추출 중...", color: "#a3a3a3" },
            { text: "useChatInputState 훅 생성 중...", color: "#a3a3a3" },
          ]}
          dimmed
        />
      </AbsoluteFill>

      {/* Note app on top */}
      <AbsoluteFill
        style={{
          alignItems: "center",
          justifyContent: "center",
          opacity: noteAppOpacity,
          transform: `translateY(${noteAppTranslate}px)`,
        }}
      >
        <FakeNoteApp
          notes={[
            { title: "주간 정리", active: true },
            { title: "회의록 — 03/12" },
            { title: "아이디어 노트" },
            { title: "독서 메모" },
          ]}
          title="주간 정리"
          body={typedBody}
          caretVisible={frame > BODY_TYPING_START && frame < BODY_TYPING_END + 120}
        />
      </AbsoluteFill>
    </AbsoluteFill>
  );
};
