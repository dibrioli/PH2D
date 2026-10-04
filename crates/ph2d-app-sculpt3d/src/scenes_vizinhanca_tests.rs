//! A cena `=54` tem o FENÓMENO: a risca atravessa arestas da malha (amostras de
//! aresta dos dois lados dela), e o degrau é o que o roteiro diz.

use super::{NIVEL, cor_da_base};

/// ⭐ **A risca cruza arestas da malha, e a peça tem os dois tons.** Uma risca
/// que caísse só no meio das faces não mostraria a costura que a cena ensina a
/// procurar.
#[test]
fn a_risca_da_cena_atravessa_arestas_da_malha() {
    let mesh = crate::scenes::tinta_fina::peca();
    let faces = || mesh.faces().iter().map(ph2d_mesh::Face::verts);
    let tinta = ph2d_mesh_colors::Tinta::nova(mesh.vert_count(), faces(), NIVEL);
    let xs = crate::vizinhanca_da_peca::posicoes(&tinta, &mesh);
    let fronteira = tinta.topologia().verts() + tinta.topologia().arestas_amostras() as usize;
    let escuras = |r: std::ops::Range<usize>| r.filter(|&i| cor_da_base(xs[i])[0] < 100).count();
    let (na_aresta, no_meio) = (escuras(0..fronteira), escuras(fronteira..xs.len()));
    assert!(
        na_aresta > 200,
        "a risca quase não toca arestas ({na_aresta})"
    );
    assert!(
        no_meio > 2_000,
        "a risca quase não tem meio de face ({no_meio})"
    );
    assert!(
        na_aresta + no_meio < xs.len() / 4,
        "a risca tem de ser uma RISCA, não a bola"
    );
    assert_eq!(1u32 << NIVEL, 16, "o roteiro diz `16x`");
}
