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
  /** 다크 웜톤 테마 — 먹빛 무대와 톤을 맞춰 흰 창이 붕 뜨지 않게 한다. */
  dark?: boolean;
};

export const FakeNoteApp: React.FC<Props> = ({ notes, title, body, caretVisible, style, dark = false }) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const blinkOn = Math.floor(frame / (fps * 0.5)) % 2 === 0;

  const c = dark
    ? {
        surface: "#17151B",
        sidebar: "#131117",
        sidebarLabel: "#8A8590",
        itemActiveFg: "#F3EFE9",
        itemActiveBg: "rgba(255,138,60,0.16)",
        itemFg: "#8A8590",
        divider: "#2A2730",
        title: "#F3EFE9",
        body: "#D9D4CC",
        caret: "#F3EFE9",
        shadow: [
          "0 3px 10px rgba(0,0,0,0.5)",
          "0 36px 90px -22px rgba(0,0,0,0.7)",
          "0 0 0 1px rgba(255,255,255,0.05)",
          "0 0 70px -10px rgba(255,138,60,0.14)",
          "inset 0 1px 0 0 rgba(255,255,255,0.06)",
        ].join(", "),
      }
    : {
        surface: "#faf8f4",
        sidebar: "#f7f6f3",
        sidebarLabel: "#787774",
        itemActiveFg: "#37352f",
        itemActiveBg: "#ebebea",
        itemFg: "#787774",
        divider: "#d8d6d2",
        title: "#37352f",
        body: "#37352f",
        caret: "#37352f",
        shadow: [
          "0 3px 10px rgba(20,12,4,0.18)",
          "0 36px 90px -22px rgba(20,12,4,0.55)",
          "0 0 0 1px rgba(0,0,0,0.05)",
          "0 0 70px -10px rgba(255,176,92,0.12)",
          "inset 0 1px 0 0 rgba(255,255,255,0.9)",
        ].join(", "),
      };

  return (
    <div
      style={{
        width: 1100,
        height: 620,
        borderRadius: 14,
        overflow: "hidden",
        background: c.surface,
        display: "flex",
        boxShadow: c.shadow,
        ...style,
      }}
    >
      {/* Sidebar */}
      <div style={{ width: 260, background: c.sidebar, padding: "20px 12px" }}>
        <div style={{ fontSize: 13, color: c.sidebarLabel, padding: "0 8px 12px", fontWeight: 600 }}>📝 메모</div>
        {notes.map((n, i) => (
          <div
            key={i}
            style={{
              padding: "6px 8px",
              borderRadius: 6,
              fontSize: 14,
              color: n.active ? c.itemActiveFg : c.itemFg,
              background: n.active ? c.itemActiveBg : "transparent",
              marginBottom: 2,
              cursor: "default",
            }}
          >
            {n.title}
          </div>
        ))}
      </div>

      {/* Separator — 별도 div + 2px 로 H.264 인코더가 hairline 을 깎아내지 않게 */}
      <div style={{ width: 2, background: c.divider, flexShrink: 0 }} />

      {/* Editor */}
      <div style={{ flex: 1, padding: "40px 60px", overflow: "hidden" }}>
        <h1 style={{ fontSize: 36, fontWeight: 700, color: c.title, margin: 0, marginBottom: 24 }}>
          {title}
        </h1>
        <div style={{ fontSize: 16, lineHeight: 1.7, color: c.body, whiteSpace: "pre-wrap" }}>
          {body}
          {caretVisible && (
            <span
              style={{
                display: "inline-block",
                width: 2,
                height: 20,
                background: c.caret,
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
