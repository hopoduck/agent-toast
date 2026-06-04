# CLAUDE.md — video

Remotion intro video for the agent-toast README.

## Commands

```bash
pnpm dev                  # Remotion Studio (live preview)
pnpm render               # Production: IntroVideo 90fps/crf23 → ../.github/media/intro.mp4 (~5MB)
pnpm render:debug         # Fast preview: IntroVideoDebug 30fps + --scale=0.5 → out/preview.mp4 (~40s)
pnpm still -- --frame=N   # Single full-res frame → out/frame.png
```

`out/` is git-ignored (debug artifacts / stills). Render a short clip for motion review with
`pnpm exec remotion render src/Root.tsx IntroVideo out/clip.mp4 --frames=A-B`.

## mp4 → animated webp (README embed)

GitHub README auto-plays **webp/gif only** — `<img src="*.mp4">` does NOT play (and `<video autoplay>`
needs `muted` + is flaky). So the README references `intro.webp`; convert the rendered mp4 with ffmpeg
**from the repo root**:

```bash
ffmpeg -y -i .github/media/intro.mp4 \
  -vcodec libwebp -filter:v "fps=60,scale=960:-1:flags=lanczos" \
  -lossless 0 -compression_level 6 -q:v 80 -loop 0 -an -vsync 0 \
  .github/media/intro.webp
```

webp animation is ~2–3× larger than h264 at equal quality, so the mp4 is the small one. Tuning knobs:

| Knob | Trade-off | Guidance |
| ---- | --------- | -------- |
| `scale=960` | resolution ↔ size | GitHub README body is ~900px wide → 960 is plenty (don't go 1280) |
| `fps=60` | smoothness ↔ size | webp size scales ~linearly with fps; 24–30 is the web norm, 60 for extra smoothness |
| `-q:v 80` | quality ↔ size | 80 ≈ good; 90 sharper/bigger, 70 smaller |

Size guide (≈17s clip): `960/q80/fps60` ≈ **11MB** · `1280/q90/fps60` ≈ 25MB · `960/q60/fps24` ≈ 3.5MB.
Default to `960/q80/fps60` unless asked otherwise. The mp4 stays local (webp source); no need to commit it.

## fps-relative timing

All scene frame constants are authored at **120fps** (`BASE_FPS` in `src/timing.ts`).
`useFrameScaler()` / `scaleFrames()` convert to the composition's fps, so `IntroVideo` (90fps prod)
and `IntroVideoDebug` (30fps) play with identical timing. **When adding any animation, wrap raw frame
values in `f()`** (from `useFrameScaler`) — never hardcode bare frame numbers, or the debug/prod fps
will diverge. `useReveal()` + `<Reveal>` (mask slide-up) is the shared kinetic-text helper.

Scenes: `Scene1Terminal` (맡긴다) → `Scene2Notes` (다른 일을 한다) → `Scene3Notification` (알린다)
→ `Scene4Return` (report scroll + ending wordmark). `IntroVideo` stitches them with `TransitionSeries`
crossfades; total length is `introTotalFrames(fps)` (Σ scenes − Σ transitions), mirrored in `Root.tsx`.
