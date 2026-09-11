//! **SONDA — as quinas de uma rosca CORTADA arredondam pela máquina que já existe?**
//!
//! Feedback do Enio no smoke de 2026-08-19: *"senti falta de controle das quinas de uma rosca
//! cortada e formas similares"*. Um anel parcial tem **quatro** quinas vivas (onde cada arco
//! encontra a reta radial), uma pizza tem **três** (as duas pontas do arco e o bico no centro),
//! e nenhuma delas tem knob — o `corner` do nó só chega às espécies cuja receita o recebe
//! (caixa, polígono, estrela).
//!
//! A casa já tem a máquina: [`round_authored_corners`] arredonda cada quina pelo
//! `corner_radius` do vértice dela, sobre segmentos CÚBICOS (não só polilinhas), com chanfro
//! no sinal negativo, e devolve `None` quando não há nada a fazer.
//!
//! ⚠️ **A pergunta que decide se ela serve é se ela SABE o que é uma quina**: um arco é feito
//! de vértices, e arredondar os do MEIO deformaria a curva. O `corner_setback` recusa uma
//! quina colinear — e um vértice suave de arco é, por construção, colinear. Se isso se
//! confirmar, carimbar o raio em TODOS os vértices é seguro, e a regra fica **derivada** em
//! vez de uma lista de índices por espécie (que apodrece à primeira forma nova).
//!
//! Ela **imprime e não afirma**. Rode com
//! `cargo test -p ph2d-vec-scene --test measure_corner_on_arcs -- --ignored --nocapture`.

use ph2d_vec_scene::corner_live::{corner_at, round_authored_corners};
use ph2d_vec_scene::{ShapeKind, VecPath, cook};

const A: [f64; 2] = [-1.0, -1.0];
const B: [f64; 2] = [1.0, 1.0];

/// Carimba `r` em todos os vértices e cozinha as quinas. Devolve
/// `(quantos vértices saíram, quantos ANCORAS mudaram de sítio)`.
fn round_all(path: &VecPath, r: f64) -> (usize, usize) {
    let mut verts = path.verts.clone();
    for v in &mut verts {
        v.corner_radius = r;
    }
    match round_authored_corners(&verts, path.closed) {
        None => (path.verts.len(), 0),
        Some(out) => {
            let moved = path
                .verts
                .iter()
                .filter(|a| {
                    !out.iter().any(|b| {
                        (b.anchor[0] - a.anchor[0]).abs() < 1e-9
                            && (b.anchor[1] - a.anchor[1]).abs() < 1e-9
                    })
                })
                .count();
            (out.len(), moved)
        }
    }
}

#[test]
#[ignore = "sonda: imprime numeros, nao afirma"]
fn which_vertices_of_a_swept_shape_the_corner_engine_actually_rounds() {
    eprintln!("\n[corner-on-arcs] carimbando o raio em TODOS os vertices e cozinhando\n");
    let radii = [0.02f64, 0.05, 0.1, 0.15, 0.25, 0.4];
    eprint!("  {:>12}  {:>6}", "forma", "verts");
    for r in radii {
        eprint!("  {:>7}", format!("r={r}"));
    }
    eprintln!();
    // A elipse com `sweep`/`inner` é o anel parcial; a pizza e a corda são as irmãs.
    let cases: &[(&str, ShapeKind, Vec<f64>)] = &[
        ("anel parcial", ShapeKind::Ellipse, vec![220.0, 30.0, 0.5]),
        ("pizza", ShapeKind::Pie, vec![70.0, 0.0, 0.0]),
        ("corda", ShapeKind::Segment, vec![120.0, 0.0]),
        ("rosquinha", ShapeKind::Ellipse, vec![0.0, 0.0, 0.55]),
        ("circulo", ShapeKind::Ellipse, vec![0.0, 0.0, 0.0]),
        ("quadrado", ShapeKind::Rectangle, vec![]),
        (
            "seta",
            ShapeKind::ArrowRight,
            ShapeKind::ArrowRight.defaults().to_vec(),
        ),
        (
            "cruz",
            ShapeKind::Cross,
            ShapeKind::Cross.defaults().to_vec(),
        ),
    ];
    for (name, kind, v) in cases {
        let path = cook(*kind, A, B, v);
        eprint!("  {name:>12}  {:>6}", path.verts.len());
        for r in radii {
            eprint!("  {:>7}", round_all(&path, r).0);
        }
        eprintln!();
    }
    eprintln!(
        "\n  LEITURA: `verts` e' o contorno cru; as colunas `r=` sao quantos vertices saem do
  cozimento. Cada quina arredondada troca UM vertice por DOIS, entao a diferenca
  diz quantas quinas o motor achou — 4 no anel parcial, 3 na pizza, 2 na corda.
  Se o CIRCULO tambem crescer, o motor esta' a arredondar vertices de ARCO e a
  regra `carimbe em todos` nao serve."
    );
}

