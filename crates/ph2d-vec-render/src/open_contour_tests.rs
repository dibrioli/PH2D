//! Gates do CONTORNO ABERTO no encode do vetor — o que um caminho que não fecha
//! desenha, e o que ele não pode desenhar.
//!
//! ⚠️ **Este arquivo existe por um TETO DE LOC** (HR-18, 700 para `crates/`), e o
//! corte é o que esta crate já usa nos quatro irmãos ao lado (`lib_tests.rs`,
//! `encode_cost_tests.rs`, `stroke_zero_tests.rs`, `standalone_tests.rs`): a LEI no
//! `lib.rs`, as PROVAS num arquivo por assunto.

use super::*;
use ph2d_vector::Shape;

/// **O triângulo escuro do cubo, executável.**
///
/// O cubo isométrico é uma silhueta hexagonal FECHADA + três arestas internas, que são
/// contornos ABERTOS. Preencher tudo junto faz o rasterizador **fechar cada contorno
/// aberto implicitamente** (é a semântica de fill, em qualquer engine): a corda que
/// fecha a polilinha `V1 → M → V3` vira uma região de winding próprio que, com
/// `NonZero`, CANCELA o hexágono onde coincide. O Enio fotografou exatamente isso — um
/// triângulo escuro comendo metade da face direita.
///
/// O teste mede o winding NO PONTO do triângulo. Ele prova as duas metades da história:
/// o path do preenchimento cobre a face (winding ≠ 0) **e** o path completo — que era o
/// que se preenchia antes — a perfura (winding = 0). Se alguém reverter a correção, a
/// segunda asserção passa a valer para o primeiro path e o teste cai.
#[test]
fn an_open_contour_never_punches_a_hole_in_the_fill() {
    let cube = ph2d_vec_scene::iso_cube([-1.0, -1.0], [1.0, 1.0], 0.5, 0.5, false);
    // As três arestas internas vivem no sub-contorno; o vértice central é o do meio.
    let inner = &cube.subpaths[0].verts;
    let (v1, m, v3) = (inner[0].anchor, inner[1].anchor, inner[2].anchor);
    // Um ponto BEM dentro do triângulo V1–M–V3: o baricentro. É o miolo da mancha.
    let p = Point::new((v1[0] + m[0] + v3[0]) / 3.0, (v1[1] + m[1] + v3[1]) / 3.0);

    let fill = build_fill_bezpath(&cube);
    assert_ne!(
        fill.winding(p),
        0,
        "a face direita do cubo tem de ser PREENCHIDA em {p:?} — o triangulo escuro voltou"
    );

    // E a prova de que o bug era real: com os contornos abertos dentro do preenchimento,
    // o mesmo ponto FICA DE FORA. (É o que se pintava antes.)
    let everything = build_bezpath(&cube);
    assert_eq!(
        everything.winding(p),
        0,
        "o teste perdeu o poder de discriminar: o contorno aberto deveria furar o fill"
    );

    // As linhas de construção são exatamente as arestas internas — e não estão no fill.
    assert!(
        !build_lines_bezpath(&cube).is_empty(),
        "as arestas internas do cubo TEM de ser desenhadas (senao e um hexagono)"
    );
}

/// A regra vale para toda forma do catálogo: o path do preenchimento nunca contém um
/// contorno aberto, e a soma dos dois caminhos é o path inteiro.
#[test]
fn fill_and_lines_partition_every_shape_in_the_catalogue() {
    for &kind in ph2d_vec_scene::ALL_SHAPES {
        let path = ph2d_vec_scene::cook(kind, [-1.0, -1.0], [1.0, 1.0], &kind.defaults());
        let (fill, lines, whole) = (
            build_fill_bezpath(&path),
            build_lines_bezpath(&path),
            build_bezpath(&path),
        );
        assert_eq!(
            fill.elements().len() + lines.elements().len(),
            whole.elements().len(),
            "{kind:?}: fill + linhas tem de particionar o path inteiro"
        );
        // Uma forma ABERTA (linha, arco, espiral, chave) não tem nada a preencher.
        if !kind.is_closed() {
            assert!(
                fill.is_empty(),
                "{kind:?} e aberta — nao tem interior para preencher"
            );
        }
    }
}
