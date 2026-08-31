//! **SONDA — em que formas do catálogo as duas REGRAS DE PREENCHIMENTO discordam?**
//!
//! A folha 14 marca `P1` no *"`fill_rule` não é exposto"* e cita o caso: *"uma estrela de
//! `sides` alto com `star_depth` alto **auto-intersecta**, e nonzero × evenodd são
//! visivelmente diferentes"*.
//!
//! ⚠️ **O oráculo dessa afirmação não é a geometria, é o PREENCHIMENTO** — e é por isso que o
//! gate `no_kind_hides_a_live_knob_or_shows_a_dead_one`, que compara vértices, não consegue
//! julgar este param: mudar a regra não move um único ponto. O que muda é *que região fica
//! pintada*.
//!
//! Mas não é preciso rasterizar para saber: as duas regras discordam **exactamente** onde o
//! número de voltas (*winding number*) tem magnitude **≥ 2** com paridade PAR — ali o
//! `NonZero` pinta e o `EvenOdd` fura. A sonda amostra uma grelha sobre a caixa de cada
//! forma, conta as voltas por lançamento de raio, e diz em quantos pontos as duas regras dão
//! respostas diferentes.
//!
//! Ela **imprime e não afirma**. Rode com
//! `cargo test -p ph2d-vec-scene --test measure_fill_rule_matters -- --ignored --nocapture`.

use ph2d_vec_scene::{ALL_SHAPES, ShapeKind, VecPath, cook};

const A: [f64; 2] = [-1.0, -1.0];
const B: [f64; 2] = [1.0, 1.0];
/// Lado da grelha de amostragem (N×N pontos sobre a caixa).
const GRID: usize = 81;
/// Quantos pedaços por segmento cúbico ao achatar o contorno em polilinha.
const FLAT: usize = 24;

/// O contorno (e os subcontornos) achatados em polilinhas.
fn polylines(p: &VecPath) -> Vec<Vec<[f64; 2]>> {
    let mut out = Vec::new();
    let mut push = |verts: &[ph2d_vec_scene::VecVertex], closed: bool| {
        if verts.len() < 2 {
            return;
        }
        let n = verts.len();
        let segs = if closed { n } else { n - 1 };
        let mut line = Vec::with_capacity(segs * FLAT + 1);
        for i in 0..segs {
            let (a, b) = (verts[i], verts[(i + 1) % n]);
            let c = [a.anchor, a.out_handle, b.in_handle, b.anchor];
            for k in 0..FLAT {
                let t = k as f64 / FLAT as f64;
                let u = 1.0 - t;
                let w = [u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t];
                line.push([
                    c[0][0] * w[0] + c[1][0] * w[1] + c[2][0] * w[2] + c[3][0] * w[3],
                    c[0][1] * w[0] + c[1][1] * w[1] + c[2][1] * w[2] + c[3][1] * w[3],
                ]);
            }
        }
        line.push(line[0]);
        out.push(line);
    };
    push(&p.verts, p.closed);
    for sp in &p.subpaths {
        push(&sp.verts, sp.closed);
    }
    out
}

/// O número de VOLTAS do contorno em torno de `q` — a soma dos cruzamentos com sinal de um
/// raio para +X.
fn winding(lines: &[Vec<[f64; 2]>], q: [f64; 2]) -> i32 {
    let mut w = 0;
    for line in lines {
        for seg in line.windows(2) {
            let (a, b) = (seg[0], seg[1]);
            if (a[1] <= q[1]) != (b[1] <= q[1]) {
                let t = (q[1] - a[1]) / (b[1] - a[1]);
                if a[0] + t * (b[0] - a[0]) > q[0] {
                    w += if b[1] > a[1] { 1 } else { -1 };
                }
            }
        }
    }
    w
}

/// Em quantos pontos da grelha as duas regras discordam.
fn disagreements(p: &VecPath) -> usize {
    let lines = polylines(p);
    let mut n = 0;
    for iy in 0..GRID {
        for ix in 0..GRID {
            // Deslocado meio passo: um ponto exactamente sobre uma aresta é ambíguo nos dois.
            let q = [
                A[0] + (B[0] - A[0]) * (ix as f64 + 0.5) / GRID as f64,
                A[1] + (B[1] - A[1]) * (iy as f64 + 0.5) / GRID as f64,
            ];
            let w = winding(&lines, q);
            if (w != 0) != (w % 2 != 0) {
                n += 1;
            }
        }
    }
    n
}

#[test]
#[ignore = "sonda: imprime numeros, nao afirma"]
fn where_the_two_fill_rules_disagree() {
    eprintln!("\n[fill-rule] pontos da grelha {GRID}x{GRID} em que NonZero e EvenOdd discordam\n");
    let mut any = Vec::new();
    for &k in ALL_SHAPES {
        let n = disagreements(&cook(k, A, B, k.defaults().as_slice()));
        if n > 0 {
            any.push((format!("{k:?}"), n));
        }
    }
    eprintln!("  nos DEFAULTS da biblioteca: {any:?}");

    // O caso que a folha cita: a estrela auto-intersecta quando as pontas passam do centro.
    eprintln!("\n  a ESTRELA, varrendo `star_depth` (pontas / raio interno):");
    eprintln!("  {:>7}  {:>7}  {:>14}", "sides", "depth", "discordancias");
    for sides in [5.0f64, 7.0, 11.0] {
        for depth in [0.1f64, 0.3, 0.45, 0.8, 0.95] {
            let v = vec![sides, depth, 0.0, 0.0];
            let n = disagreements(&cook(ShapeKind::Star, A, B, &v));
            eprintln!("  {sides:>7.0}  {depth:>7.2}  {n:>14}");
        }
    }
    eprintln!(
        "\n  LEITURA: zero discordancias = as duas regras pintam a MESMA coisa, e um dropdown
  de `fill_rule` seria um botao morto naquela forma. Um numero grande = a folha
  esta' certa e o knob muda o desenho."
    );
}