/// **POR VÉRTICE: o motor VÊ a quina, e quanto ele deixa recuar?** A contagem agregada diz
/// *"duas das quatro"*; esta diz QUAIS e por quê.
#[test]
#[ignore = "sonda: imprime numeros, nao afirma"]
fn what_the_engine_sees_at_each_vertex_of_a_cut_ring() {
    for (name, kind, v) in [
        ("anel parcial", ShapeKind::Ellipse, vec![220.0, 30.0, 0.5]),
        ("pizza", ShapeKind::Pie, vec![70.0, 0.0, 0.0]),
    ] {
        let path = cook(kind, A, B, &v);
        eprintln!("\n[corner-on-arcs] {name} — {} vertices", path.verts.len());
        eprintln!(
            "  {:>3}  {:>10}  {:>12}  {:>12}",
            "i", "quina?", "meio-angulo", "recuo max"
        );
        for i in 0..path.verts.len() {
            match corner_at(&path.verts, path.closed, i) {
                None => eprintln!(
                    "  {i:>3}  {:>10}   anchor {:+.3},{:+.3}  in {:+.3},{:+.3}  out {:+.3},{:+.3}  {:?}",
                    "-",
                    path.verts[i].anchor[0],
                    path.verts[i].anchor[1],
                    path.verts[i].in_handle[0],
                    path.verts[i].in_handle[1],
                    path.verts[i].out_handle[0],
                    path.verts[i].out_handle[1],
                    path.verts[i].kind,
                ),
                Some(f) => eprintln!(
                    "  {i:>3}  {:>10}  {:>12.4}  {:>12.4}",
                    "SIM",
                    f.half_angle.to_degrees(),
                    f.max_setback
                ),
            }
        }
    }
    eprintln!(
        "\n  LEITURA: um `-` e' um vertice de ARCO (colinear, nada a arredondar). Um `SIM`
  com `recuo max` minusculo e' uma quina que o motor VE mas mal deixa mexer — e ai'
  o teto e' a CORDA do segmento vizinho, nao o raio pedido."
    );
}

/// **A ARESTA RADIAL de uma fatia é RETA?** A sonda anterior achou que os vértices de ponta
/// de arco são `Smooth` e carregam o handle do arco nos DOIS lados — e o lado que fecha a
/// fatia não é arco, é a reta até ao centro. Se o handle sobra ali, a aresta **abaúla**.
#[test]
#[ignore = "sonda: imprime numeros, nao afirma"]
fn is_the_radial_edge_of_a_slice_actually_straight() {
    /// Um ponto da cúbica em `t` (de Casteljau, escrito aqui para a sonda não depender de
    /// um privado do crate).
    fn at(c: [[f64; 2]; 4], t: f64) -> [f64; 2] {
        let u = 1.0 - t;
        let w = [u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t];
        [
            c[0][0] * w[0] + c[1][0] * w[1] + c[2][0] * w[2] + c[3][0] * w[3],
            c[0][1] * w[0] + c[1][1] * w[1] + c[2][1] * w[2] + c[3][1] * w[3],
        ]
    }
    eprintln!("\n[corner-on-arcs] o desvio da aresta radial contra a reta que ela deveria ser\n");
    eprintln!("  {:>14}  {:>10}  {:>12}", "forma", "raio", "pior desvio");
    for (name, kind, v) in [
        ("pizza 70", ShapeKind::Pie, vec![70.0, 0.0, 0.0]),
        ("pizza 180", ShapeKind::Pie, vec![180.0, 0.0, 0.0]),
        ("anel parcial", ShapeKind::Ellipse, vec![220.0, 30.0, 0.5]),
    ] {
        let path = cook(kind, A, B, &v);
        let n = path.verts.len();
        let mut worst = 0.0f64;
        for i in 0..n {
            let a = path.verts[i];
            let b = path.verts[(i + 1) % n];
            let c = [a.anchor, a.out_handle, b.in_handle, b.anchor];
            // Só as arestas que ligam DOIS pontos a raios diferentes são as radiais.
            let ra = (a.anchor[0].powi(2) + a.anchor[1].powi(2)).sqrt();
            let rb = (b.anchor[0].powi(2) + b.anchor[1].powi(2)).sqrt();
            if (ra - rb).abs() < 1e-6 {
                continue; // é um pedaço de arco, não a aresta radial
            }
            for k in 1..20 {
                let t = f64::from(k) / 20.0;
                let q = at(c, t);
                // ⚠️ **A distância PERPENDICULAR à reta, nunca ao ponto interpolado em `t`.**
                // Uma aresta reta é uma cúbica degenerada (`[a, a, b, b]`), que percorre a
                // MESMA reta com outra parametrização — a 1ª versão desta sonda comparava com
                // `lerp(a, b, t)` e reportava **0,0960** de "desvio" sobre uma reta perfeita,
                // igual para 70° e 180°, que é a assinatura de estar a medir o relógio e não
                // a forma.
                let d = [b.anchor[0] - a.anchor[0], b.anchor[1] - a.anchor[1]];
                let len = (d[0] * d[0] + d[1] * d[1]).sqrt();
                if len < 1e-12 {
                    continue;
                }
                let w = [q[0] - a.anchor[0], q[1] - a.anchor[1]];
                worst = worst.max((w[0] * d[1] - w[1] * d[0]).abs() / len);
            }
        }
        eprintln!("  {name:>14}  {:>10.3}  {worst:>12.4}", 1.0);
    }
    eprintln!(
        "\n  LEITURA: a aresta que fecha uma fatia e' uma RETA. Um desvio de ordem 0,1 num
  raio 1 e' 10% do raio — visivel, e explica por que o motor de quinas nao ve'
  quina nenhuma ali: a tangente e' continua porque o handle do arco sobrou."
    );
}
