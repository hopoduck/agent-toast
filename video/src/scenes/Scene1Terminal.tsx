import React from "react";
import { AbsoluteFill, useCurrentFrame, interpolate } from "remotion";
import { FakeTerminal, type TerminalLine } from "../components/FakeTerminal";

const USER_INPUT = "ChatPanel 컴포넌트 리팩터링좀 해줘.";
const TYPING_START = 40;   // frame
const TYPING_END = 280;    // frame — 240프레임 동안 21자, 자연스러운 타이핑 속도
const ENTER_FRAME = 300;
const RESPONSE_START = 320;

export const Scene1Terminal: React.FC = () => {
  const frame = useCurrentFrame();

  // Typed characters
  const typedCount = Math.floor(
    interpolate(frame, [TYPING_START, TYPING_END], [0, USER_INPUT.length], {
      extrapolateLeft: "clamp",
      extrapolateRight: "clamp",
    }),
  );
  const typedText = USER_INPUT.slice(0, typedCount);

  const showResponse = frame >= RESPONSE_START;

  const lines: TerminalLine[] = [
    { prefix: "❯", text: typedText },
  ];
  if (showResponse) {
    lines.push({
      text: "ChatPanel 컴포넌트 현황 파악부터 하겠습니다. Explore 에이전트로 구조랑 의존성 쫙 훑어볼게요.",
      color: "#a3a3a3",
    });
  }
  // Scene1 끝부분(~frame 380 이후)에 Explore 에이전트 호출 줄을 미리 넣어 Scene2 배경과 자연 전환
  if (frame >= 380) {
    lines.push({ text: "● Explore(ChatPanel 컴포넌트 구조 탐색)" });
  }

  // Subtle fade-in
  const opacity = interpolate(frame, [0, 32], [0, 1], { extrapolateRight: "clamp" });

  return (
    <AbsoluteFill style={{ background: "#0f0f10", alignItems: "center", justifyContent: "center", opacity }}>
      <FakeTerminal
        title="claude"
        lines={lines}
        cursorVisible={frame < ENTER_FRAME}
      />
    </AbsoluteFill>
  );
};
