//! Testes de `marker.rs` — arquivo irmao (teto de LOC).

use super::*;
use crate::VertexKind;

/// Uma linha horizontal simples, da esquerda para a direita.
fn line(x0: f64, x1: f64) -> VecPath {
    VecPath {
        verts: vec![VecVertex::corner([x0, 0.0]), VecVertex::corner([x1, 0.0])],
        closed: false,
        ..VecPath::default()
    }
}

/// **A ponta olha para FORA.** É a asserção que separa uma seta de uma seta ao contrário —
/// e ela é fácil de errar por um sinal, exatamente como o espelhamento vertical foi.
#[test]
fn the_arrowhead_points_away_from_the_line_not_into_it() {
    let l = line(0.0, 10.0);

    let (tip, dir) = end_tangent(&l, false).expect("a linha tem fim");
    assert_eq!(tip, [10.0, 0.0], "o fim da linha e a ponta");
    assert!(
        (dir[0] - 1.0).abs() < 1e-9 && dir[1].abs() < 1e-9,
        "no FIM, a tangente aponta para +x (para fora): {dir:?}"
    );

    let (tip0, dir0) = end_tangent(&l, true).expect("a linha tem comeco");
    assert_eq!(tip0, [0.0, 0.0]);
    assert!(
        (dir0[0] + 1.0).abs() < 1e-9,
        "no COMECO, a tangente aponta para -x (tambem para fora): {dir0:?}"
    );

    // E a geometria segue a direção: o triângulo do fim tem o bico no extremo, e o corpo
    // dele fica ATRÁS (do lado da linha), nunca à frente.
    let t = Marker::Triangle
        .build(tip, dir, 1.0)
        .expect("o triangulo existe");
    let apex = t.verts[0].anchor;
    assert_eq!(apex, tip, "o bico do triangulo E a ponta da linha");
    for v in &t.verts[1..] {
        assert!(
            v.anchor[0] < tip[0],
            "a base do triangulo tem de ficar ATRAS do bico: {:?}",
            v.anchor
        );
    }
}

/// **A ponta escala com a largura do traço** (o `markerUnits = "strokeWidth"` do SVG). Sem
/// isso, uma linha grossa terminaria num alfinete.
#[test]
fn the_marker_scales_with_the_stroke_width() {
    let tip = [0.0, 0.0];
    let dir = [1.0, 0.0];
    let thin = Marker::Triangle.build(tip, dir, 1.0).expect("fino");
    let fat = Marker::Triangle.build(tip, dir, 3.0).expect("grosso");

    let span = |p: &VecPath| {
        let ys: Vec<f64> = p.verts.iter().map(|v| v.anchor[1]).collect();
        ys.iter().copied().fold(f64::MIN, f64::max) - ys.iter().copied().fold(f64::MAX, f64::min)
    };
    assert!(
        (span(&fat) - 3.0 * span(&thin)).abs() < 1e-9,
        "triplicar a largura do traco tem de triplicar a ponta: {} vs {}",
        span(&fat),
        span(&thin)
    );
}

/// **A linha recua para caber a ponta.** Sem o recuo, o traço aparece por dentro de uma
/// ponta vazada e faz uma bolha na base de uma cheia.
#[test]
fn the_line_is_shortened_to_make_room_for_the_head() {
    let l = line(0.0, 10.0);
    let inset = Marker::Triangle.inset(); // em múltiplos da largura
    let w = 1.0;
    let t = trim_path(&l, 0.0, inset * w).expect("sobra linha");
    let end = t.verts.last().expect("tem vertices").anchor;
    assert!(
        (end[0] - (10.0 - inset)).abs() < 1e-9,
        "a linha tinha de parar em {}, parou em {}",
        10.0 - inset,
        end[0]
    );
    // O começo não se mexeu.
    assert_eq!(t.verts[0].anchor, [0.0, 0.0]);
}

