import React from "react";

// 키네틱 reveal — 자식을 마스크(overflow hidden) 안에서 아래→위로 슬라이드업.
// y/opacity 는 useReveal() 이 만들어 준다. style 은 내부(움직이는) 요소에 적용.
export const Reveal: React.FC<{
  y: number;
  opacity: number;
  style?: React.CSSProperties;
  children: React.ReactNode;
}> = ({ y, opacity, style, children }) => (
  <div style={{ overflow: "hidden", paddingBottom: "0.14em" }}>
    <div style={{ transform: `translateY(${y}%)`, opacity, ...style }}>{children}</div>
  </div>
);
