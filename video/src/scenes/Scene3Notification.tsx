import React from "react";
import { AbsoluteFill, useCurrentFrame, interpolate, spring, useVideoConfig, Easing } from "remotion";
import { FakeNoteApp } from "../components/FakeNoteApp";
import { FakeToast } from "../components/FakeToast";
import { FakeCursor } from "../components/FakeCursor";
import { Stage } from "../components/Stage";
import { Reveal } from "../components/Reveal";
import { useFrameScaler, useReveal } from "../timing";

const FULL_BODY = "이번 주에 정리할 내용\n\n- 리팩토링 결과 검토\n- 다음 스프린트 계획\n- 회의 일정 조율";

export const Scene3Notification: React.FC = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const f = useFrameScaler();

  // Toast slide-in (frames 0-15)
  const toastSpring = spring({
    frame,
    fps,
    config: { damping: 14, stiffness: 110, mass: 0.6 },
  });
  const toastX = interpolate(toastSpring, [0, 1], [400, 0]); // slide from right
  const toastOpacity = interpolate(toastSpring, [0, 1], [0, 1]);

  // Progress bar countdown
  const progress = interpolate(frame, [f(40), f(360)], [1, 0.45], {
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
  const cursorX = interpolate(frame, [f(100), f(260)], [600, 1010], cursorEase);
  const cursorY = interpolate(frame, [f(100), f(260)], [400, 645], cursorEase);

  const isHovering = frame >= f(260);

  // 클릭 = 누름(다운) → 잠깐 누른 채 유지(홀드) → 튀어오르는 릴리즈(back.out 바운스).
  // 홀드를 둬 "딸깍" 하고 실제로 누르는 텀을 만든다. 씬 끝(360) 전에 모션 완결.
  const cursorScale =
    frame < f(316)
      ? interpolate(frame, [f(305), f(316)], [1, 0.86], {
          extrapolateLeft: "clamp",
          extrapolateRight: "clamp",
          easing: Easing.in(Easing.quad),
        })
      : frame < f(334)
        ? 0.86
        : interpolate(frame, [f(334), f(354)], [0.86, 1], {
            extrapolateLeft: "clamp",
            extrapolateRight: "clamp",
            easing: Easing.out(Easing.back(1.4)),
          });

  // Toast position: bottom-right
  const toastRight = 40;
  const toastBottom = 40;

  // 캡션 reveal
  const rv = useReveal();
  const capIdx = rv(8, 28);
  const capText = rv(18, 38);

  return (
    <Stage glowY={52}>
      {/* 좌측 캡션 */}
      <AbsoluteFill>
        <div
          style={{
            position: "absolute",
            left: 88,
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
            03
          </Reveal>
          <Reveal
            y={capText.y}
            opacity={capText.opacity}
            style={{ fontFamily: "var(--font-sans)", fontSize: 104, fontWeight: 900, letterSpacing: -1, lineHeight: 1.05, color: "#F5F2EA" }}
          >
            알린다
          </Reveal>
        </div>
      </AbsoluteFill>

      {/* Note app — 우측, 다크 웜톤 */}
      <AbsoluteFill style={{ flexDirection: "row", alignItems: "center", justifyContent: "flex-end", paddingRight: 44 }}>
        <div style={{ transform: "scale(0.78)", transformOrigin: "right center" }}>
          <FakeNoteApp
            notes={[
              { title: "주간 정리", active: true },
              { title: "회의록 — 03/12" },
              { title: "아이디어 노트" },
              { title: "독서 메모" },
            ]}
            title="주간 정리"
            body={FULL_BODY}
            dark
          />
        </div>
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
      {frame >= f(100) && <FakeCursor x={cursorX} y={cursorY} scale={cursorScale} />}
    </Stage>
  );
};
