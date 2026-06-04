import { useVideoConfig, useCurrentFrame, interpolate, Easing } from "remotion";

// 모든 씬의 하드코딩 프레임 상수가 기준으로 삼는 fps.
// 컴포지션 fps가 달라도(예: 디버그 30fps) 동일한 "시간"으로 재생되도록 변환한다.
export const BASE_FPS = 120;

// 씬 길이(120fps 기준). IntroVideo 의 Series 와 Root 의 composition 이 공유.
export const SCENE_FRAMES = {
  scene1: 440,
  scene2: 480,
  scene3: 360,
  scene4: 840,
} as const;

export const TOTAL_FRAMES =
  SCENE_FRAMES.scene1 + SCENE_FRAMES.scene2 + SCENE_FRAMES.scene3 + SCENE_FRAMES.scene4;

/** 120fps 기준 프레임 수 → 현재 fps 의 정수 프레임으로 변환 (durationInFrames 등 정수 필요처). */
export const scaleFrames = (framesAtBase: number, fps: number) =>
  Math.round((framesAtBase * fps) / BASE_FPS);

/** 씬 사이 크로스페이드 길이(120fps 기준). */
export const TRANSITION_FRAMES = 30;

/**
 * TransitionSeries 총 길이 = Σ시퀀스 - Σ전환(전환은 인접 시퀀스를 겹친다).
 * Root 의 IntroVideo composition durationInFrames 를 이 값과 일치시켜 끝의 빈 프레임/잘림을 막는다.
 * (씬 길이가 모두 4의 배수라 scaleFrames(합) === ΣscaleFrames(각) 이 성립.)
 */
export const introTotalFrames = (fps: number) =>
  scaleFrames(TOTAL_FRAMES, fps) - 3 * scaleFrames(TRANSITION_FRAMES, fps);

/**
 * 씬 컴포넌트 내부용 변환기. 120fps 기준으로 적은 프레임 값을 현재 fps 로 환산한다.
 * interpolate 의 입력 범위엔 float 그대로 넘겨도 안전(정수일 필요 없음).
 */
export const useFrameScaler = () => {
  const { fps } = useVideoConfig();
  return (framesAtBase: number) => (framesAtBase * fps) / BASE_FPS;
};

/**
 * 키네틱 reveal 값 생성기 — 마스크(overflow hidden) 안에서 아래→위로 슬라이드업.
 * startAtBase/durBase 는 120fps 기준 프레임. <Reveal> 컴포넌트에 y/opacity 로 넘긴다.
 */
export const useReveal = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const f = (n: number) => (n * fps) / BASE_FPS;
  return (startAtBase: number, durBase = 42) => {
    const p = interpolate(frame, [f(startAtBase), f(startAtBase + durBase)], [0, 1], {
      extrapolateLeft: "clamp",
      extrapolateRight: "clamp",
      easing: Easing.out(Easing.cubic),
    });
    return { y: (1 - p) * 115, opacity: p };
  };
};
