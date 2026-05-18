import React from "react";
import { useCurrentFrame, useVideoConfig } from "remotion";

type NoteItem = {
  title: string;
  active?: boolean;
};

type Props = {
  notes: NoteItem[];
  title: string;
  body: string; // \n 으로 줄바꿈
  caretVisible?: boolean;
  style?: React.CSSProperties;
};

export const FakeNoteApp: React.FC<Props> = ({ notes, title, body, caretVisible, style }) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const blinkOn = Math.floor(frame / (fps * 0.5)) % 2 === 0;

  return (
    <div
      style={{
        width: 1100,
        height: 620,
        borderRadius: 12,
        overflow: "hidden",
        background: "#ffffff",
        display: "flex",
        boxShadow: "0 20px 60px rgba(0,0,0,0.3)",
        ...style,
      }}
    >
      {/* Sidebar */}
      <div style={{ width: 260, background: "#f7f6f3", padding: "20px 12px" }}>
        <div style={{ fontSize: 13, color: "#787774", padding: "0 8px 12px", fontWeight: 600 }}>📝 메모</div>
        {notes.map((n, i) => (
          <div
            key={i}
            style={{
              padding: "6px 8px",
              borderRadius: 6,
              fontSize: 14,
              color: n.active ? "#37352f" : "#787774",
              background: n.active ? "#ebebea" : "transparent",
              marginBottom: 2,
              cursor: "default",
            }}
          >
            {n.title}
          </div>
        ))}
      </div>

      {/* Separator — 별도 div + 2px 로 H.264 인코더가 hairline 을 깎아내지 않게 */}
      <div style={{ width: 2, background: "#d8d6d2", flexShrink: 0 }} />

      {/* Editor */}
      <div style={{ flex: 1, padding: "40px 60px", overflow: "hidden" }}>
        <h1 style={{ fontSize: 36, fontWeight: 700, color: "#37352f", margin: 0, marginBottom: 24 }}>
          {title}
        </h1>
        <div style={{ fontSize: 16, lineHeight: 1.7, color: "#37352f", whiteSpace: "pre-wrap" }}>
          {body}
          {caretVisible && (
            <span
              style={{
                display: "inline-block",
                width: 2,
                height: 20,
                background: "#37352f",
                marginLeft: 1,
                verticalAlign: "text-bottom",
                opacity: blinkOn ? 1 : 0,
              }}
            />
          )}
        </div>
      </div>

    </div>
  );
};
