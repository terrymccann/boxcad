import { defineConfig } from "vite";

// Tauri loads the interface from a fixed port, so fail if it is taken instead of
// moving to another. 5174 leaves 5173 free for LayerCut running beside it.
export default defineConfig({
  root: "ui",
  clearScreen: false,
  server: { port: 5174, strictPort: true, watch: { ignored: ["**/src-tauri/**", "**/target/**"] } },
  build: { outDir: "../dist", emptyOutDir: true, target: "es2022", chunkSizeWarningLimit: 1000 },
});
