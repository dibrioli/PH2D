//! Arch-gate: the substrate's contracts that *external node crates implement*
//! must stay tiny. Every node crate implements `NodeOp`; the registry
//! implements `OpResolver`. If these grow, the change ripples to every node in
//! the engine and re-serializes the multi-agent fan-out (the failure mode the
//! whole node-centric plan is built to avoid — ADR-0030/0031/0032).
//!
//! Mirrors `ph2d-editor-core`'s `architecture_panel_host_surface`. To raise a
//! cap, bump the number here *and justify it in review* — a contract change is
//! a rare, Coordenador-only event (the freeze discipline).
//!
//! **FROZEN at W2.T4 (ADR-0039, 2026-05-22):** the caps below are pinned to the
//! *current* surface (no headroom), so ANY addition to the node-implemented
//! contract now trips this gate and forces a conscious cap bump + ADR. This is
//! what makes the freeze bite — the fan-out builds against a fixed contract.

/// Count `fn ` declarations inside the body of `trait_decl` (up to its first
/// closing `\n}`). Method signatures end in `;`, so the first `\n}` after the
/// declaration is the trait's own close.
fn trait_method_count(src: &str, trait_decl: &str) -> usize {
    let start = src.find(trait_decl).expect("trait declaration present");
    let rest = &src[start..];
    let end = rest.find("\n}").expect("trait body closes");
    rest[..end].matches("fn ").count()
}

#[test]
fn nodeop_contract_is_capped() {
    let src = include_str!("../src/node.rs");
    let n = trait_method_count(src, "pub trait NodeOp");
    assert!(
        n <= 2,
        "NodeOp has {n} methods; cap is 2. FROZEN at W2.T4 (ADR-0039) to the \
         current surface — every node crate implements this, so any growth \
         ripples the whole fan-out. Adding a method is a Coordenador-only \
         contract change: bump the cap here + write the ADR (ADR-0031)."
    );
}

#[test]
fn opresolver_contract_is_capped() {
    let src = include_str!("../src/cook.rs");
    let n = trait_method_count(src, "pub trait OpResolver");
    assert!(
        n <= 1,
        "OpResolver has {n} methods; cap is 1. FROZEN at W2.T4 (ADR-0039) to the \
         current surface — the registry implements this. Adding a method is a \
         Coordenador-only contract change: bump the cap here + write the ADR \
         (ADR-0032)."
    );
}

#[test]
fn nodemanifest_field_count_is_capped() {
    // Every node crate writes a `NodeManifest { .. }` literal (no `..Default`
    // in const context), so adding a field breaks them ALL — the fan-out
    // re-serialization this gate guards (audit M1, ADR-0032).
    let src = include_str!("../src/node.rs");
    let start = src
        .find("pub struct NodeManifest {")
        .expect("struct present");
    let body_start = src[start..].find('{').expect("opening brace") + start;
    let body_end = src[body_start..].find("\n}").expect("struct closes") + body_start;
    let n = src[body_start..body_end].matches("pub ").count();
    assert!(
        n <= 8,
        "NodeManifest has {n} pub fields; cap is 8. FROZEN at W2.T4 (ADR-0039) \
         to the current surface — adding a field breaks every node crate's \
         `MANIFEST {{ .. }}` literal at once. A Coordenador-only contract change: \
         bump the cap here + write the ADR (ADR-0032)."
    );
}
