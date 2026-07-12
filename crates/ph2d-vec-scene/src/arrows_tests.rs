//! Testes de `arrows.rs` — arquivo irmao (teto de 700 LOC por arquivo, HR-18).
//!
//! Ligado por `#[path]` no modulo pai, entao `use super::*` continua valendo.

use super::*;

const A: [f64; 2] = [-2.0, -1.0];
const B: [f64; 2] = [2.0, 1.0];

/// Quantas amostras por cúbica a bbox do teste usa. O `fit` resolve a derivada da
/// cúbica e acha o extremo EXATO; o teste, de propósito, não copia essa conta — ele
/// varre a curva. O erro da varredura cai com `1/n²` (a 64 amostras erra 2e-5 do lado
/// de DENTRO, o que faria o teste acusar a forma de não encostar na borda).
const SAMPLES: u32 = 512;

/// A bbox VERDADEIRA (a CURVA, não as âncoras) de um contorno fechado. Medir só as
/// âncoras subestima a forma: a Bézier passeia fora do casco delas, e é justamente aí
/// que o arco da faixa toca a borda da caixa.
fn curve_bbox(p: &VecPath) -> ([f64; 2], [f64; 2]) {
    let (mut lo, mut hi) = ([f64::MAX; 2], [f64::MIN; 2]);
    let n = p.verts.len();
    for i in 0..n {
        let (a, b) = (&p.verts[i], &p.verts[(i + 1) % n]);
        for s in 0..=SAMPLES {
            let q = crate::cubic_at(
                a.anchor,
                a.out_handle,
                b.in_handle,
                b.anchor,
                f64::from(s) / f64::from(SAMPLES),
            );
            for k in 0..2 {
                lo[k] = lo[k].min(q[k]);
                hi[k] = hi[k].max(q[k]);
            }
        }
    }
    (lo, hi)
}

/// O afim DIAGONAL que o [`fit`] aplicou, recuperado comparando a forma crua com a
/// publicada (`x' = ax·x + bx`). O `fit` é escala + translação por eixo — nunca
/// rotaciona —, então dois vértices bastam. É o que permite ao teste **desfazer** o
/// ajuste e medir o arco onde ele é um CÍRCULO de verdade, sem copiar uma linha da
/// implementação.
fn recover_fit(raw: &VecPath, out: &VecPath) -> [[f64; 2]; 2] {
    let axis = |k: usize| -> [f64; 2] {
        let r0 = raw.verts[0].anchor[k];
        let i = (0..raw.verts.len())
            .max_by(|&x, &y| {
                (raw.verts[x].anchor[k] - r0)
                    .abs()
                    .total_cmp(&(raw.verts[y].anchor[k] - r0).abs())
            })
            .expect("a seta tem vertices");
        let dr = raw.verts[i].anchor[k] - r0;
        assert!(
            dr.abs() > 1e-6,
            "eixo degenerado: nao da para recuperar o fit"
        );
        let scale = (out.verts[i].anchor[k] - out.verts[0].anchor[k]) / dr;
        [scale, out.verts[0].anchor[k] - scale * r0]
    };
    [axis(0), axis(1)]
}

/// Mundo (publicado) → espaço de autoria: desfaz o `fit` e depois o `Unit::p`.
fn to_unit(p: [f64; 2], m: [[f64; 2]; 2]) -> Uv {
    let (cx, cy) = ((A[0] + B[0]) * 0.5, (A[1] + B[1]) * 0.5);
    let (hw, hh) = ((B[0] - A[0]).abs() * 0.5, (B[1] - A[1]).abs() * 0.5);
    let x = (p[0] - m[0][1]) / m[0][0];
    let y = (p[1] - m[1][1]) / m[1][0];
    (((x - cx) / hw + 1.0) * 0.5, (1.0 - (y - cy) / hh) * 0.5)
}

