import wasm from "vite-plugin-wasm";
import topLevelAwait from "vite-plugin-top-level-await";
import { defineConfig } from "astro/config";

export default defineConfig({
  vite: {
    plugins: [wasm(), topLevelAwait()],
  },
});
