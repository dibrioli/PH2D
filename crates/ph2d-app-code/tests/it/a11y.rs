//! A aba entrega a árvore de acessibilidade da vista: o `Document` e um `TextRun` por linha visível.

use crate::Mesa;
use ph2d_a11y::Role;
use ph2d_editor_core::doc_surface::DocumentSurface;

#[test]
fn a_aba_da_um_no_por_linha_visivel() {
    let (mut m, ids) = Mesa::new(&[("bob.luau", "local a = 1\nlocal b = 2\n")]);
    m.pede(ids[0]);
    let antes = m.docs().a11y(ids[0].0);
    assert!(
        antes.iter().all(|(_, n)| n.role() != Role::TextRun),
        "controlo: antes de pintar não há linhas"
    );
    m.pinta(ids[0]);
    let nodes = m.docs().a11y(ids[0].0);
    assert_eq!(nodes[0].1.role(), Role::Document);
    let runs: Vec<_> = nodes
        .iter()
        .filter(|(_, n)| n.role() == Role::TextRun)
        .map(|(_, n)| n.value().unwrap_or_default().to_owned())
        .collect();
    assert_eq!(runs, ["local a = 1", "local b = 2", ""]);
}