/// A seta reta APONTA: a ponta é o único vértice no extremo +X, e ela fica na linha do
/// meio. É o que distingue uma seta de um retângulo com um entalhe.
#[test]
fn the_block_arrow_has_a_single_tip_on_the_right_centreline() {
    let p = arrow_block(A, B, 0.4, 0.4, 1.0);
    let hi = p.verts_all().map(|v| v.anchor[0]).fold(f64::MIN, f64::max);
    let tips: Vec<&VecVertex> = p
        .verts
        .iter()
        .filter(|v| (v.anchor[0] - hi).abs() < 1e-9)
        .collect();
    assert_eq!(tips.len(), 1, "uma ponta so");
    assert!(
        tips[0].anchor[1].abs() < 1e-9,
        "a ponta esta na linha do meio"
    );
    assert!(
        (hi - B[0]).abs() < 1e-9,
        "a ponta encosta na borda da caixa"
    );
}

/// A cabeça é mais LARGA que a haste — senão não é uma seta, é uma barra. O clamp
/// garante isso mesmo com um `head_w` menor que o `tail` (o usuário pode digitar).
#[test]
fn the_head_is_never_narrower_than_the_tail() {
    let p = arrow_block(A, B, 0.9, 0.4, 0.1); // cabeça pedida MENOR que a haste
    let widest = p
        .verts
        .iter()
        .map(|v| v.anchor[1].abs())
        .fold(0.0, f64::max);
    let tail = 0.9; // meia-espessura pedida, em mundo (a caixa tem meia-altura 1)
    assert!(
        widest >= tail - 1e-9,
        "o contorno inverteria: cabeca {widest} < haste {tail}"
    );
}

/// A seta dupla tem ponta nos DOIS extremos, ambas na linha do meio.
#[test]
fn the_double_arrow_points_both_ways() {
    let p = arrow_double(A, B, 0.4, 0.3, 1.0);
    let lo = p.verts_all().map(|v| v.anchor[0]).fold(f64::MAX, f64::min);
    let hi = p.verts_all().map(|v| v.anchor[0]).fold(f64::MIN, f64::max);
    assert!(
        (lo - A[0]).abs() < 1e-9 && (hi - B[0]).abs() < 1e-9,
        "as duas bordas"
    );
    let on_axis = p.verts.iter().filter(|v| v.anchor[1].abs() < 1e-9).count();
    assert_eq!(on_axis, 2, "as duas pontas ficam no eixo");
}

/// **A seta em L aponta para CIMA** (mundo Y-para-cima): a ponta é o vértice mais
/// alto, é única, e nasce sobre a haste vertical.
#[test]
fn the_bent_arrow_points_up() {
    for corner in [0.0, 0.35, 1.0] {
        let p = arrow_bent(A, B, 0.25, 0.3, 0.6, corner);
        let hi = p.verts_all().map(|v| v.anchor[1]).fold(f64::MIN, f64::max);
        let tips = p
            .verts
            .iter()
            .filter(|v| (v.anchor[1] - hi).abs() < 1e-9)
            .count();
        assert_eq!(tips, 1, "corner {corner}: uma ponta so");
        assert!(
            (hi - B[1]).abs() < 1e-9,
            "corner {corner}: a ponta encosta no TOPO da caixa ({hi})"
        );
    }
}

/// O cotovelo da seta em L é um par de arcos CONCÊNTRICOS — e com `corner = 0` ele
/// degenera na quina quadrada sem sobrar um vértice solto.
#[test]
fn the_bent_elbow_is_a_true_arc_and_squares_off_at_zero() {
    let round = arrow_bent(A, B, 0.25, 0.3, 0.6, 0.4);
    let curvy = round
        .verts
        .iter()
        .filter(|v| (v.out_handle[0] - v.anchor[0]).hypot(v.out_handle[1] - v.anchor[1]) > 1e-6)
        .count();
    assert!(curvy >= 2, "o cotovelo redondo tem de ter handles: {curvy}");

    let square = arrow_bent(A, B, 0.25, 0.3, 0.6, 0.0);
    for v in square.verts_all() {
        let h = (v.out_handle[0] - v.anchor[0]).hypot(v.out_handle[1] - v.anchor[1]);
        assert!(h < 1e-9, "corner=0 e o cotovelo QUADRADO: handle solto {h}");
    }
    assert_eq!(
        square.verts.len(),
        9,
        "quadrado = 9 quinas (cauda, cotovelo externo, 5 da cabeca, cotovelo interno, volta)"
    );
}

