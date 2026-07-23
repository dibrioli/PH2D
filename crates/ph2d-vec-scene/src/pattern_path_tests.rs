//! Gates do [`super::pattern_along`] — o motor de Pattern Along Path (plano 23, W1).
//!
//! O oráculo é **geométrico** (contagem de cópias, espaçamento medido, a direção que a cópia
//! aponta), nunca um espelho da fórmula de amostragem. A propriedade que mais importa é a que
//! separa este motor do Repeater: as cópias **rodam para a tangente** da guia — e essa é medida da
//! GEOMETRIA emitida contra a tangente de verdade da curva, de duas formas independentes, para que
//! a mutação "usa um referencial fixo" sangre por qualquer uma.

use super::{PatternSpec, pattern_along};
use crate::arc_path::ArcPath;
use crate::{Contour, VecPath, VecVertex, VertexKind};

/// Uma guia reta de `(0,0)` a `(len,0)`, aberta.
fn straight(len: f64) -> ArcPath {
    ArcPath::from_contour(
        &[VecVertex::corner([0.0, 0.0]), VecVertex::corner([len, 0.0])],
        false,
    )
    .expect("guia reta")
}

/// Um círculo de raio `R` em quatro cúbicas — a guia que faz a tangente girar 360°.
fn circle(r: f64) -> ArcPath {
    const K: f64 = 0.552_284_749_830_793_4;
    let p = [[r, 0.0], [0.0, r], [-r, 0.0], [0.0, -r]];
    let tang = [[0.0, K * r], [-K * r, 0.0], [0.0, -K * r], [K * r, 0.0]];
    let verts: Vec<VecVertex> = (0..4)
        .map(|i| VecVertex {
            anchor: p[i],
            in_handle: [p[i][0] - tang[i][0], p[i][1] - tang[i][1]],
            out_handle: [p[i][0] + tang[i][0], p[i][1] + tang[i][1]],
            kind: VertexKind::Smooth,
            corner_radius: 0.0,
        })
        .collect();
    ArcPath::from_contour(&verts, true).expect("círculo")
}

/// Um quadrado de lado 40 na origem — bbox 40×40, centro `(20,20)`.
fn square() -> VecPath {
    VecPath {
        verts: [[0.0, 0.0], [40.0, 0.0], [40.0, 40.0], [0.0, 40.0]]
            .map(VecVertex::corner)
            .to_vec(),
        closed: true,
        ..VecPath::default()
    }
}

/// Uma seta ASSIMÉTRICA que aponta em `+x` — a ponta é o vértice de índice **1**. bbox 40 de
/// largura, centro `(20, 0)`. É o motivo que revela a rotação: uma forma simétrica não teria como.
fn arrow() -> VecPath {
    VecPath {
        verts: [[0.0, -5.0], [40.0, 0.0], [0.0, 5.0]]
            .map(VecVertex::corner)
            .to_vec(),
        closed: true,
        ..VecPath::default()
    }
}

fn bbox(c: &Contour) -> ([f64; 2], [f64; 2]) {
    let (mut lo, mut hi) = ([f64::MAX; 2], [f64::MIN; 2]);
    for v in &c.verts {
        for k in 0..2 {
            lo[k] = lo[k].min(v.anchor[k]);
            hi[k] = hi[k].max(v.anchor[k]);
        }
    }
    (lo, hi)
}

fn centroid(c: &Contour) -> [f64; 2] {
    let n = c.verts.len() as f64;
    let s = c
        .verts
        .iter()
        .fold([0.0, 0.0], |a, v| [a[0] + v.anchor[0], a[1] + v.anchor[1]]);
    [s[0] / n, s[1] / n]
}

fn norm(v: [f64; 2]) -> [f64; 2] {
    let m = v[0].hypot(v[1]);
    if m == 0.0 { [0.0, 0.0] } else { [v[0] / m, v[1] / m] }
}

/// **As cópias tilam a guia reta**: contagem certa, espaçadas pelo avanço, forma preservada.
///
/// Guia de 100, quadrado de 40, `spacing 1.0` ⇒ avanço 40. As FATIAS que cabem em `[0,100]` são
/// `[0,40]` e `[40,80]` — a terceira (`[80,120]`) transbordaria ⇒ **2 cópias** (centros 20 e 60),
/// e a cauda `[80,100]` sobra (convenção sem transbordo, plano 23 §3). Cada cópia é o quadrado
/// 40×40 centrado no seu arco — sem escala, sem rotação (a tangente da reta é constante).
#[test]
fn the_copies_tile_the_straight_guide() {
    let out = pattern_along(&square(), &straight(100.0), &PatternSpec::default());
    assert_eq!(out.len(), 2, "contagem de cópias");
    for (k, c) in out.iter().enumerate() {
        let (lo, hi) = bbox(c);
        let cx = 20.0 + 40.0 * k as f64;
        assert!((hi[0] - lo[0] - 40.0).abs() < 1e-9, "largura preservada");
        assert!((hi[1] - lo[1] - 40.0).abs() < 1e-9, "altura preservada");
        assert!(((lo[0] + hi[0]) * 0.5 - cx).abs() < 1e-9, "centro x da cópia {k}");
        assert!(((lo[1] + hi[1]) * 0.5).abs() < 1e-9, "centro y na baseline");
    }
}

