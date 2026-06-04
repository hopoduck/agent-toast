import { useVideoConfig } from "remotion";
import { TransitionSeries, linearTiming } from "@remotion/transitions";
import { fade } from "@remotion/transitions/fade";
import { Scene1Terminal } from "./scenes/Scene1Terminal";
import { Scene2Notes } from "./scenes/Scene2Notes";
import { Scene3Notification } from "./scenes/Scene3Notification";
import { Scene4Return } from "./scenes/Scene4Return";
import { SCENE_FRAMES, TRANSITION_FRAMES, scaleFrames } from "./timing";

export const IntroVideo: React.FC = () => {
  const { fps } = useVideoConfig();
  const dur = (framesAtBase: number) => scaleFrames(framesAtBase, fps);
  const tdur = dur(TRANSITION_FRAMES);
  // 씬 사이를 부드러운 크로스페이드로 — 먹빛 배경끼리 겹쳐 자연스럽게 디졸브된다.
  const crossfade = (
    <TransitionSeries.Transition presentation={fade()} timing={linearTiming({ durationInFrames: tdur })} />
  );

  return (
    <TransitionSeries>
      <TransitionSeries.Sequence durationInFrames={dur(SCENE_FRAMES.scene1)}>
        <Scene1Terminal />
      </TransitionSeries.Sequence>
      {crossfade}
      <TransitionSeries.Sequence durationInFrames={dur(SCENE_FRAMES.scene2)}>
        <Scene2Notes />
      </TransitionSeries.Sequence>
      {crossfade}
      <TransitionSeries.Sequence durationInFrames={dur(SCENE_FRAMES.scene3)}>
        <Scene3Notification />
      </TransitionSeries.Sequence>
      {crossfade}
      <TransitionSeries.Sequence durationInFrames={dur(SCENE_FRAMES.scene4)}>
        <Scene4Return />
      </TransitionSeries.Sequence>
    </TransitionSeries>
  );
};