/// O chevron tem entalhe: com `notch = 0` é um pentágono; acima disso, o vértice de
/// trás entra na caixa (é o encaixe do próximo chevron).
#[test]
fn the_chevron_notch_bites_into_the_back_edge() {
    let flat = chevron(A, B, 0.3, 0.0);
    let bitten = chevron(A, B, 0.3, 0.25);
    assert!(
        (flat.verts[5].anchor[0] - A[0]).abs() < 1e-9,
        "sem entalhe, a traseira e reta"
    );
    assert!(
        bitten.verts[5].anchor[0] > flat.verts[5].anchor[0],
        "o entalhe entra na forma"
    );
}

/// Todas as setas cabem na caixa do gesto — medido na CURVA. Uma que vazasse faria a
/// bbox mentir e o gizmo desalinhar do desenho.
#[test]
fn every_arrow_fits_inside_the_gesture_box() {
    let shapes = [
        ("block", arrow_block(A, B, 0.4, 0.4, 1.0)),
        ("double", arrow_double(A, B, 0.4, 0.3, 1.0)),
        ("bent", arrow_bent(A, B, 0.25, 0.3, 0.6, 0.35)),
        ("chevron", chevron(A, B, 0.3, 0.2)),
        ("curved", curved_default()),
    ];
    for (name, p) in shapes {
        let (lo, hi) = curve_bbox(&p);
        assert!(
            lo[0] >= A[0] - 1e-6
                && hi[0] <= B[0] + 1e-6
                && lo[1] >= A[1] - 1e-6
                && hi[1] <= B[1] + 1e-6,
            "{name}: a curva vaza a caixa ({lo:?}..{hi:?})"
        );
    }
}

/// **Determinismo:** cozinhar duas vezes dá os MESMOS bytes. Zero aleatoriedade no
/// cozimento — é o que permite ao undo por snapshot comparar estados por igualdade.
#[test]
fn cooking_twice_gives_the_same_bytes() {
    let mk = || {
        vec![
            arrow_block(A, B, 0.37, 0.41, 0.83),
            arrow_double(A, B, 0.37, 0.29, 0.83),
            arrow_bent(A, B, 0.23, 0.31, 0.61, 0.37),
            chevron(A, B, 0.29, 0.19),
        ]
    };
    let (one, two) = (mk(), mk());
    for (a, b) in one.iter().zip(&two) {
        let (x, y) = (
            postcard::to_allocvec(a).expect("serializa"),
            postcard::to_allocvec(b).expect("serializa"),
        );
        assert_eq!(x, y, "o cozimento nao e deterministico");
    }
}

/// Nenhum parâmetro, em nenhum extremo da faixa que a UI publica, produz NaN ou uma
/// forma vazia — o clamp de cada um deles é uma promessa executável.
#[test]
fn the_parameter_extremes_never_degenerate() {
    for &tail in &[0.02, 0.5, 0.95] {
        for &hl in &[0.05, 0.5, 0.95] {
            for &hw in &[0.05, 0.5, 1.0] {
                for p in [
                    arrow_block(A, B, tail, hl, hw),
                    arrow_double(A, B, tail, hl, hw),
                    arrow_bent(A, B, tail, hl, hw, 0.37),
                    chevron(A, B, hl, tail),
                ] {
                    assert!(p.verts.len() >= 4, "contorno raquitico");
                    for v in p.verts_all() {
                        assert!(
                            v.anchor.iter().all(|x| x.is_finite())
                                && v.in_handle.iter().all(|x| x.is_finite())
                                && v.out_handle.iter().all(|x| x.is_finite()),
                            "NaN/inf num vertice: {v:?}"
                        );
                    }
                }
            }
        }
    }
}