/// As pontas SEM região (a aberta e a barra) não recuam nada: não há nada para esconder o
/// traço, e recuar deixaria um vão visível entre a linha e a ponta.
#[test]
fn open_heads_do_not_shorten_the_line() {
    assert_eq!(Marker::Open.inset(), 0.0);
    assert_eq!(Marker::Bar.inset(), 0.0);
    assert_eq!(Marker::None.inset(), 0.0);
    // Já as que fecham região recuam — e o losango, que tem o dobro do comprimento, recua o
    // dobro do triângulo.
    assert!(Marker::Triangle.inset() > 0.0);
    assert!(
        (Marker::Diamond.inset() - 2.0 * Marker::Triangle.inset()).abs() < 1e-9,
        "o losango tem o dobro do comprimento, logo o dobro do recuo"
    );
}

/// Uma linha **mais curta que os recuos** não vira uma linha invertida: vira nenhuma linha.
/// (O caso do usuário que arrasta um conector de 2 px com uma seta gorda.)
#[test]
fn a_line_shorter_than_its_heads_yields_no_line_instead_of_an_inverted_one() {
    let tiny = line(0.0, 1.0);
    assert!(
        trim_path(&tiny, 4.0, 4.0).is_none(),
        "recuar 8 de uma linha de 1 tem de dar NENHUMA linha, nao uma linha ao contrario"
    );
    // E uma que sobra, sobra de verdade.
    assert!(trim_path(&line(0.0, 20.0), 4.0, 4.0).is_some());
}

/// A ponta redonda é um CÍRCULO (quatro cúbicas exatas), não um polígono — a mesma regra do
/// resto do catálogo.
#[test]
fn the_round_head_is_a_real_circle() {
    let c = Marker::Circle
        .build([0.0, 0.0], [1.0, 0.0], 1.0)
        .expect("o circulo existe");
    assert_eq!(c.verts.len(), 4, "quatro cubicas de 90 graus");
    assert!(c.verts.iter().all(|v| v.kind != VertexKind::Corner));
    // Ele encosta na ponta da linha (tangente ao extremo), não a ultrapassa.
    let max_x = c.verts.iter().map(|v| v.anchor[0]).fold(f64::MIN, f64::max);
    assert!(max_x <= 1e-9, "o circulo nao passa da ponta: {max_x}");
}

/// **A ponta segue a CURVA, não a corda.** Numa curva bem encurvada as duas divergem
/// visivelmente, e uma seta alinhada pela corda sai torta.
#[test]
fn the_head_follows_the_curves_tangent_not_the_chord() {
    // Um quarto de círculo: começa em (0,0) indo para +y, termina em (1,1) indo para +x.
    let path = VecPath {
        verts: vec![
            VecVertex::smooth([0.0, 0.0], [0.0, 0.0], [0.0, 0.55]),
            VecVertex::smooth([1.0, 1.0], [0.45, 1.0], [1.0, 1.0]),
        ],
        closed: false,
        ..VecPath::default()
    };
    let (_, dir) = end_tangent(&path, false).expect("tem fim");
    // A tangente no fim é ~(+1, 0). A CORDA seria ~(0.707, 0.707) — 45° de erro.
    assert!(
        dir[0] > 0.95 && dir[1].abs() < 0.2,
        "a ponta tem de seguir a tangente (~+x), nao a corda (~45 graus): {dir:?}"
    );
}

/// Todo discriminante sobrevive ao round-trip pelo documento, e um desconhecido (save de uma
/// versão futura) resolve para `None` em vez de entrar em pânico.
#[test]
fn every_marker_round_trips_and_an_unknown_one_degrades_gracefully() {
    for &m in ALL_MARKERS {
        assert_eq!(Marker::from_u8(m.as_u8()), Some(m));
    }
    assert_eq!(
        Marker::from_u8(200),
        None,
        "uma ponta do futuro nao entra em panico"
    );
}
