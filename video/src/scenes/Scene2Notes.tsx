import { AbsoluteFill, useCurrentFrame, interpolate } from "remotion";
import { FakeNoteApp } from "../components/FakeNoteApp";
import { Stage } from "../components/Stage";
import { Reveal } from "../components/Reveal";
import { useFrameScaler, useReveal } from "../timing";

const BODY_FULL = "이번 주에 정리할 내용\n\n- 리팩토링 결과 검토\n- 다음 스프린트 계획\n- 회의 일정 조율";
const BODY_TYPING_START = 60;  // frame@120fps
const BODY_TYPING_END = 400;   // frame@120fps

export const Scene2Notes: React.FC = () => {
  const frame = useCurrentFrame();
  const f = useFrameScaler();

  const typedCount = Math.floor(
    interpolate(frame, [f(BODY_TYPING_START), f(BODY_TYPING_END)], [0, BODY_FULL.length], {
      extrapolateLeft: "clamp",
      extrapolateRight: "clamp",
    }),
  );
  const typedBody = BODY_FULL.slice(0, typedCount);

  const noteAppOpacity = interpolate(frame, [0, f(48)], [0, 1], { extrapolateRight: "clamp" });
  const noteAppTranslate = interpolate(frame, [0, f(48)], [20, 0], { extrapolateRight: "clamp" });

  // 캡션 reveal
  const rv = useReveal();
  const capIdx = rv(12, 28);
  const capText = rv(22, 38);

  return (
    <Stage>
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
            02
          </Reveal>
          <Reveal
            y={capText.y}
            opacity={capText.opacity}
            style={{ fontFamily: "var(--font-sans)", fontSize: 72, fontWeight: 900, letterSpacing: -1, lineHeight: 1.1, color: "#F5F2EA" }}
          >
            다른 일을
            <br />
            한다
          </Reveal>
        </div>
      </AbsoluteFill>

      {/* 노트앱 — 우측, 다크 웜톤 */}
      <AbsoluteFill
        style={{
          flexDirection: "row",
          alignItems: "center",
          justifyContent: "flex-end",
          paddingRight: 44,
          opacity: noteAppOpacity,
          transform: `translateY(${noteAppTranslate}px)`,
        }}
      >
        <div style={{ transform: "scale(0.78)", transformOrigin: "right center" }}>
          <FakeNoteApp
            notes={[
              { title: "주간 정리", active: true },
              { title: "회의록 — 03/12" },
              { title: "아이디어 노트" },
              { title: "독서 메모" },
            ]}
            title="주간 정리"
            body={typedBody}
            caretVisible={frame > f(BODY_TYPING_START) && frame < f(BODY_TYPING_END + 120)}
            dark
          />
        </div>
      </AbsoluteFill>
    </Stage>
  );
};
