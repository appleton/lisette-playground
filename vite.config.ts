import { defineConfig } from "vite";
import monacoEditorPlugin from "vite-plugin-monaco-editor";

// When building for GitHub Pages the workflow sets VITE_BASE_PATH to the
// repository sub-path (e.g. /lisette-playground/).  Locally it defaults to /.
const base = process.env["VITE_BASE_PATH"] ?? "/";

export default defineConfig({
  base,
  plugins: [
    (monacoEditorPlugin as unknown as typeof monacoEditorPlugin.default).default(
      {
        languageWorkers: ["editorWorkerService"],
      }
    ),
  ],
  server: {
    headers: {
      // Enables SharedArrayBuffer in local dev (Monaco can use it).
      // GitHub Pages can't set these headers, but Monaco works without them.
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
