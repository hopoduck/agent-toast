import { Series } from "remotion";
import { Scene1Terminal } from "./scenes/Scene1Terminal";
import { Scene2Notes } from "./scenes/Scene2Notes";
import { Scene3Notification } from "./scenes/Scene3Notification";
import { Scene4Return } from "./scenes/Scene4Return";

export const IntroVideo: React.FC = () => {
  return (
    <Series>
      <Series.Sequence durationInFrames={440}>
        <Scene1Terminal />
      </Series.Sequence>
      <Series.Sequence durationInFrames={480}>
        <Scene2Notes />
      </Series.Sequence>
      <Series.Sequence durationInFrames={360}>
        <Scene3Notification />
      </Series.Sequence>
      <Series.Sequence durationInFrames={540}>
        <Scene4Return />
      </Series.Sequence>
    </Series>
  );
};
