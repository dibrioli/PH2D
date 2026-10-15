//! Gate (a): PH2D's `ph2d.d.luau` is ACCEPTED by the pinned luau-lsp. The server never fails on a bad
//! definitions file — it says «Failed to read definitions file … Extended types will not be provided»
//! in a `showMessage` and carries on without our API — so the gate reads every channel for those
//! words, and its control proves it sees them on a file luau-lsp 1.70.1 rejects.

use ph2d_code_lsp::{Position, Solver, pinned_version};

use crate::support::{diagnostics_of, fixture, ph2d_definitions, rejection_evidence, start, uri};

#[test]
fn the_ph2d_definitions_load_without_the_silent_rejection() {
    let defs = ph2d_definitions();
    let mut c = start(&defs, Solver::Old);
    // The definitions load after `workspace/configuration`; a checked document means they were read.
    let text = "local _api = ph2d.\n";
    diagnostics_of(&mut c, "toca.luau", text);
    let evidence = rejection_evidence(&mut c, "ph2d.d.luau");
    assert!(
        evidence.is_empty(),
        "luau-lsp {} REJECTED {} — the editor would run with no PH2D API:\n{}",
        pinned_version(),
        defs.display(),
        evidence.join("\n")
    );
    // Accepted AND applied: completion after `ph2d.` offers the API.
    let items = c.completion(&uri("toca.luau"), Position::new(0, 18)).expect("completion");
    assert!(!items.is_empty(), "no completion after `ph2d.` — the definitions did not reach the checker");
    c.shutdown().expect("shutdown");
}

/// Control: the detector above is not blind — a `declare class` file is rejected and seen.
#[test]
fn a_rejected_definitions_file_is_seen_by_the_gate() {
    let defs = fixture("definicoes_rejeitadas.d.luau");
    let mut c = start(&defs, Solver::Old);
    diagnostics_of(&mut c, "toca.luau", "local _x = 1\n");
    let evidence = rejection_evidence(&mut c, "definicoes_rejeitadas.d.luau");
    assert!(!evidence.is_empty(), "the rejection of a `declare class` file went unseen");
    c.shutdown().expect("shutdown");
}

/// The binary resolved is the version the manifest pins (a stale cache or a wrong `PH2D_LUAU_LSP`
/// would silently test another luau-lsp).
#[test]
fn the_resolved_binary_is_the_pinned_version() {
    let c = start(&ph2d_definitions(), Solver::Old);
    let reported = c.server_info().get("version").and_then(|v| v.as_str()).unwrap_or_default().to_owned();
    assert_eq!(reported, pinned_version(), "serverInfo = {}", c.server_info());
    c.shutdown().expect("shutdown");
}
