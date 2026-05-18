import React from "react";
import { AbsoluteFill, useCurrentFrame, interpolate, spring, useVideoConfig, Easing } from "remotion";
import { FakeTerminal } from "../components/FakeTerminal";
import { FakeNoteApp } from "../components/FakeNoteApp";
import { FakeToast } from "../components/FakeToast";
import { FakeCursor } from "../components/FakeCursor";

const FULL_BODY = "이번 주에 정리할 내용\n\n- 리팩토링 결과 검토\n- 다음 스프린트 계획\n- 회의 일정 조율";

export const Scene3Notification: React.FC = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  // Toast slide-in (frames 0-15)
  const toastSpring = spring({
    frame,
    fps,
    config: { damping: 14, stiffness: 110, mass: 0.6 },
  });
  const toastX = interpolate(toastSpring, [0, 1], [400, 0]); // slide from right
  const toastOpacity = interpolate(toastSpring, [0, 1], [0, 1]);

  // Progress bar countdown
  const progress = interpolate(frame, [40, 360], [1, 0.45], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });

  // Cursor movement: starts off-screen at frame 25, reaches button at frame 65.
  // ease-out cubic 으로 실제 사람 마우스처럼 시작은 빠르고 끝에서 부드럽게 감속.
  const cursorEase = {
    extrapolateLeft: "clamp" as const,
    extrapolateRight: "clamp" as const,
    easing: Easing.out(Easing.cubic),
  };
  const cursorX = interpolate(frame, [100, 260], [600, 1010], cursorEase);
  const cursorY = interpolate(frame, [100, 260], [400, 645], cursorEase);

  const isHovering = frame >= 260;

  // 클릭은 Scene3 끝부분(85~89)에 배치 — 누른 순간(frame 90)이 곧 Scene4 전환 시점.
  // 회복 모션 없이 누르는 모션만(1 → 0.78) 두어 "클릭이 화면 전환을 일으킨다" 인상을 줌.
  const cursorScale = interpolate(
    frame,
    [336, 356],
    [1, 0.78],
    { extrapolateLeft: "clamp", extrapolateRight: "clamp" },
  );

  // Toast position: bottom-right
  const toastRight = 40;
  const toastBottom = 40;

  return (
    <AbsoluteFill style={{ background: "#0f0f10" }}>
      {/* Background — dimmed terminal */}
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
            { text: "MacroList/MentionList → AutocompleteDropdown 통합 중...", color: "#a3a3a3" },
            { text: "MessageBubble.tsx 분할 중...", color: "#a3a3a3" },
            { text: "pnpm test 실행 중...", color: "#a3a3a3" },
          ]}
          dimmed
        />
      </AbsoluteFill>

      {/* Note app — still visible but no caret */}
      <AbsoluteFill style={{ alignItems: "center", justifyContent: "center" }}>
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

      {/* Toast — bottom-right */}
      <div
        style={{
          position: "absolute",
          right: toastRight,
          bottom: toastBottom,
          transform: `translateX(${toastX}px)`,
          opacity: toastOpacity,
        }}
      >
        <FakeToast
          eventLabel="작업 완료"
          windowTitle="Claude Code"
          message="작업이 완료되었습니다"
          progress={progress}
          hovered={isHovering}
        />
      </div>

      {/* Cursor */}
      {frame >= 100 && <FakeCursor x={cursorX} y={cursorY} scale={cursorScale} />}
    </AbsoluteFill>
  );
};
