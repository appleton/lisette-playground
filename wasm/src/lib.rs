//! WebAssembly bindings for the Lisette compiler.
//!
//! All functions are called directly from JavaScript via wasm-bindgen.
//! Diagnostics are serialised as JSON strings so the TS layer can decode them.

use serde::Serialize;
use wasm_bindgen::prelude::*;

// ─── Panic hook ───────────────────────────────────────────────────────────────
#[wasm_bindgen(start)]
pub fn init() {
    console_error_panic_hook::set_once();
}

// ─── Serialisable output types ────────────────────────────────────────────────

#[derive(Serialize, Default)]
struct JsDiagnostic {
    severity: String,
    message: String,
    line: u32,
    col: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    end_line: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    end_col: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    code: Option<String>,
}

#[derive(Serialize)]
struct JsCompileResult {
    ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    go_source: Option<String>,
    diagnostics: Vec<JsDiagnostic>,
}

#[derive(Serialize)]
struct JsCompletionItem {
    label: String,
    kind: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    detail: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    insert_text: Option<String>,
}

// ─── Helper: byte offset → (line, col), both 1-based ────────────────────────
fn offset_to_line_col(source: &str, byte_offset: usize) -> (u32, u32) {
    let clamped = byte_offset.min(source.len());
    let prefix = &source[..clamped];
    let line = (prefix.bytes().filter(|&b| b == b'\n').count() + 1) as u32;
    let col = (prefix
        .rfind('\n')
        .map(|i| clamped - i - 1)
        .unwrap_or(clamped)
        + 1) as u32;
    (line, col)
}

// ─── In-memory Loader ─────────────────────────────────────────────────────────
// The playground only handles a single in-memory file.  The `Loader` trait is
// used to scan folders for sibling modules; we return only the entry file so
// the compiler runs in standalone-file mode.

use lisette_semantics::loader::{Files, Loader};
use rustc_hash::FxHashMap;

struct MemoryLoader {
    filename: String,
    source: String,
}

impl Loader for MemoryLoader {
    fn scan_folder(&self, folder: &str) -> Files {
        // Only return files for the semantic analysis entry module ("_entry_").
        // Any other folder — including Go stdlib identifiers like "go:fmt" — must
        // return an empty map, otherwise the module-graph builder treats the
        // playground source as belonging to those modules and reports false import
        // cycles (e.g. "go:fmt -> go:fmt").
        if folder == "_entry_" {
            let mut map: FxHashMap<String, String> = FxHashMap::default();
            map.insert(self.filename.clone(), self.source.clone());
            map
        } else {
            FxHashMap::default()
        }
    }
}

// ─── Convert LisetteDiagnostic to JsDiagnostic ────────────────────────────────

fn convert_lisette_diag(
    diag: &lisette_diagnostics::LisetteDiagnostic,
    source: &str,
) -> JsDiagnostic {
    let message = diag.plain_message().to_string();
    let severity = if diag.is_error() { "error" } else { "warning" }.to_string();

    let offset = diag.primary_offset();
    let (line, col, end_line, end_col) = {
        let (l, c) = offset_to_line_col(source, offset);
        (l, c, None, None)
    };

    JsDiagnostic {
        severity,
        message,
        line,
        col,
        end_line,
        end_col,
        code: None,
    }
}

fn convert_parse_error(e: &lisette_syntax::ParseError, source: &str) -> JsDiagnostic {
    let message = e.message.clone();
    let code = if e.code.is_empty() { None } else { Some(e.code.clone()) };

    // `labels` is `Vec<(Span, String)>` – first label is the primary one
    let (line, col, end_line, end_col) = if let Some((span, _)) = e.labels.first() {
        let offset = span.byte_offset as usize;
        let len = span.byte_length as usize;
        let (l, c) = offset_to_line_col(source, offset);
        let (el, ec) = offset_to_line_col(source, offset + len);
        (l, c, Some(el), Some(ec))
    } else {
        (1, 1, None, None)
    };

    JsDiagnostic {
        severity: "error".to_string(),
        message,
        line,
        col,
        end_line,
        end_col,
        code,
    }
}

