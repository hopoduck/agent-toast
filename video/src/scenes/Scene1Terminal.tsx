import React from "react";
import { AbsoluteFill, useCurrentFrame, interpolate } from "remotion";
import { FakeTerminal, type TerminalLine } from "../components/FakeTerminal";
import { Stage } from "../components/Stage";
import { Reveal } from "../components/Reveal";
import { useFrameScaler, useReveal } from "../timing";

const USER_INPUT = "ChatPanel 컴포넌트 리팩터링좀 해줘.";
const TYPING_START = 40;   // frame@120fps
const TYPING_END = 280;    // frame@120fps — 240프레임 동안 21자, 자연스러운 타이핑 속도
const ENTER_FRAME = 300;
const RESPONSE_START = 320;

export const Scene1Terminal: React.FC = () => {
  const frame = useCurrentFrame();
  const f = useFrameScaler();

  // Typed characters
  const typedCount = Math.floor(
    interpolate(frame, [f(TYPING_START), f(TYPING_END)], [0, USER_INPUT.length], {
      extrapolateLeft: "clamp",
      extrapolateRight: "clamp",
    }),
  );
  const typedText = USER_INPUT.slice(0, typedCount);

  const showResponse = frame >= f(RESPONSE_START);

  const lines: TerminalLine[] = [
    { prefix: "❯", text: typedText },
  ];
  if (showResponse) {
    lines.push({
      text: "ChatPanel 컴포넌트 현황 파악부터 하겠습니다. Explore 에이전트로 구조랑 의존성 쫙 훑어볼게요.",
      color: "#a3a3a3",
    });
  }
  // Scene1 끝부분(~frame 380@120fps 이후)에 Explore 에이전트 호출 줄을 미리 넣어 Scene2 배경과 자연 전환
  if (frame >= f(380)) {
    lines.push({ text: "● Explore(ChatPanel 컴포넌트 구조 탐색)" });
  }

  // Subtle fade-in
  const opacity = interpolate(frame, [0, f(32)], [0, 1], { extrapolateRight: "clamp" });
  // 무대 글로우도 함께 천천히 차오르게 (오프닝)
  const stageGlow = interpolate(frame, [0, f(48)], [0.3, 1], { extrapolateRight: "clamp" });

  // 캡션 reveal — 인덱스 먼저, 텍스트 스태거
  const rv = useReveal();
  const capIdx = rv(8, 28);
  const capText = rv(18, 38);

  return (
    <Stage glow={stageGlow}>
      {/* 좌측 에디토리얼 캡션 — 비대칭 내러티브 + reveal */}
      <AbsoluteFill>
        <div
          style={{
            position: "absolute",
            left: 92,
            top: "50%",
            transform: "translateY(-50%)",
            display: "flex",
            flexDirection: "column",
            alignItems: "flex-start",
            gap: 12,
          }}
        >
          <Reveal
            y={capIdx.y}
            opacity={capIdx.opacity}
            style={{ fontFamily: "var(--font-sans)", fontSize: 19, fontWeight: 700, letterSpacing: 7, color: "#FF8A3C" }}
          >
            01
          </Reveal>
          <Reveal
            y={capText.y}
            opacity={capText.opacity}
            style={{ fontFamily: "var(--font-sans)", fontSize: 104, fontWeight: 900, letterSpacing: -1, lineHeight: 1.05, color: "#F5F2EA" }}
          >
            맡긴다
          </Reveal>
        </div>
      </AbsoluteFill>

      {/* 터미널 — 우측으로 밀어 비대칭 구도(Scene2 와 위치 일치) */}
      <AbsoluteFill style={{ flexDirection: "row", alignItems: "center", justifyContent: "flex-end", paddingRight: 64, opacity }}>
        <div style={{ transform: "scale(0.86)", transformOrigin: "right center" }}>
          <FakeTerminal
            title="claude"
            lines={lines}
            cursorVisible={frame < f(ENTER_FRAME)}
          />
        </div>
      </AbsoluteFill>
    </Stage>
  );
};
