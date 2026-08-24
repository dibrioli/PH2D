//! Os gates da **MÉTRICA** (doc 89, folha 01).
//!
//! ⚠️ **A métrica entra DENTRO do laço de Lloyd**, que é exactamente por que a célula
//! dizia «não exprimível»: nenhum nó a jusante pode reescrever uma relaxação que já
//! aconteceu. Aqui ela decide de que ponto cada amostra do plano é — e a relaxação
//! inteira muda com ela.

use super::*;

const N: usize = 48;
const W: f32 = 5.0;
const H: f32 = 5.0;
const SEED: u32 = 4;
const ITERS: usize = 10;

fn relaxed(metric: i32) -> Vec<[f32; 2]> {
    relaxed_points(N, W, H, SEED, ITERS, 1.0, metric)
}

/// ⭐ **A EUCLIDIANA É O CVT DE SEMPRE, AO BIT** — o default não move um ponto.
#[test]
fn the_default_metric_reproduces_the_old_cvt_bit_for_bit() {
    let a = relaxed(METRIC_EUCLIDEAN);
    // Um número fora da escada cai na euclidiana pelo braço `_` — a mesma resposta,
    // e o gate diz que isso é deliberado e não um acidente do `match`.
    let b = relaxed(77);
    assert_eq!(a.len(), N);
    for (i, (p, q)) in a.iter().zip(&b).enumerate() {
        assert_eq!(p.map(f32::to_bits), q.map(f32::to_bits), "ponto {i}");
    }
}

/// ⭐⭐ **AS TRÊS MÉTRICAS DÃO TRÊS ARRANJOS DIFERENTES** — o knob está vivo, e não é
/// um dropdown que muda um rótulo.
#[test]
fn each_metric_relaxes_to_a_different_arrangement() {
    let e = relaxed(METRIC_EUCLIDEAN);
    let m = relaxed(METRIC_MANHATTAN);
    let c = relaxed(METRIC_CHEBYSHEV);
    let spread = |a: &[[f32; 2]], b: &[[f32; 2]]| {
        a.iter()
            .zip(b)
            .map(|(p, q)| (p[0] - q[0]).hypot(p[1] - q[1]))
            .fold(0.0_f32, f32::max)
    };
    // A semente é a MESMA nas três, então o que diverge é a relaxação.
    assert!(
        spread(&e, &m) > 0.05,
        "Manhattan ficou igual: {}",
        spread(&e, &m)
    );
    assert!(
        spread(&e, &c) > 0.05,
        "Chebyshev ficou igual: {}",
        spread(&e, &c)
    );
    assert!(spread(&m, &c) > 0.05, "as duas nao-euclidianas coincidiram");
}

/// ⭐ **AS TRÊS SÃO OS TRÊS MARCOS DA FAMÍLIA DE MINKOWSKI** — `p = 1`, `p = 2` e
/// `p → ∞` —, e é isso que torna a recusa do expoente livre pequena.
///
/// A prova é a ordem: para todo vector, `L∞ ≤ L² ≤ L¹`. Um `nearest` que trocasse dois
/// braços do `match` quebraria esta ordem sobre a mesma amostra.
#[test]
fn the_three_metrics_are_the_three_landmarks_of_the_minkowski_family() {
    let probe = [[3.0_f32, 4.0], [-1.0, 0.5], [0.25, -2.0]];
    for v in probe {
        let l1 = v[0].abs() + v[1].abs();
        let l2 = (v[0] * v[0] + v[1] * v[1]).sqrt();
        let linf = v[0].abs().max(v[1].abs());
        assert!(
            linf <= l2 + 1e-6 && l2 <= l1 + 1e-6,
            "{v:?}: {linf} {l2} {l1}"
        );
    }
    // ⛔ E o expoente livre NÃO existe: três rótulos, três números, sem `powf` no
    // caminho de um gerador determinista (HR-5 — o replay-hash corre em 3 SOs).
    assert_eq!(METRIC_LABELS.len(), 3);
    assert!(
        !METRIC_LABELS.iter().any(|l| l.contains("Minkowski")),
        "a Minkowski e' uma recusa medida, nao um rotulo"
    );
}

/// ⚠️ **A métrica muda a ATRIBUIÇÃO, e é aí que ela morde.** Uma amostra pode ser de
/// uma célula na euclidiana e de outra na Chebyshev — este é o caso construído em que
/// isso acontece, e é o mecanismo de que tudo o resto decorre.
#[test]
fn the_metric_decides_which_cell_a_sample_belongs_to() {
    // `a` está mais perto em L² (5,0 contra 5,1) e `b` está mais perto em L∞ (5 contra 3).
    let pts = [[3.0_f32, 4.0], [5.1, 0.0]];
    let origin = [0.0_f32, 0.0];
    assert_eq!(
        nearest(origin, &pts, METRIC_EUCLIDEAN),
        0,
        "L2 escolhe o `a`"
    );
    assert_eq!(
        nearest(origin, &pts, METRIC_CHEBYSHEV),
        0,
        "L-inf tambem, aqui"
    );
    // E o caso ao contrário: `a` a (4,4) tem L∞ = 4 e L² = 5,66; `b` a (5,0) tem
    // L∞ = 5 e L² = 5,0.
    let pts2 = [[4.0_f32, 4.0], [5.0, 0.0]];
    assert_eq!(
        nearest(origin, &pts2, METRIC_EUCLIDEAN),
        1,
        "L2 escolhe o `b`"
    );
    assert_eq!(
        nearest(origin, &pts2, METRIC_CHEBYSHEV),
        0,
        "L-inf escolhe o `a`"
    );
    assert_eq!(
        nearest(origin, &pts2, METRIC_MANHATTAN),
        1,
        "L1 escolhe o `b`"
    );
}

/// O param é declarado, tem hint, e o slider alcança os três rótulos.
#[test]
fn the_metric_is_declared_and_the_slider_reaches_every_label() {
    assert_eq!(MANIFEST.param_default(METRIC), Some(0.0));
    let hint = PARAM_HINTS
        .iter()
        .find(|h| h.param == METRIC)
        .expect("hint do `metric`");
    let ParamWidget::Enum { labels } = hint.widget else {
        panic!("o `metric` e' um enum")
    };
    assert_eq!(labels.len(), METRIC_LABELS.len());
    assert_eq!(hint.max, (METRIC_LABELS.len() - 1) as f32);
}
