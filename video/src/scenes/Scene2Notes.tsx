import { AbsoluteFill, useCurrentFrame, interpolate } from "remotion";
import { FakeNoteApp } from "../components/FakeNoteApp";
import { Stage } from "../components/Stage";
import { Reveal } from "../components/Reveal";
import { useFrameScaler, useReveal } from "../timing";
import { STRINGS, type Locale } from "../locale";

const BODY_TYPING_START = 60;  // frame@120fps
const BODY_TYPING_END = 400;   // frame@120fps

export const Scene2Notes: React.FC<{ locale?: Locale }> = ({ locale = "ko" }) => {
  const frame = useCurrentFrame();
  const f = useFrameScaler();
  const s = STRINGS[locale];

  const typedCount = Math.floor(
    interpolate(frame, [f(BODY_TYPING_START), f(BODY_TYPING_END)], [0, s.noteBody.length], {
      extrapolateLeft: "clamp",
      extrapolateRight: "clamp",
    }),
  );
  const typedBody = s.noteBody.slice(0, typedCount);

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
            style={{ fontFamily: "var(--font-sans)", fontSize: locale === "en" ? 56 : 72, fontWeight: 900, letterSpacing: -1, lineHeight: locale === "en" ? 1.02 : 1.1, color: "#F5F2EA", whiteSpace: locale === "en" ? "normal" : "pre-line", ...(locale === "en" ? { maxWidth: 300 } : {}) }}
          >
            {s.cap2}
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
            notes={s.notes.map((title, i) => ({ title, active: i === 0 }))}
            title={s.noteTitle}
            body={typedBody}
            caretVisible={frame > f(BODY_TYPING_START) && frame < f(BODY_TYPING_END + 120)}
            dark
            notesLabel={s.notesLabel}
          />
        </div>
      </AbsoluteFill>
    </Stage>
  );
};
