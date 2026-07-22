//! Gates do [`ArcPath`] — a porta única de *"onde fica o arco `s` neste contorno?"*.
//!
//! O oráculo é sempre **geométrico** (o círculo de raio conhecido, o comprimento medido), nunca um
//! espelho da fórmula: um teste que recomputasse `inv_arclen` para saber o que esperar seria
//! sempre verde.

use super::ArcPath;
use crate::{VecVertex, VertexKind};

const R: f64 = 60.0;

/// Um círculo em quatro cúbicas — perímetro conhecido: `2πR`.
fn circle() -> Vec<VecVertex> {
    const K: f64 = 0.552_284_749_830_793_4;
    let p = [[R, 0.0], [0.0, R], [-R, 0.0], [0.0, -R]];
    let tang = [[0.0, K * R], [-K * R, 0.0], [0.0, -K * R], [K * R, 0.0]];
    (0..4)
        .map(|i| VecVertex {
            anchor: p[i],
            in_handle: [p[i][0] - tang[i][0], p[i][1] - tang[i][1]],
            out_handle: [p[i][0] + tang[i][0], p[i][1] + tang[i][1]],
            kind: VertexKind::Smooth,
            corner_radius: 0.0,
        })
        .collect()
}

/// Uma linha reta de A a B, aberta.
fn segment_of(a: [f64; 2], b: [f64; 2]) -> Vec<VecVertex> {
    vec![VecVertex::corner(a), VecVertex::corner(b)]
}

/// Menos de dois vértices não tem segmento — e devolver um `ArcPath` vazio faria o
/// `frame_at` indexar `segs.len() - 1` com `len == 0`.
#[test]
fn a_contour_without_a_segment_has_no_arc_path() {
    assert!(ArcPath::from_contour(&[], true).is_none());
    assert!(ArcPath::from_contour(&[VecVertex::corner([1.0, 2.0])], true).is_none());
}

/// O total é o COMPRIMENTO, medido contra uma figura cujo perímetro se sabe de fora do código.
///
/// ⚠️ **A tolerância é 3e-4 e o resíduo NÃO é do integrador.** Medido: `377,0440` contra
/// `2πR = 376,9911`, ou **1,40e-4 relativo** — que é a assinatura conhecida do círculo aproximado
/// por quatro cúbicas (`K = 0.5522847…` minimiza o erro RADIAL, e o perímetro do polígono de
/// Bézier fica um bocadinho LONGO). O controle que separa as duas explicações é o gate da reta
/// abaixo, onde o mesmo integrador acerta `100.0` a **1e-9**: se o erro fosse da quadratura de
/// Gauss-Legendre, ele apareceria lá também.
#[test]
fn the_total_is_the_arc_length_of_the_whole_contour() {
    let ap = ArcPath::from_contour(&circle(), true).expect("círculo");
    let expected = 2.0 * std::f64::consts::PI * R;
    assert!(
        (ap.total() - expected).abs() < expected * 3e-4,
        "perímetro {} contra 2piR {expected}",
        ap.total()
    );
}

/// **Fechado tem um segmento a mais que aberto** — o que volta ao começo. Sem isto, um contorno
/// fechado perderia a última aresta e todo consumidor mediria um caminho mais curto do que o
/// desenhado.
#[test]
fn a_closed_contour_carries_the_segment_that_returns_to_the_start() {
    let v = circle();
    let closed = ArcPath::from_contour(&v, true).expect("fechado");
    let open = ArcPath::from_contour(&v, false).expect("aberto");
    assert_eq!(closed.anchor_arcs().len(), 4, "4 segmentos no fechado");
    assert_eq!(open.anchor_arcs().len(), 3, "3 no aberto");
    assert!(
        closed.total() > open.total(),
        "o fechado tem de ser mais longo ({} vs {})",
        closed.total(),
        open.total()
    );
}

/// Percorrer por arco é percorrer por DISTÂNCIA: a meio de uma reta de 100 está o ponto a 50, e a
/// tangente aponta ao longo dela. É o invariante inteiro do texto em caminho.
#[test]
fn walking_half_the_arc_of_a_straight_line_lands_at_its_middle() {
    let ap = ArcPath::from_contour(&segment_of([0.0, 0.0], [100.0, 0.0]), false).expect("reta");
    assert!(
        (ap.total() - 100.0).abs() < 1e-9,
        "comprimento {}",
        ap.total()
    );
    let (p, t) = ap.frame_at(50.0);
    assert!(
        (p[0] - 50.0).abs() < 1e-6 && p[1].abs() < 1e-9,
        "ponto {p:?}"
    );
    assert!(
        (t[0] - 1.0).abs() < 1e-9 && t[1].abs() < 1e-9,
        "tangente {t:?}"
    );
}

/// **As âncoras SÃO posições de arco.** Consultar o arco de uma âncora tem de devolver a âncora —
/// é o que permite a um efeito amostrar exatamente onde o caminho já tem vértice, sem aliasing.
#[test]
fn each_anchor_arc_resolves_back_to_its_own_anchor() {
    let v = circle();
    let ap = ArcPath::from_contour(&v, true).expect("círculo");
    for (i, &s) in ap.anchor_arcs().iter().enumerate() {
        let (p, _) = ap.frame_at(s);
        let d = (p[0] - v[i].anchor[0]).hypot(p[1] - v[i].anchor[1]);
        assert!(d < 1e-6, "âncora {i}: arco {s} caiu a {d} dela");
    }
}

