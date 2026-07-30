//! Gates do ajuste de Hobby — arquivo irmão de `hobby.rs`.
//!
//! # O oráculo que importa é EXTERNO
//!
//! Este módulo é um **port** de 300 linhas de aritmética (`ph2d-vector-doc::hobby`, f32/`glam`
//! → f64/`[f64;2]`), e um port sem oráculo externo é uma reescrita torcendo por sorte: todo
//! gate de propriedade que eu escrevesse aqui passaria igual sobre um sinal trocado no lugar
//! errado, porque as propriedades (interpola, é suave) são *invariantes que sobrevivem a erros
//! de escala e de sinal simétricos*. Então o 1º gate compara os dois solvers **lado a lado**, e
//! os outros descrevem o que o consumidor precisa.
//!
//! ⚠️ A crate congelada entra em `[dev-dependencies]` — **nenhuma linha de `src/` a usa**
//! (machete-safe), exatamente como as 5 crates-nó no dev-dep da `ph2d-gpu-cook` servem só o
//! gate de paridade CPU×GPU.
//!
//! ⚠️ **O épsilon é o do `as f32` do original, e é MEDIDO**: ele calcula em f64 e converte o
//! resultado para `Vec2` no fim, então alimentar os mesmos nós (escolhidos exatamente
//! representáveis em f32) faz a aritmética interna ser *idêntica* e a única diferença ser aquele
//! arredondamento final. Pior delta medido nas quatro fixturas: **4,0e-7** em coordenadas de
//! magnitude até 150 — o bar de [`PARITY_EPS`] fica duas ordens acima, e um erro de verdade (um
//! sinal, um índice, um fator ⅓) sangra por ordens de grandeza, não por ulps.

use super::*;
use ph2d_vec_scene::VertexKind;

/// Bar da paridade contra o solver congelado. Ver o épsilon MEDIDO no doc do módulo.
const PARITY_EPS: f64 = 1e-4;

/// Um traço em S com nós exatamente representáveis em f32 (inteiros e meios) — a fixtura
/// canônica: tem virada nos dois sentidos, que é onde um sinal trocado aparece.
fn s_curve() -> Vec<[f64; 2]> {
    vec![
        [0.0, 0.0],
        [10.0, 30.0],
        [40.0, 35.0],
        [70.0, 5.0],
        [90.0, 40.0],
        [120.0, 20.0],
    ]
}

/// Avalia a cúbica do segmento `i` em `t`, a partir dos handles ABSOLUTOS do `VecVertex` —
/// exatamente como o renderer a lê (`c₀ = out_handle`, `c₁ = in_handle` do nó seguinte).
fn eval(verts: &[VecVertex], i: usize, t: f64) -> [f64; 2] {
    let p0 = verts[i].anchor;
    let c1 = verts[i].out_handle;
    let c2 = verts[i + 1].in_handle;
    let p3 = verts[i + 1].anchor;
    let mt = 1.0 - t;
    let (a, b, c, d) = (mt * mt * mt, 3.0 * mt * mt * t, 3.0 * mt * t * t, t * t * t);
    [
        p0[0] * a + c1[0] * b + c2[0] * c + p3[0] * d,
        p0[1] * a + c1[1] * b + c2[1] * c + p3[1] * d,
    ]
}

fn dist(a: [f64; 2], b: [f64; 2]) -> f64 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2)).sqrt()
}

/// Direção de saída do nó `i` (para frente ao longo da curva).
fn out_dir(v: &VecVertex) -> [f64; 2] {
    [v.out_handle[0] - v.anchor[0], v.out_handle[1] - v.anchor[1]]
}
/// Direção de ENTRADA no nó `i`, apontada para FRENTE ao longo da curva (o handle de entrada
/// aponta para trás, então o vetor para frente é o negativo dele).
fn in_dir_forward(v: &VecVertex) -> [f64; 2] {
    [v.anchor[0] - v.in_handle[0], v.anchor[1] - v.in_handle[1]]
}

fn cross_unit(a: [f64; 2], b: [f64; 2]) -> f64 {
    let (la, lb) = (
        (a[0] * a[0] + a[1] * a[1]).sqrt(),
        (b[0] * b[0] + b[1] * b[1]).sqrt(),
    );
    if la < 1e-12 || lb < 1e-12 {
        return 0.0;
    }
    (a[0] / la) * (b[1] / lb) - (a[1] / la) * (b[0] / lb)
}

