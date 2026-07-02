import React from "react";
import { AbsoluteFill, Img, staticFile } from "remotion";
import { Stage } from "./components/Stage";
import { fraunces } from "./fonts";

// OG/소셜 프리뷰 정적 이미지 — 엔딩 화면(워드마크 + 토스트 로고 + 앰버 헤일로)을
// OG 캔버스에 맞춰 재구성. 번호/커서/태그라인 없이 브랜드 락업만.
// 크기 무관: 1200×630, 1280×640 두 컴포지션이 같은 컴포넌트를 공유(중앙 정렬).
export const OgImage: React.FC = () => {
  return (
    <Stage glow={1.3}>
      <AbsoluteFill
        style={{
          flexDirection: "row",
          alignItems: "center",
          justifyContent: "center",
          gap: 84,
        }}
      >
        {/* 워드마크 — Fraunces 900, 두 줄 세로 스택 (엔딩과 동일 톤) */}
        <div
          style={{
            fontFamily: fraunces,
            fontWeight: 900,
            fontSize: 132,
            lineHeight: 0.9,
            letterSpacing: -4,
            color: "#F5F2EA",
          }}
        >
          Agent
          <br />
          Toast<span style={{ color: "#FF8A3C" }}>.</span>
        </div>

        {/* 로고 — 앰버 글로우 헤일로 */}
        <div style={{ position: "relative" }}>
          <div
            style={{
              position: "absolute",
              inset: -60,
              borderRadius: "50%",
              background:
                "radial-gradient(circle, rgba(255,138,60,0.45), transparent 66%)",
              filter: "blur(30px)",
            }}
          />
          <Img
            src={staticFile("logo.png")}
            style={{
              position: "relative",
              width: 252,
              height: 252,
              objectFit: "contain",
              filter: "drop-shadow(0 12px 30px rgba(0,0,0,0.55))",
            }}
          />
        </div>
      </AbsoluteFill>
    </Stage>
  );
};
