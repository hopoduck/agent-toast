import React from "react";
import { AbsoluteFill, useCurrentFrame, interpolate, staticFile, spring, useVideoConfig } from "remotion";
import { FakeTerminal, type TerminalLine } from "../components/FakeTerminal";
import { Stage } from "../components/Stage";
import { Reveal } from "../components/Reveal";
import { useFrameScaler, useReveal } from "../timing";
import { fraunces } from "../fonts";

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
  const f = useFrameScaler();

  // 리포트를 빠르게 좌르륵 출력 — 작업 로그가 흘러 완료되는 연출("이만큼 쭉 했구나").
  // FakeTerminal 이 bottom-anchored 라 줄이 쌓이며 자연히 위로 스크롤된다.
  const shownReport = Math.floor(
    interpolate(frame, [f(16), f(150)], [0, FULL_REPORT_LINES.length], {
      extrapolateLeft: "clamp",
      extrapolateRight: "clamp",
    }),
  );
  const lines: TerminalLine[] = [...BASE_LINES, ...FULL_REPORT_LINES.slice(0, shownReport)];

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
              letterSpacing: 8,
              color: "rgba(245,242,234,0.6)",
            }}
          >
            정확한 순간에 돌아오세요
          </Reveal>
        </div>
      </AbsoluteFill>
    </Stage>
  );
};
