//! Arch-gate: the substrate's contracts that *external node crates implement*
//! must stay tiny. Every node crate implements `NodeOp`; the registry
//! implements `OpResolver`. If these grow, the change ripples to every node in
//! the engine and re-serializes the multi-agent fan-out (the failure mode the
//! whole node-centric plan is built to avoid — ADR-0030/0031/0032).
//!
//! Mirrors `ph2d-editor-core`'s `architecture_panel_host_surface`. To raise a
//! cap, bump the number here *and justify it in review* — a contract change is
//! a rare, Coordenador-only event (the freeze discipline).

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
        n <= 4,
        "NodeOp has {n} methods; cap is 4. Every node crate implements this — \
         keep it tiny. Justify a bump in review (ADR-0031)."
    );
}

#[test]
fn opresolver_contract_is_capped() {
    let src = include_str!("../src/cook.rs");
    let n = trait_method_count(src, "pub trait OpResolver");
    assert!(
        n <= 2,
        "OpResolver has {n} methods; cap is 2. The registry implements this — \
         keep it tiny. Justify a bump in review (ADR-0032)."
    );
}
