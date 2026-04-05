import type * as Monaco from "monaco-editor";

export const THEME_NAME = "lisette-dark";

export function registerTheme(monaco: typeof Monaco): void {
  monaco.editor.defineTheme(THEME_NAME, {
    base: "vs-dark",
    inherit: true,
    rules: [
      // Base
      { token: "", foreground: "e6edf3", background: "0d1117" },

      // Syntax
      { token: "keyword",          foreground: "ff7b72", fontStyle: "bold" },
      { token: "keyword.operator", foreground: "f78166", fontStyle: "" },
      { token: "type",             foreground: "ffa657" },
      { token: "type.identifier",  foreground: "ffa657" },
      { token: "identifier",       foreground: "e6edf3" },

      { token: "string",           foreground: "a5d6ff" },
      { token: "string.escape",    foreground: "79c0ff" },
      { token: "string.invalid",   foreground: "f85149" },

      { token: "number",           foreground: "79c0ff" },
      { token: "number.float",     foreground: "79c0ff" },
      { token: "number.hex",       foreground: "79c0ff" },

      { token: "comment",          foreground: "8b949e", fontStyle: "italic" },

      { token: "operator",         foreground: "c9d1d9" },
      { token: "delimiter",        foreground: "8b949e" },
      { token: "brackets",         foreground: "e6edf3" },
    ],
    colors: {
      "editor.background":             "#0d1117",
      "editor.foreground":             "#e6edf3",
      "editor.lineHighlightBackground":"#161b22",
      "editor.selectionBackground":    "#264f78",
      "editorCursor.foreground":       "#7c6cff",
      "editorLineNumber.foreground":   "#484f58",
      "editorLineNumber.activeForeground": "#8b949e",
      "editorIndentGuide.background":  "#21262d",
      "editorIndentGuide.activeBackground": "#444c56",
      "editorWidget.background":       "#161b22",
      "editorWidget.border":           "#30363d",
      "editorSuggestWidget.background":"#161b22",
      "editorSuggestWidget.border":    "#30363d",
      "editorSuggestWidget.selectedBackground": "#264f78",
      "editorHoverWidget.background":  "#161b22",
      "editorHoverWidget.border":      "#30363d",
      "scrollbar.shadow":              "#00000060",
      "scrollbarSlider.background":    "#30363d80",
      "scrollbarSlider.hoverBackground":"#484f5880",
      "input.background":              "#0d1117",
      "input.border":                  "#30363d",
      "focusBorder":                   "#7c6cff",
    },
  });
}