/// **O port responde o MESMO que o solver que ele substitui** — o gate que torna o port um port.
///
/// ⚠️ Mutação que tem de sangrar: qualquer erro de sinal, de índice ou de fator na aritmética
/// portada. Medido com `beta[i] = -gamma[i+1] + alpha[i+1]` (um `+` no lugar de um `−`): delta
/// de **34,7** contra o bar de 1e-4.
#[test]
fn the_port_matches_the_frozen_solver_it_replaces() {
    let fixtures: [(&str, Vec<[f64; 2]>); 4] = [
        ("S", s_curve()),
        (
            "arco simetrico",
            vec![
                [-30.0, 0.0],
                [-15.0, 20.0],
                [0.0, 26.0],
                [15.0, 20.0],
                [30.0, 0.0],
            ],
        ),
        (
            "zigue-zague",
            (0..7)
                .map(|i| [f64::from(i) * 8.0, f64::from(i % 2) * 12.0])
                .collect(),
        ),
        ("reta", (0..6).map(|i| [f64::from(i) * 2.0, 0.0]).collect()),
    ];
    let mut worst = 0.0_f64;
    for (name, knots) in fixtures {
        let mine = fit_hobby_open(&knots);
        let theirs = ph2d_vector_doc::hobby::fit_hobby_open(
            &knots
                .iter()
                .map(|k| glam::Vec2::new(k[0] as f32, k[1] as f32))
                .collect::<Vec<_>>(),
        );
        assert_eq!(
            mine.len(),
            theirs.len() + 1,
            "{name}: o port devolve um VERTICE por no' ({} segmentos)",
            theirs.len()
        );
        for (i, t) in theirs.iter().enumerate() {
            // O original dá offsets a partir da âncora de cada ponta; o port dá posições
            // absolutas. A comparação desfaz a diferença de convenção, não de valor.
            let mine_out = out_dir(&mine[i]);
            let mine_in = [
                mine[i + 1].in_handle[0] - mine[i + 1].anchor[0],
                mine[i + 1].in_handle[1] - mine[i + 1].anchor[1],
            ];
            let d_out = dist(
                mine_out,
                [f64::from(t.out_at_start.x), f64::from(t.out_at_start.y)],
            );
            let d_in = dist(
                mine_in,
                [f64::from(t.in_at_end.x), f64::from(t.in_at_end.y)],
            );
            worst = worst.max(d_out).max(d_in);
            assert!(
                d_out < PARITY_EPS && d_in < PARITY_EPS,
                "{name} seg {i}: o port divergiu do solver congelado (out {d_out:.3e}, in \
                 {d_in:.3e}) — bar {PARITY_EPS:.0e}"
            );
        }
    }
    // O número fica na mensagem de sucesso do -- --nocapture: é ele que justifica o bar.
    println!("paridade Hobby f64 x f32 congelado: pior delta {worst:.3e}");
}

/// **A spline PASSA por todo nó** — a propriedade que escolheu Hobby em vez de Schneider. Um
/// ajuste que não passa pelas amostras põe a curva onde a mão não esteve.
#[test]
fn it_passes_through_every_knot() {
    let knots = s_curve();
    let verts = fit_hobby_open(&knots);
    for (i, k) in knots.iter().enumerate() {
        assert!(
            dist(verts[i].anchor, *k) < 1e-12,
            "no' {i} nao e' a ancora do vertice {i}"
        );
    }
    for i in 0..verts.len() - 1 {
        assert!(
            dist(eval(&verts, i, 0.0), knots[i]) < 1e-9,
            "seg {i} inicio"
        );
        assert!(
            dist(eval(&verts, i, 1.0), knots[i + 1]) < 1e-9,
            "seg {i} fim"
        );
    }
}

/// **A tangente é contínua em todo nó interior, e o `kind` DIZ isso.**
///
/// As duas metades andam juntas de propósito: a geometria ser suave e o vértice se declarar
/// `Smooth` são fatos diferentes, e é o segundo que faz o editor de nós **manter** a suavidade
/// quando o artista arrasta um handle. Um nó geometricamente suave marcado `Corner` viraria uma
/// cúspide no primeiro toque.
#[test]
fn the_tangent_is_continuous_at_every_interior_knot() {
    let verts = fit_hobby_open(&s_curve());
    for (i, v) in verts.iter().enumerate().take(verts.len() - 1).skip(1) {
        let c = cross_unit(in_dir_forward(v), out_dir(v));
        assert!(
            c.abs() < 1e-9,
            "no' {i}: tangente quebrada (cross={c:.2e}) — a spline nao e' G1 ali"
        );
        assert_eq!(
            v.kind,
            VertexKind::Smooth,
            "no' {i} interior tem de se DECLARAR suave, senao o editor cria cuspide no 1o toque"
        );
    }
}

/// **Dois nós = uma reta com os handles nos terços.** O caso degenerado do lápis (um risco
/// curto) tem de sair reto, não abaulado.
#[test]
fn two_knots_make_a_straight_cubic_at_the_thirds() {
    let verts = fit_hobby_open(&[[0.0, 0.0], [9.0, 0.0]]);
    assert_eq!(verts.len(), 2);
    assert!(
        dist(verts[0].out_handle, [3.0, 0.0]) < 1e-9,
        "{:?}",
        verts[0]
    );
    assert!(
        dist(verts[1].in_handle, [6.0, 0.0]) < 1e-9,
        "{:?}",
        verts[1]
    );
}

