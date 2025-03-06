import { defineConfig } from "vite"
import wasm from "vite-plugin-wasm";
import topLevelAwait from "vite-plugin-top-level-await";

export default defineConfig({
  plugins: [wasm(), topLevelAwait()],
  build: {
    target: "esnext",
    assetsInlineLimit: 0, // ensure WASM file isn't inlined
  },
  server: {
    fs: {
      // Allow serving files from node_modules
      allow: ['..']
    }
  }
})
