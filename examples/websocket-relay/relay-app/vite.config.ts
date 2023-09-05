import { sveltekit } from "@sveltejs/kit/vite";
import { defineConfig } from "vitest/config";

export default defineConfig({
  build: {
    sourcemap: true,
    target: "es2022",
  },
  plugins: [sveltekit()],
});
