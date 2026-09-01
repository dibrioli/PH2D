//! Gates do **BALDE** (plano 40) — a lei pura, sem cena e sem ponteiro.

use super::*;
use ph2d_vec_scene::VertexKind;

fn v(x: f64, y: f64) -> VecVertex {
    VecVertex {
        anchor: [x, y],
        in_handle: [x, y],
        out_handle: [x, y],
        kind: VertexKind::Corner,
        corner_radius: 0.0,
    }
}

/// Um quadrado FECHADO de lado `l` centrado na origem.
fn quadrado(l: f64) -> (Vec<VecVertex>, bool) {
    let h = l * 0.5;
    (vec![v(-h, -h), v(h, -h), v(h, h), v(-h, h)], true)
}

/// ⭐ **Um anel sozinho tem UMA face limitada**, e o clique lá dentro acha-a.
///
/// ⚠️ Ele não tem cruzamento nenhum, então só existe porque a rede o transforma num **laço** — sem
/// isso um contorno fechado não tem meia-aresta e a face dele seria invisível ao passeio.
#[test]
fn a_lone_ring_has_one_bounded_face_and_the_click_finds_it() {
    let r = rede(&[quadrado(100.0)]);
    assert_eq!(r.arcos.len(), 1, "o anel tem de virar UM arco-laco");
    assert_eq!(
        r.arcos[0].de, r.arcos[0].ate,
        "e as duas pontas sao o MESMO no'"
    );
    let f = r.face_em([0.0, 0.0]).expect("o centro esta' dentro");
    assert!((f.area - 10_000.0).abs() < 1.0, "area {}", f.area);
    assert!(r.face_em([500.0, 500.0]).is_none(), "fora nao e' face");
}

/// ⭐⭐⭐ **O PEDIDO: quatro linhas SOLTAS que se cruzam fecham um quadrado.**
///
/// Nenhuma delas é fechada, nenhuma tem dentro — é exactamente o caso que o arranjo do Shape
/// Builder não sabe exprimir (`região(M) = ∩M − ∪¬M` não se define para um traço aberto).
#[test]
fn four_open_lines_that_cross_enclose_a_square() {
    let a = 60.0;
    let contornos = vec![
        (vec![v(-a, -20.0), v(a, -20.0)], false),
        (vec![v(-a, 20.0), v(a, 20.0)], false),
        (vec![v(-20.0, -a), v(-20.0, a)], false),
        (vec![v(20.0, -a), v(20.0, a)], false),
    ];
    let r = rede(&contornos);
    let f = r
        .face_em([0.0, 0.0])
        .expect("o miolo das quatro linhas e' uma face");
    assert!(
        (f.area - 1600.0).abs() < 1.0,
        "o miolo e' 40x40: {}",
        f.area
    );
    assert_eq!(f.arcos.len(), 4, "a fronteira sao QUATRO arcos inteiros");
    // E a geometria sai com um vértice por canto — não um polígono achatado.
    let g = r.geometria(&f);
    assert_eq!(g.len(), 4, "quatro cantos: {g:?}");
}

/// ⚠️ **A de MENOR área**, e é ela que resolve o aninhamento.
#[test]
fn the_click_takes_the_innermost_face() {
    let r = rede(&[quadrado(100.0), quadrado(40.0)]);
    let f = r
        .face_em([0.0, 0.0])
        .expect("o centro esta' dentro dos dois");
    assert!(
        (f.area - 1600.0).abs() < 1.0,
        "o clique tem de apanhar o quadrado de DENTRO: {}",
        f.area
    );
    // E entre os dois quadrados a face é o anel — área = 10 000 − 1 600.
    let anel = r.face_em([-45.0, 0.0]).expect("entre os dois ha' face");
    assert!(
        anel.area > 1600.0,
        "a face entre os dois anéis nao pode ser a de dentro: {}",
        anel.area
    );
}

/// ⛔ **Fora de tudo não é face** — e a recusa é a resposta certa, não um erro.
#[test]
fn a_click_outside_everything_is_not_a_face() {
    let r = rede(&[quadrado(100.0)]);
    assert!(r.face_em([80.0, 80.0]).is_none());
}

/// ⚠️ **A face de FORA tem área negativa**, e é isso que a mantém fora da escolha sem uma regra à
/// parte. Sem esta metade, *"a de menor área"* escolheria a face errada num documento inteiro.
#[test]
fn the_outer_face_comes_out_negative() {
    let r = rede(&[quadrado(100.0)]);
    let faces = r.faces();
    assert_eq!(faces.len(), 2, "um laco da' duas faces: dentro e fora");
    assert!(
        faces.iter().any(|f| f.area < 0.0),
        "nenhuma face saiu negativa: {:?}",
        faces.iter().map(|f| f.area).collect::<Vec<_>>()
    );
}

/// ⭐⭐ **A CURVA sobrevive**: a fronteira é feita dos arcos, então as alças chegam à forma.
///
/// ⚠️ É o que separa este balde do do Inkscape (que traça pixels) e do do Flip (que devolve
/// polígono): num círculo cortado em dois arcos, a geometria da face tem de trazer alças
/// diferentes da âncora.
#[test]
fn the_filled_shape_keeps_the_curve_not_a_polygon() {
    let c = ph2d_vec_scene::ellipse([0.0, 0.0], 50.0, 50.0);
    let linha = (vec![v(-80.0, 0.0), v(80.0, 0.0)], false);
    let r = rede(&[(c.verts.clone(), true), linha]);
    let f = r.face_em([0.0, 20.0]).expect("a metade de cima do circulo");
    let g = r.geometria(&f);
    assert!(
        g.iter().any(|v| v.out_handle != v.anchor),
        "a forma saiu sem alcas — isto e' um poligono, nao a curva"
    );
    // Meia bola de raio 50: ~3927. A recta corta-a ao meio.
    assert!(
        (f.area - 3927.0).abs() < 60.0,
        "a metade de cima do circulo mede {}",
        f.area
    );
}
