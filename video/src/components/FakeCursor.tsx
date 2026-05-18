import React from "react";

type Props = {
  x: number;
  y: number;
  scale?: number;
};

export const FakeCursor: React.FC<Props> = ({ x, y, scale = 1 }) => {
  return (
    <svg
      width={24 * scale}
      height={24 * scale}
      viewBox="0 0 24 24"
      style={{
        position: "absolute",
        left: x,
        top: y,
        pointerEvents: "none",
        filter: "drop-shadow(0 2px 4px rgba(0,0,0,0.5))",
        transition: "transform 0.1s ease-out",
      }}
    >
      <path
        d="M 2 2 L 2 18 L 6 14 L 9 21 L 12 19.5 L 9 13 L 14 13 Z"
        fill="white"
        stroke="black"
        strokeWidth="1.2"
        strokeLinejoin="round"
      />
    </svg>
  );
};
