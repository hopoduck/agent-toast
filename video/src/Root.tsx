import "./tailwind.css";
import { Composition, registerRoot, AbsoluteFill } from "remotion";
import { IntroVideo } from "./IntroVideo";
import { FakeCursor } from "./components/FakeCursor";
import { FakeTerminal } from "./components/FakeTerminal";
import { FakeNoteApp } from "./components/FakeNoteApp";
import { FakeToast } from "./components/FakeToast";
import { Scene1Terminal } from "./scenes/Scene1Terminal";
import { Scene2Notes } from "./scenes/Scene2Notes";
import { Scene3Notification } from "./scenes/Scene3Notification";
import { Scene4Return } from "./scenes/Scene4Return";

export const RemotionRoot: React.FC = () => {
  return (
    <>
      <Composition
        id="IntroVideo"
        component={IntroVideo}
        durationInFrames={1820}
        fps={120}
        width={1280}
        height={720}
      />
      <Composition
        id="Scene1Terminal"
        component={Scene1Terminal}
        durationInFrames={440}
        fps={120}
        width={1280}
        height={720}
      />
      <Composition
        id="Scene2Notes"
        component={Scene2Notes}
        durationInFrames={480}
        fps={120}
        width={1280}
        height={720}
      />
      <Composition
        id="Scene3Notification"
        component={Scene3Notification}
        durationInFrames={360}
        fps={120}
        width={1280}
        height={720}
      />
      <Composition
        id="Scene4Return"
        component={Scene4Return}
        durationInFrames={540}
        fps={120}
        width={1280}
        height={720}
      />
      <Composition
        id="DebugCursor"
        component={() => (
          <AbsoluteFill style={{ background: "#444" }}>
            <FakeCursor x={400} y={300} />
          </AbsoluteFill>
        )}
        durationInFrames={120}
        fps={120}
        width={800}
        height={600}
      />
      <Composition
        id="DebugTerminal"
        component={() => (
          <AbsoluteFill style={{ background: "#0f0f10", alignItems: "center", justifyContent: "center" }}>
            <FakeTerminal
              title="claude"
              lines={[
                { prefix: "❯", text: "리팩토링 작업 시작해줘" },
                { text: "분석 중...", color: "#a3a3a3" },
              ]}
              cursorVisible
            />
          </AbsoluteFill>
        )}
        durationInFrames={120}
        fps={120}
        width={1280}
        height={720}
      />
      <Composition
        id="DebugNoteApp"
        component={() => (
          <AbsoluteFill style={{ background: "#e5e5e5", alignItems: "center", justifyContent: "center" }}>
            <FakeNoteApp
              notes={[
                { title: "주간 정리", active: true },
                { title: "회의록 — 03/12" },
                { title: "아이디어 노트" },
                { title: "독서 메모" },
              ]}
              title="주간 정리"
              body={"이번 주에 정리할 내용\n\n- 리팩토링 결과 검토\n- 다음 스프린트 계획"}
              caretVisible
            />
          </AbsoluteFill>
        )}
        durationInFrames={120}
        fps={120}
        width={1280}
        height={720}
      />
      <Composition
        id="DebugToast"
        component={() => (
          <AbsoluteFill style={{ background: "#1a1a1a", alignItems: "center", justifyContent: "center" }}>
            <FakeToast eventLabel="작업 완료" windowTitle="Claude Code" message="작업이 완료되었습니다" progress={0.7} />
          </AbsoluteFill>
        )}
        durationInFrames={120}
        fps={120}
        width={1280}
        height={720}
      />
    </>
  );
};

registerRoot(RemotionRoot);
