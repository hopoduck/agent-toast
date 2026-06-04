import React from "react";
import { AbsoluteFill, useCurrentFrame } from "remotion";

// 브랜드 앰버(구운 식빵 톤). rgb 문자열로 알파만 조절.
const AMBER = "255, 138, 60";

// 필름 그레인 — feTurbulence 노이즈를 data URI 로. seed 를 프레임마다 바꿔
// 실제로 다른 패턴이 생성되게(=떨림이 눈에 보이게) 한다.
const noiseUri = (seed: number) =>
  `data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='240' height='240'%3E%3Cfilter id='n'%3E%3CfeTurbulence type='fractalNoise' baseFrequency='0.7' numOctaves='2' seed='${seed}'/%3E%3C/filter%3E%3Crect width='100%25' height='100%25' filter='url(%23n)'/%3E%3C/svg%3E`;

type Props = {
  children: React.ReactNode;
  /** 앰비언트 글로우 강도 0~1 — 씬별 분위기 조절용(오프닝 차오름, 엔딩 강조 등) */
  glow?: number;
  /** (하위호환) 예전 중앙 글로우의 수직 위치. 비대칭 조명으로 바뀌며 더는 쓰지 않음. */
  glowY?: number;
};

// 에디토리얼 무대: 먹빛 블랙 위에 한쪽(좌상단)에서 들어오는 비대칭 앰버 스포트라이트.
// 중앙 대칭 wash 를 버려 "잡지 표지 조명" 같은 방향성과 깊이를 준다.
export const Stage: React.FC<Props> = ({ children, glow = 1 }) => {
  const frame = useCurrentFrame();
  const noise = noiseUri(frame % 8);
  const gx = (frame * 11) % 240;
  const gy = (frame * 7) % 240;

  return (
    <AbsoluteFill style={{ background: "#0B0A0D" }}>
      {/* 좌상단에서 비스듬히 들어오는 핵심 앰버 스포트 — 방향성 있는 따뜻한 광 */}
      <AbsoluteFill
        style={{
          background: `radial-gradient(1150px 820px at 16% 4%, rgba(${AMBER},${(0.22 * glow).toFixed(3)}), transparent 58%)`,
        }}
      />
      {/* 핫스폿 — 광원 근처를 한 단계 더 밝혀 빛이 '꽂히는' 느낌 */}
      <AbsoluteFill
        style={{
          background: `radial-gradient(420px 320px at 13% 0%, rgba(${AMBER},${(0.14 * glow).toFixed(3)}), transparent 60%)`,
        }}
      />
      {/* 반대편(우하단) 차가운 깊이 — 따뜻↔차가움 대비로 입체감 */}
      <AbsoluteFill
        style={{
          background:
            "radial-gradient(1000px 760px at 92% 100%, rgba(18,16,30,0.7), transparent 52%)",
        }}
      />

      {/* 콘텐츠 (창·타이포) */}
      {children}

      {/* 방향성 비네팅 — 광원 반대쪽을 더 눌러 시선을 좌상단→중앙으로 흐르게 */}
      <AbsoluteFill
        style={{
          pointerEvents: "none",
          background:
            "radial-gradient(135% 125% at 28% 16%, transparent 44%, rgba(0,0,0,0.66) 100%)",
        }}
      />

      {/* Film grain — 보이라고 넣는 게 아니라 다크 그라데이션의 밴딩(색 계단)을
          흩어주는 보조 장치. 입자가 드러나지 않게 최소치만 깐다. */}
      <AbsoluteFill
        style={{
          pointerEvents: "none",
          opacity: 0.045,
          backgroundImage: `url("${noise}")`,
          backgroundRepeat: "repeat",
          backgroundSize: "240px 240px",
          backgroundPosition: `${gx}px ${gy}px`,
        }}
      />
    </AbsoluteFill>
  );
};
