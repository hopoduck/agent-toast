import { defineConfig } from "vitest/config";

// Without a local config, vitest walks up and loads the repo root's
// vite.config.ts, which requires the frontend's deps (absent in the
// worker-only CI job).
export default defineConfig({
  test: {
    include: ["test/**/*.test.ts"],
  },
});
