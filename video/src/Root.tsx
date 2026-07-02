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
import { OgImage } from "./OgImage";
import { introTotalFrames } from "./timing";

// 프로덕션 120fps / 디버그 30fps. 씬 타이밍은 모두 fps-relative 라 두 fps 에서 동일 시간으로 재생된다.
const PROD_FPS = 90;
const DEBUG_FPS = 30;

export const RemotionRoot: React.FC = () => {
  return (
    <>
      <Composition
        id="IntroVideo"
        component={IntroVideo}
        durationInFrames={introTotalFrames(PROD_FPS)}
        fps={PROD_FPS}
        width={1280}
        height={720}
        defaultProps={{ locale: "ko" as const }}
      />
      {/* 영문판 — 같은 컴포넌트에 locale="en". `../.github/media/intro.en.webp` 소스. */}
      <Composition
        id="IntroVideoEn"
        component={IntroVideo}
        durationInFrames={introTotalFrames(PROD_FPS)}
        fps={PROD_FPS}
        width={1280}
        height={720}
        defaultProps={{ locale: "en" as const }}
      />
      {/* 디버그 프리뷰: 30fps(프레임 수 1/4) — `pnpm render:debug` 가 --scale=0.5 와 함께 렌더.
          레이아웃은 px 고정이라 width/height 는 1280x720 그대로 두고 해상도는 CLI scale 로 낮춘다. */}
      <Composition
        id="IntroVideoDebug"
        component={IntroVideo}
        durationInFrames={introTotalFrames(DEBUG_FPS)}
        fps={DEBUG_FPS}
        width={1280}
        height={720}
        defaultProps={{ locale: "ko" as const }}
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
        durationInFrames={840}
        fps={120}
        width={1280}
        height={720}
      />
      {/* OG/소셜 프리뷰 정적 이미지 — 같은 컴포넌트, 두 규격.
          `pnpm exec remotion still src/Root.tsx OgImage   ../.github/media/og.png`
          `pnpm exec remotion still src/Root.tsx OgImageGh ../.github/media/og-github.png` */}
      <Composition
        id="OgImage"
        component={OgImage}
        durationInFrames={1}
        fps={1}
        width={1200}
        height={630}
      />
      <Composition
        id="OgImageGh"
        component={OgImage}
        durationInFrames={1}
        fps={1}
        width={1280}
        height={640}
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
