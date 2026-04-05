/**
 * TextMate-based syntax highlighting for Lisette.
 *
 * Replaces the Monarch tokenizer with the official grammar from
 * `editors/vscode/syntaxes/lisette.tmLanguage.json` in the Lisette repo.
 *
 * Loads lazily and falls back to Monarch silently on any error.
 */

import type * as Monaco from "monaco-editor";
import { loadWASM, OnigScanner, OnigString } from "onigasm";
import { Registry } from "vscode-textmate";
import { wireTmGrammars } from "monaco-editor-textmate";

import { LANG_ID } from "./language.js";

const ONIGASM_WASM_URL = "/onigasm.wasm";
const TM_GRAMMAR_URL   = "/lisette.tmLanguage.json";
const TM_SCOPE         = "source.lisette";

let _wirePromise: Promise<void> | null = null;
let _onigasmLoaded = false;

async function ensureOnigasm(): Promise<void> {
  if (_onigasmLoaded) return;
  await loadWASM(ONIGASM_WASM_URL);
  _onigasmLoaded = true;
}

export async function wireTextMateGrammar(monaco: typeof Monaco): Promise<void> {
  if (_wirePromise) return _wirePromise;

  _wirePromise = (async () => {
    // 1. Boot Oniguruma WASM
    await ensureOnigasm();

    // 2. Build the vscode-textmate Registry, providing the Oniguruma lib
    const onigLib = Promise.resolve({
      createOnigScanner: (patterns: string[]) => new OnigScanner(patterns),
      createOnigString:  (s: string)          => new OnigString(s),
    });

    const registry = new Registry({
      onigLib,
      loadGrammar: async (scopeName: string) => {
        if (scopeName !== TM_SCOPE) return null;
        const resp = await fetch(TM_GRAMMAR_URL);
        if (!resp.ok) throw new Error(`Grammar fetch failed: ${resp.status}`);
        return resp.json();
      },
    });

    // 3. Wire into Monaco (cast needed – vscode-textmate major version
    //    may differ from what monaco-editor-textmate was typed against)
    const grammarMap = new Map([[LANG_ID, TM_SCOPE]]);
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    await wireTmGrammars(monaco, registry as any, grammarMap);
  })();

  return _wirePromise;
}
