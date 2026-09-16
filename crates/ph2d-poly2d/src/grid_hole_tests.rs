//! ⭐ **UM BURACO NA ARTE É UM BURACO NA MALHA** — irmão do [`super`] (a grelha graduada).
//!
//! ⚠️ A fila do esqueleto dizia, desde a 2.ª mídia (2026-09-09), *«um buraco no meio de uma forma não
//! é traçado, e a malha cobre-o»* — e isso era verdade para o **contorno** triangulado
//! (`crate::mesh_of`), que só traça o anel EXTERIOR. A grelha da F6-b (2026-09-10) decide **célula a
//! célula** se há tinta, então um buraco maior que uma célula fica de fora por construção — e nenhum
//! gate o dizia. *Uma ausência que ninguém mede é uma nota que envelhece dos dois lados.*

use super::*;

/// Um quadrado de `120 px` com um furo quadrado de `60 px` no meio — um anel de `30 px` de largura.
fn anel() -> (Vec<u8>, u32, u32) {
    let (w, h) = (120usize, 120usize);
    let mut a = vec![255u8; w * h];
    for y in 30..90 {
        for x in 30..90 {
            a[y * w + x] = 0;
        }
    }
    (a, w as u32, h as u32)
}

/// ⭐ **Nenhum triângulo nasce dentro do furo** — e a malha cerca-o pelos quatro lados.
///
/// A margem é a maior aresta da malha mais a folga da cobertura: uma célula que toca a borda do furo
/// é legítima (ela tem tinta), e o que o gate proíbe é uma célula INTEIRA de vazio.
///
/// (Mutação: a cobertura responder sempre `true` ⇒ RED.)
#[test]
fn a_hole_in_the_art_is_a_hole_in_the_mesh() {
    let (a, w, h) = anel();
    let opts = GridOptions::default();
    let m = grid_mesh_of(&a, w, h, &[], opts).expect("ha' tinta");
    let aresta = m
        .tris
        .iter()
        .flat_map(|t| [(t[0], t[1]), (t[1], t[2]), (t[2], t[0])])
        .map(|(i, j)| {
            let (p, q) = (m.rest[i as usize], m.rest[j as usize]);
            (p[0] - q[0]).hypot(p[1] - q[1])
        })
        .fold(0.0_f64, f64::max);
    let margem = aresta + opts.expand;
    assert!(
        60.0 > 2.0 * margem,
        "a fixtura nao contem o fenomeno: o furo (60 px) nao cabe duas celulas ({margem:.1} px)"
    );
    let (lo, hi) = (30.0 + margem, 90.0 - margem);
    let mut lados = [false; 4];
    for t in &m.tris {
        let c = t.iter().fold([0.0_f64; 2], |c, &i| {
            let p = m.rest[i as usize];
            [c[0] + p[0] / 3.0, c[1] + p[1] / 3.0]
        });
        assert!(
            !(c[0] > lo && c[0] < hi && c[1] > lo && c[1] < hi),
            "um triangulo nasceu dentro do furo, em {c:?}"
        );
        lados[0] |= c[1] < 30.0;
        lados[1] |= c[1] > 90.0;
        lados[2] |= c[0] < 30.0;
        lados[3] |= c[0] > 90.0;
    }
    assert_eq!(
        lados, [true; 4],
        "a malha nao cerca o furo pelos quatro lados"
    );
}