/// **Um `s` fora do intervalo satura**, não entra em pânico nem devolve lixo. Quem varre uma
/// grade encosta no fim por arredondamento, e negativo é a mesma pergunta do outro lado.
#[test]
fn an_out_of_range_arc_saturates_at_the_ends() {
    let ap = ArcPath::from_contour(&segment_of([0.0, 0.0], [10.0, 0.0]), false).expect("reta");
    let (before, _) = ap.frame_at(-5.0);
    let (after, _) = ap.frame_at(999.0);
    assert!(before[0].abs() < 1e-9, "antes do início: {before:?}");
    assert!((after[0] - 10.0).abs() < 1e-6, "depois do fim: {after:?}");
}

/// **A geometria manda, a autoria não.** A mesma reta partida em muitos vértices tem de dar o
/// mesmo comprimento e o mesmo ponto no mesmo arco — é a lei que o `ph2d-vec-blend` pagou caro e
/// que o Zig Zag herda deste walker.
#[test]
fn subdividing_a_contour_does_not_move_the_arc() {
    let plain = ArcPath::from_contour(&segment_of([0.0, 0.0], [90.0, 0.0]), false).expect("reta");
    let picked: Vec<VecVertex> = (0..=9)
        .map(|i| VecVertex::corner([f64::from(i) * 10.0, 0.0]))
        .collect();
    let many = ArcPath::from_contour(&picked, false).expect("reta picada");
    assert!((plain.total() - many.total()).abs() < 1e-9);
    for s in [0.0, 7.5, 33.3, 90.0] {
        let (a, _) = plain.frame_at(s);
        let (b, _) = many.frame_at(s);
        assert!(
            (a[0] - b[0]).abs() < 1e-6 && (a[1] - b[1]).abs() < 1e-6,
            "arco {s}: {a:?} vs {b:?}"
        );
    }
}

/// **`closest_arc` inverte `frame_at`** — o ponto na curva no arco `s` tem `closest_arc` de volta
/// em `s`. É o contrato da alça arrastável: o dedo sobre a curva pousa exatamente onde está.
#[test]
fn closest_arc_inverts_frame_at_on_the_curve() {
    let ap = ArcPath::from_contour(&circle(), true).expect("círculo");
    for frac in [0.05, 0.2, 0.37, 0.5, 0.83, 0.99] {
        let s = ap.total() * frac;
        let (p, _) = ap.frame_at(s);
        let got = ap.closest_arc(p);
        assert!(
            (got - s).abs() < 0.05,
            "frac {frac}: fechou em {got:.4}, esperado {s:.4}"
        );
    }
}

/// **Um ponto FORA da curva cai no arco mais próximo** — e a busca não é local, então uma curva
/// em S não a engana. O dedo raramente está exatamente sobre a linha; a alça segue o ponto mais
/// perto dela.
#[test]
fn closest_arc_finds_the_nearest_point_off_the_curve() {
    // Uma onda em S: dois lóbulos, onde uma busca puramente local (Newton) cairia no lóbulo
    // errado a partir de um chute do lado oposto.
    let s_wave = vec![
        VecVertex {
            anchor: [0.0, 0.0],
            in_handle: [-20.0, 0.0],
            out_handle: [20.0, 0.0],
            kind: VertexKind::Smooth,
            corner_radius: 0.0,
        },
        VecVertex {
            anchor: [40.0, 30.0],
            in_handle: [20.0, 30.0],
            out_handle: [60.0, 30.0],
            kind: VertexKind::Smooth,
            corner_radius: 0.0,
        },
        VecVertex {
            anchor: [80.0, -30.0],
            in_handle: [60.0, -30.0],
            out_handle: [100.0, -30.0],
            kind: VertexKind::Smooth,
            corner_radius: 0.0,
        },
        VecVertex {
            anchor: [120.0, 0.0],
            in_handle: [100.0, 0.0],
            out_handle: [140.0, 0.0],
            kind: VertexKind::Smooth,
            corner_radius: 0.0,
        },
    ];
    let ap = ArcPath::from_contour(&s_wave, false).expect("onda");
    // Um alvo bem no meio do primeiro lóbulo, deslocado para fora: a resposta tem de estar no
    // primeiro terço do arco, NÃO no lóbulo de baixo do outro lado.
    let (mid, _) = ap.frame_at(ap.total() * 0.25);
    let target = [mid[0], mid[1] + 8.0];
    let got = ap.closest_arc(target);
    assert!(
        got < ap.total() * 0.45,
        "o alvo perto do 1º lóbulo caiu no arco {got:.3} (total {:.3})",
        ap.total()
    );
    // E o ponto que ele achou está de fato mais perto do alvo que o começo do arco.
    let (q, _) = ap.frame_at(got);
    let d_got = (q[0] - target[0]).hypot(q[1] - target[1]);
    let (q0, _) = ap.frame_at(0.0);
    let d0 = (q0[0] - target[0]).hypot(q0[1] - target[1]);
    assert!(
        d_got < d0,
        "o mais próximo ({d_got:.3}) bate o começo ({d0:.3})"
    );
}

/// Num contorno degenerado (comprimento zero) `closest_arc` é `0`, não `NaN` — a alça não salta.
#[test]
fn closest_arc_of_a_degenerate_contour_is_zero() {
    let ap = ArcPath::from_contour(
        &[VecVertex::corner([5.0, 5.0]), VecVertex::corner([5.0, 5.0])],
        false,
    )
    .expect("contorno");
    assert_eq!(ap.closest_arc([9.0, 9.0]), 0.0);
}
