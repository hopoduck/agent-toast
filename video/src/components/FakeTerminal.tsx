import React from "react";
import { useCurrentFrame, useVideoConfig } from "remotion";

export type TerminalLine = {
  text: string;
  color?: string; // CSS color
  prefix?: string; // 예: "❯ " 또는 "$"
};

type Props = {
  title?: string;
  lines: TerminalLine[];
  cursorVisible?: boolean;
  style?: React.CSSProperties;
  dimmed?: boolean;
};

export const FakeTerminal: React.FC<Props> = ({
  title = "claude",
  lines,
  cursorVisible = false,
  style,
  dimmed = false,
}) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  // 0.5초 on / 0.5초 off (실제 터미널 커서 깜빡임 주기 ~1초)
  const blinkOn = Math.floor(frame / (fps * 0.5)) % 2 === 0;

  return (
    <div
      style={{
        width: 900,
        height: 540,
        borderRadius: 12,
        overflow: "hidden",
        background: "#1e1e1e",
        boxShadow: "0 20px 60px rgba(0,0,0,0.4)",
        fontFamily: "ui-monospace, SFMono-Regular, Menlo, monospace",
        color: "#e4e4e7",
        opacity: dimmed ? 0.35 : 1,
        filter: dimmed ? "blur(2px)" : "none",
        display: "flex",
        flexDirection: "column",
        ...style,
      }}
    >
      {/* Title bar with traffic lights */}
      <div
        style={{
          height: 32,
          background: "#2d2d2d",
          display: "flex",
          alignItems: "center",
          padding: "0 12px",
          gap: 8,
          borderBottom: "1px solid #3a3a3a",
          flexShrink: 0,
        }}
      >
        <div style={{ width: 12, height: 12, borderRadius: "50%", background: "#ff5f56" }} />
        <div style={{ width: 12, height: 12, borderRadius: "50%", background: "#ffbd2e" }} />
        <div style={{ width: 12, height: 12, borderRadius: "50%", background: "#27c93f" }} />
        <div style={{ flex: 1, textAlign: "center", color: "#888", fontSize: 13 }}>{title}</div>
      </div>

      {/* Body — bottom-anchored: newest line at bottom, oldest scroll off top */}
      <div
        style={{
          padding: 16,
          fontSize: 16,
          lineHeight: 1.5,
          flex: 1,
          minHeight: 0,
          overflow: "hidden",
          display: "flex",
          flexDirection: "column",
          justifyContent: "flex-end",
        }}
      >
        {lines.map((line, i) => (
          <div key={i} style={{ color: line.color ?? "#e4e4e7", whiteSpace: "pre-wrap" }}>
            {line.prefix && <span style={{ color: "#22c55e", marginRight: 6 }}>{line.prefix}</span>}
            {line.text}
            {cursorVisible && i === lines.length - 1 && (
              <span
                style={{
                  display: "inline-block",
                  width: 8,
                  height: 18,
                  background: "#e4e4e7",
                  marginLeft: 2,
                  verticalAlign: "text-bottom",
                  opacity: blinkOn ? 1 : 0,
                }}
              />
            )}
          </div>
        ))}
      </div>

    </div>
  );
};