/// **RED-FIRST: as cópias rodam para a tangente da curva.** É o que separa o pattern do Repeater
/// (que não conhece a guia). Numa guia circular, a seta de cada cópia tem de apontar ao longo da
/// tangente ALI — e provo isso por DUAS medidas independentes, ambas da geometria emitida:
///
/// 1. a direção `ponta − centroide` de cada cópia é **paralela** à tangente da guia no arco do seu
///    centro (a tangente vem do `frame_at`, a direção vem dos vértices emitidos);
/// 2. as direções das cópias **espalham** mais de 90° ao longo do círculo (medida que NÃO chama o
///    `frame_at`).
///
/// A mutação "usa um referencial fixo" (ignora `s_k`) faz TODAS as setas apontarem igual: (1)
/// falha para as cópias cuja tangente difere, e (2) colapsa o espalhamento a 0°.
#[test]
fn the_copies_rotate_to_the_tangent_on_a_curve() {
    let r = 60.0;
    let guide = circle(r);
    let advance = 40.0; // largura(seta) × spacing 1.0
    let out = pattern_along(&arrow(), &guide, &PatternSpec::default());
    assert!(out.len() >= 8, "um círculo de raio 60 cabe ~9 setas, veio {}", out.len());

    let mut dirs = Vec::new();
    for (k, c) in out.iter().enumerate() {
        assert_eq!(c.verts.len(), 3, "a seta tem 3 vértices");
        let tip = c.verts[1].anchor; // a ponta é o índice 1, preservado pela ordem
        let dir = norm([tip[0] - centroid(c)[0], tip[1] - centroid(c)[1]]);
        // (1) paralela à tangente no arco do centro da cópia.
        let s = advance * (k as f64 + 0.5);
        let (_, t) = guide.frame_at(s);
        let tan = norm(t);
        let dot = dir[0] * tan[0] + dir[1] * tan[1];
        assert!(dot > 0.999, "cópia {k}: dir·tangente = {dot} (deveria ser ~1)");
        dirs.push(dir);
    }
    // (2) espalhamento: o maior ângulo entre duas direções passa de 90° (colapsa a 0° na mutação).
    let mut max_ang = 0.0_f64;
    for i in 0..dirs.len() {
        for j in (i + 1)..dirs.len() {
            let d = (dirs[i][0] * dirs[j][0] + dirs[i][1] * dirs[j][1]).clamp(-1.0, 1.0);
            max_ang = max_ang.max(d.acos());
        }
    }
    assert!(
        max_ang > std::f64::consts::FRAC_PI_2,
        "as setas mal giram (espalhamento {}°) — referencial fixo?",
        max_ang.to_degrees()
    );
}

/// **A saída é LINEAR na contagem** — `cópias × contornos(motivo)` contornos, e
/// `cópias × verts(motivo)` vértices. Uma multiplicação acidental (cópias de cópias) estouraria
/// isto; é a guarda anti-quadrática barata que roda sempre.
#[test]
fn the_output_is_linear_in_the_copy_count() {
    let motif = square();
    let out = pattern_along(&motif, &straight(1000.0), &PatternSpec::default());
    // 1000 / 40 = 25 cópias (centros 20,60,…,980 ⇒ 25).
    assert_eq!(out.len(), 25 * motif.contour_count());
    let verts: usize = out.iter().map(|c| c.verts.len()).sum();
    assert_eq!(verts, 25 * motif.total_verts());
}

/// Guia degenerada ⇒ nada, sem pânico (o `total <= 0` e o `k_hi < k_lo`).
#[test]
fn a_guide_that_holds_no_copy_yields_nothing() {
    // Motivo largo demais para caber uma vez: avanço 40 numa guia de 10.
    let out = pattern_along(&square(), &straight(10.0), &PatternSpec::default());
    assert!(out.is_empty(), "nenhuma cópia cabe: veio {}", out.len());
}

/// **Perf (kill):** um motivo de ~40 vértices numa guia que cabe ~200 cópias re-cozinha sob o kill
/// de 8 ms (o mesmo do texto). `#[ignore]` + `--release`: em debug o número mede o build, não o
/// produto (a lição do texto). Se passar disto, o próximo passo é cache por-params, não subir o
/// teto (plano 23 §0).
#[test]
#[ignore = "perf: rode com --release -- --ignored"]
fn a_keystroke_recook_stays_under_the_kill() {
    // Motivo de 40 vértices (um polígono), largura ~40.
    let verts: Vec<VecVertex> = (0..40)
        .map(|i| {
            let a = i as f64 / 40.0 * std::f64::consts::TAU;
            VecVertex::corner([20.0 + 20.0 * a.cos(), 20.0 * a.sin()])
        })
        .collect();
    let motif = VecPath {
        verts,
        closed: true,
        ..VecPath::default()
    };
    let guide = straight(40.0 * 200.0); // ~200 cópias com avanço ~40
    let spec = PatternSpec::default();

    let t0 = std::time::Instant::now();
    let out = pattern_along(&motif, &guide, &spec);
    let dt = t0.elapsed().as_secs_f64() * 1e3;
    eprintln!("pattern_along: {} cópias × 40 verts em {dt:.3} ms", out.len());
    assert!(out.len() >= 190, "esperava ~200 cópias, veio {}", out.len());
    assert!(dt < 8.0, "re-cook de {} cópias custou {dt:.2} ms (kill 8)", out.len());
}