/// **Nós colineares ficam na linha.** Traçar uma borda reta à mão livre não pode produzir uma
/// onda: nem os handles nem o meio de cada cúbica saem do eixo.
#[test]
fn collinear_knots_stay_on_the_line() {
    let knots: Vec<[f64; 2]> = (0..6).map(|i| [f64::from(i) * 2.0, 0.0]).collect();
    let verts = fit_hobby_open(&knots);
    for (i, v) in verts.iter().enumerate() {
        assert!(v.in_handle[1].abs() < 1e-9, "no' {i} in {:?}", v.in_handle);
        assert!(
            v.out_handle[1].abs() < 1e-9,
            "no' {i} out {:?}",
            v.out_handle
        );
    }
    for i in 0..verts.len() - 1 {
        assert!(
            eval(&verts, i, 0.5)[1].abs() < 1e-9,
            "seg {i} meio fora da linha"
        );
    }
}

/// **Menos de dois nós não é um traço**, e um por nó no resto.
#[test]
fn the_vertex_count_is_one_per_knot() {
    assert!(fit_hobby_open(&[]).is_empty());
    assert!(fit_hobby_open(&[[1.0, 2.0]]).is_empty());
    for n in 2..12 {
        let knots: Vec<[f64; 2]> = (0..n).map(|i| [f64::from(i), f64::from(i % 3)]).collect();
        assert_eq!(fit_hobby_open(&knots).len(), n as usize, "n={n}");
    }
}

/// **As pontas não têm handle para fora do traço.** Um handle pendurado além da ponta desenha
/// uma aba que ninguém pediu — e no editor de nós ele é agarrável, então mentiria sobre existir
/// curva ali.
#[test]
fn the_endpoints_have_no_handle_outside_the_stroke() {
    let verts = fit_hobby_open(&s_curve());
    let last = verts.len() - 1;
    assert_eq!(
        verts[0].in_handle, verts[0].anchor,
        "o 1o no' tem handle de ENTRADA"
    );
    assert_eq!(
        verts[last].out_handle, verts[last].anchor,
        "o ultimo no' tem handle de SAIDA"
    );
}

/// **Nó não-finito degrada para uma cadeia FINITA.** Um NaN que atravessasse envenenaria a
/// bbox, o hit-test e o save — o modo de falha tem de ser um traço reto, não um documento morto.
#[test]
fn a_non_finite_knot_degrades_to_a_finite_chain() {
    for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let verts = fit_hobby_open(&[[0.0, 0.0], [bad, 10.0], [20.0, 0.0]]);
        assert_eq!(verts.len(), 3);
        for (i, v) in verts.iter().enumerate() {
            // A âncora não-finita é a ENTRADA (ela atravessa por definição); o que não pode
            // vazar é um handle infinito calculado a partir dela.
            if i != 1 {
                assert!(
                    v.in_handle[0].is_finite() && v.out_handle[0].is_finite(),
                    "no' {i} vazou {bad}: {v:?}"
                );
            }
        }
    }
}

/// **Nós coincidentes não dividem por zero** — um decimador com bug pode deixar um duplicado.
#[test]
fn coincident_knots_do_not_divide_by_zero() {
    let verts = fit_hobby_open(&[[0.0, 0.0], [0.0, 0.0], [10.0, 10.0], [20.0, 0.0]]);
    for (i, v) in verts.iter().enumerate() {
        assert!(
            v.in_handle
                .iter()
                .chain(v.out_handle.iter())
                .all(|c| c.is_finite()),
            "no' {i} nao-finito {v:?}"
        );
    }
}

/// **Uma reversão brusca mantém os handles limitados** (o teto de velocidade do MetaPost). Sem
/// ele, um traço que volta sobre si mesmo produz um laço gigante.
#[test]
fn a_sharp_reversal_keeps_the_handles_bounded() {
    let knots = [[0.0, 0.0], [10.0, 0.0], [0.5, 0.2], [10.0, 5.0]];
    let verts = fit_hobby_open(&knots);
    for (i, v) in verts.iter().enumerate() {
        // Teto 4 ⇒ handle ≤ 4·d/3; a corda mais longa é ~10, então nada acima de ~14.
        assert!(
            dist(v.in_handle, v.anchor) < 14.0 && dist(v.out_handle, v.anchor) < 14.0,
            "no' {i} com handle em fuga: {v:?}"
        );
    }
}

/// **O curl move a tangente da PONTA** (e só ela é fronteira). É o parâmetro que um dia vira
/// controle; hoje o gate impede que ele seja silenciosamente ignorado.
#[test]
fn the_curl_moves_the_endpoint_tangent() {
    let knots = [[0.0, 0.0], [20.0, 30.0], [50.0, 10.0]];
    let relaxed = fit_hobby_open_with_curl(&knots, 0.0);
    let curled = fit_hobby_open_with_curl(&knots, 1.0);
    assert!(
        dist(relaxed[0].out_handle, curled[0].out_handle) > 1e-3,
        "o curl nao teve efeito: {:?} vs {:?}",
        relaxed[0].out_handle,
        curled[0].out_handle
    );
}

/// **Mesma entrada, mesma saída** — o ajuste é função pura (nenhum estado, nenhum relógio).
#[test]
fn the_fit_is_a_pure_function_of_the_knots() {
    let knots = s_curve();
    assert_eq!(fit_hobby_open(&knots), fit_hobby_open(&knots));
}