// ─── Core pipeline ────────────────────────────────────────────────────────────

const PLAYGROUND_FILE: &str = "playground.lis";

use lisette_semantics::analyze::{analyze, AnalyzeInput, CompilePhase, SemanticConfig};

fn run_pipeline(
    code: &str,
    phase: CompilePhase,
) -> (Vec<lisette_emit::OutputFile>, Vec<JsDiagnostic>) {
    // 1. Parse
    let ast_result = lisette_syntax::build_ast(code, 0);

    let mut diagnostics: Vec<JsDiagnostic> = ast_result
        .errors
        .iter()
        .map(|e| convert_parse_error(e, code))
        .collect();

    if ast_result.failed() {
        return (vec![], diagnostics);
    }

    // 2. Semantic analysis
    let loader = MemoryLoader {
        filename: PLAYGROUND_FILE.to_string(),
        source: code.to_string(),
    };

    let input = AnalyzeInput {
        config: SemanticConfig {
            run_lints: true,
            standalone_mode: true,
            load_siblings: false,
        },
        loader: &loader,
        source: code.to_string(),
        filename: PLAYGROUND_FILE.to_string(),
        ast: ast_result.ast,
        project_root: None,
        compile_phase: phase.clone(),
    };

    let (sem_result, _facts) = analyze(input);

    for e in &sem_result.errors {
        diagnostics.push(convert_lisette_diag(e, code));
    }
    for w in &sem_result.lints {
        diagnostics.push(convert_lisette_diag(w, code));
    }

    if matches!(phase, CompilePhase::Check) || !sem_result.errors.is_empty() {
        return (vec![], diagnostics);
    }

    // 3. Emit Go
    let emit_input = lisette_diagnostics::SemanticResult::into_emit_input(sem_result);
    let go_files = lisette_emit::Emitter::emit(
        &emit_input,
        "lisette_playground",
        lisette_emit::EmitOptions { debug: false },
    );

    (go_files, diagnostics)
}

// ─── Public WASM API ──────────────────────────────────────────────────────────

/// Format Lisette source. Returns the formatted source, or the original on failure.
#[wasm_bindgen]
pub fn format(code: &str) -> String {
    match lisette_format::format_source(code) {
        Ok(formatted) => formatted,
        Err(_) => code.to_string(),
    }
}

/// Type-check source and return a JSON array of diagnostics.
#[wasm_bindgen]
pub fn check(code: &str) -> String {
    let (_files, diags) = run_pipeline(code, CompilePhase::Check);
    serde_json::to_string(&diags).unwrap_or_else(|_| "[]".to_string())
}

/// Compile Lisette → Go. Returns a JSON object:
/// `{ "ok": bool, "go_source": "...", "diagnostics": [...] }`
#[wasm_bindgen]
pub fn compile(code: &str) -> String {
    let (files, diags) = run_pipeline(code, CompilePhase::Emit);
    let has_errors = diags.iter().any(|d| d.severity == "error");

    let go_source = if !has_errors && !files.is_empty() {
        Some(
            files
                .iter()
                .map(|f| format!("// === {} ===\n{}", f.name, f.to_go()))
                .collect::<Vec<_>>()
                .join("\n\n"),
        )
    } else {
        None
    };

    let result = JsCompileResult {
        ok: !has_errors && go_source.is_some(),
        go_source,
        diagnostics: diags,
    };

    serde_json::to_string(&result).unwrap_or_else(|_| {
        r#"{"ok":false,"diagnostics":[{"severity":"error","message":"Internal error","line":1,"col":1}]}"#.to_string()
    })
}

/// Completion items at byte offset (JSON array).
/// Semantic completions are a TODO; keyword/snippet completions come from the TS layer.
#[wasm_bindgen]
pub fn complete(_code: &str, _offset: u32) -> String {
    let items: Vec<JsCompletionItem> = vec![];
    serde_json::to_string(&items).unwrap_or_else(|_| "[]".to_string())
}

/// Hover info at byte offset. Returns `{ "markdown": "..." }` or empty string.
#[wasm_bindgen]
pub fn hover(_code: &str, _offset: u32) -> String {
    String::new()
}
