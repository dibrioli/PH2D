//! Shared helpers of the gates: the pinned binary (FAILS loudly when absent), PH2D's definitions,
//! the canonical `bob.luau`, and one-shot "open and read the diagnostics".

use std::path::{Path, PathBuf};
use std::time::Duration;

use ph2d_code_lsp::{Diagnostic, LspClient, ServerConfig, Solver, resolve_binary};

pub const WAIT: Duration = Duration::from_secs(30);

/// The luau-lsp binary; never skips — a missing binary is a red gate with the cure in the message.
pub fn binary() -> PathBuf {
    match resolve_binary() {
        Ok((p, _)) => p,
        Err(e) => panic!("⛔ {e}\n   This gate needs the pinned luau-lsp: run `bash scripts/luau-lsp-fetch.sh`."),
    }
}

pub fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repo root")
}

pub fn ph2d_definitions() -> PathBuf {
    repo_root().join("runtime/luau/ph2d.d.luau")
}

pub fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/it/fixtures").join(name)
}

/// `bob.luau` read from its single source, `BOB_LUAU` in the components smoke — no copy to drift.
pub fn bob_luau() -> String {
    let path = repo_root().join("crates/ph2d-app-components/src/script_smoke.rs");
    let src = std::fs::read_to_string(&path).expect("script_smoke.rs");
    let open = "pub const BOB_LUAU: &str = r#\"";
    assert_eq!(src.matches(open).count(), 1, "BOB_LUAU moved out of {}", path.display());
    let start = src.find(open).unwrap() + open.len();
    let len = src[start..].find("\"#;").expect("end of BOB_LUAU");
    src[start..start + len].to_owned()
}

pub fn start(definitions: &Path, solver: Solver) -> LspClient {
    let mut cfg = ServerConfig::new(binary()).with_definitions("@ph2d", definitions.to_path_buf());
    cfg.solver = solver;
    LspClient::start(cfg).expect("luau-lsp starts")
}

pub fn uri(name: &str) -> String {
    format!("file:///ph2d-gate/{name}")
}

/// Opens `text` as `name` and returns the first diagnostics published for it.
pub fn diagnostics_of(c: &mut LspClient, name: &str, text: &str) -> Vec<Diagnostic> {
    let u = uri(name);
    c.pump();
    c.did_open(&u, text).expect("didOpen");
    c.wait_diagnostics(&u, WAIT).expect("diagnostics arrive").diagnostics
}

pub fn render(diags: &[Diagnostic]) -> String {
    diags
        .iter()
        .map(|d| {
            format!(
                "\n   line {} [{}] {}",
                d.range.start.line + 1,
                d.code.as_deref().unwrap_or("-"),
                d.message
            )
        })
        .collect()
}

/// The words luau-lsp uses when it drops a definitions file — it keeps running without them.
pub const REJECTION: [&str; 2] = ["Failed to read definitions file", "Extended types will not be provided"];

/// Every trace that the definitions file `file_name` was rejected: server messages, stderr, and
/// diagnostics published against the file itself.
pub fn rejection_evidence(c: &mut LspClient, file_name: &str) -> Vec<String> {
    c.pump();
    let mut out: Vec<String> = c
        .messages()
        .iter()
        .map(|m| m.text.clone())
        .chain(c.stderr_tail())
        .filter(|t| REJECTION.iter().any(|r| t.contains(r)))
        .collect();
    let defs_uri_suffix = format!("/{file_name}");
    for d in c.all_diagnostics() {
        if d.uri.ends_with(&defs_uri_suffix) && !d.diagnostics.is_empty() {
            out.push(format!("diagnostics on {}:{}", d.uri, render(&d.diagnostics)));
        }
    }
    out
}
