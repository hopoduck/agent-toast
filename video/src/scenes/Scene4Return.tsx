import React from "react";
import { AbsoluteFill, useCurrentFrame, interpolate, staticFile, spring, useVideoConfig } from "remotion";
import { FakeTerminal, type TerminalLine } from "../components/FakeTerminal";
import { Stage } from "../components/Stage";
import { Reveal } from "../components/Reveal";
import { useFrameScaler, useReveal } from "../timing";
import { fraunces } from "../fonts";
import { STRINGS, type Locale } from "../locale";

export const Scene4Return: React.FC<{ locale?: Locale }> = ({ locale = "ko" }) => {
  const frame = useCurrentFrame();
  const f = useFrameScaler();
  const s = STRINGS[locale];

  // 완료 리포트(응답 시작 → 파일 구조 → 검증 결과). 색은 로케일 무관, 텍스트만 번역.
  const report = s.report;
  // Scene3 끝에서 이미 보이던 사용자 입력 줄
  const baseLines: TerminalLine[] = [{ prefix: "❯", text: s.userInput }];

  // 리포트를 빠르게 좌르륵 출력 — 작업 로그가 흘러 완료되는 연출("이만큼 쭉 했구나").
  // FakeTerminal 이 bottom-anchored 라 줄이 쌓이며 자연히 위로 스크롤된다.
  const shownReport = Math.floor(
    interpolate(frame, [f(16), f(150)], [0, report.length], {
      extrapolateLeft: "clamp",
      extrapolateRight: "clamp",
    }),
  );
  const lines: TerminalLine[] = [...baseLines, ...report.slice(0, shownReport)];

  // 진입 줌인 — 클릭으로 터미널이 "열리는" 인과를 준다.
  const terminalScale = interpolate(frame, [0, f(48)], [0.965, 1], { extrapolateRight: "clamp" });

  // 종료값 0 으로 — 잔상이 남으면 로고 위에 어두운 패치가 보임
  const terminalFadeOut = interpolate(frame, [f(280), f(380)], [1, 0], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });

  // 엔딩에서 앰비언트 글로우를 한 단계 더 끌어올려 워드마크를 따뜻하게 감쌈
  const endingGlow = interpolate(frame, [f(300), f(400)], [1, 1.5], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });

  // Phase 3 엔딩 — 로고 팝(spring) + 줄별 키네틱 reveal(마스크 아래→위 슬라이드업).
  const { fps } = useVideoConfig();
  const logoPop = spring({ frame: frame - f(298), fps, config: { damping: 12, stiffness: 170, mass: 0.6 } });
  const rv = useReveal();
  const revLine1 = rv(316);
  const revLine2 = rv(330);
  const revTagline = rv(354, 38);

  return (
    <Stage glow={endingGlow}>
      {/* Terminal — 리포트. 크로스페이드로 Scene3 노트앱에서 넘어오며 줌인 + 좌르륵 출력 */}
      <AbsoluteFill style={{ alignItems: "center", justifyContent: "center", opacity: terminalFadeOut, transform: `scale(${terminalScale})` }}>
        <FakeTerminal title="claude" lines={lines} />
      </AbsoluteFill>

      {/* 우측 큰 로고 — 팝(spring) + 앰버 글로우 헤일로(좌 워드마크와 비대칭 균형) */}
      <AbsoluteFill style={{ alignItems: "flex-end", justifyContent: "center", paddingRight: 130 }}>
        <div
          style={{
            position: "relative",
            transform: `scale(${logoPop})`,
            opacity: Math.min(1, logoPop * 1.6),
          }}
        >
          <div
            style={{
              position: "absolute",
              inset: -54,
              borderRadius: "50%",
              background: "radial-gradient(circle, rgba(255,138,60,0.42), transparent 66%)",
              filter: "blur(28px)",
            }}
          />
          <img
            src={staticFile("logo.png")}
            alt=""
            style={{
              position: "relative",
              width: 224,
              height: 224,
              objectFit: "contain",
              filter: "drop-shadow(0 12px 30px rgba(0,0,0,0.55))",
            }}
          />
        </div>
      </AbsoluteFill>

      {/* Ending — 좌측 워드마크 + 태그라인(에디토리얼 비대칭) */}
      <AbsoluteFill style={{ justifyContent: "center", paddingLeft: 132 }}>
        <div style={{ display: "flex", flexDirection: "column", alignItems: "flex-start", gap: 30 }}>
          {/* 거대 워드마크 — Fraunces 900, 두 줄 세로 스택 */}
          <div
            style={{
              fontFamily: fraunces,
              fontWeight: 900,
              fontSize: 168,
              lineHeight: 0.92,
              letterSpacing: -5,
              color: "#F5F2EA",
            }}
          >
            <Reveal y={revLine1.y} opacity={revLine1.opacity}>
              Agent
            </Reveal>
            <Reveal y={revLine2.y} opacity={revLine2.opacity}>
              Toast<span style={{ color: "#FF8A3C" }}>.</span>
            </Reveal>
          </div>

          {/* 태그라인 */}
          <Reveal
            y={revTagline.y}
            opacity={revTagline.opacity}
            style={{
              marginLeft: 6,
              fontFamily: "var(--font-sans)",
              fontSize: 25,
              fontWeight: 500,
              // ko 는 음절 사이 넓은 트래킹이 우아하지만, en 소문자 문장엔 과해서 좁힌다
              letterSpacing: locale === "en" ? 2 : 8,
              color: "rgba(245,242,234,0.6)",
            }}
          >
            {s.tagline}
          </Reveal>
        </div>
      </AbsoluteFill>
    </Stage>
  );
};
