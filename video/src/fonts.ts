import { loadFont as loadFraunces } from "@remotion/google-fonts/Fraunces";

// 디스플레이 세리프 — 따뜻한 옵티컬 세리프(빵/토스트 브랜드의 온기 + 에디토리얼 감성).
// 엔딩 워드마크·대형 헤드라인 전용. 본문/UI 는 계속 Pretendard.
export const fraunces = loadFraunces("normal", {
  weights: ["400", "600", "900"],
  subsets: ["latin"],
}).fontFamily;
