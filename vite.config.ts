import { defineConfig } from "vite";
import monacoEditorPlugin from "vite-plugin-monaco-editor";

export default defineConfig({
  plugins: [
    (monacoEditorPlugin as unknown as typeof monacoEditorPlugin.default).default(
      {
        languageWorkers: ["editorWorkerService"],
      }
    ),
  ],
  server: {
    headers: {
      // Required for SharedArrayBuffer (Monaco uses it)
      "Cross-Origin-Opener-Policy": "same-origin",
      "Cross-Origin-Embedder-Policy": "require-corp",
    },
  },
  optimizeDeps: {
    exclude: ["monaco-editor"],
  },
  build: {
    target: "es2020",
  },
  worker: {
    format: "es",
  },
});
