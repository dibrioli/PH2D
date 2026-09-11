//! **SONDA — os seis controles do *pattern along path*, medidos contra o que a
//! composição já exprime.**
//!
//! A folha 06 linha 46 marca `P1` no `motion.path` e chama-o *"o caso mais forte
//! da conferência: o mesmo app responde a mesma pergunta com 6 controles num
//! módulo e 3 no outro"* — o `pattern_along_path` do módulo Vector shipa
//! **Spacing · Start/End · Slide · Offset perpendicular · Side** desde 2026-07-23,
//! e o `motion.path` tem `count · offset · align`.
//!
//! ⚠️ **A célula é de 2026-08-10 e a primeira coisa de toda wave desta conferência
//! é MEDIR se a composição já exprime o item.** Esta sonda faz isso, e o candidato
//! não é um nó novo: é o **irmão** `motion.spline_wrap`, que o doc do próprio
//! `motion.path` chama de *"o segundo consumidor da mesma curva desenhada"*.
//!
//! As duas rotas:
//!
//! ```text
//! CONTROLE     motion.path(path="Track", count, offset, align)
//! COMPOSIÇÃO   motion.grid(1×N) → motion.move(dy) →
//!                  motion.spline_wrap(path="Track", from, to, offset, height_scale, follow_rotation)
//! ```
//!
//! Rode com `cargo test -p ph2d-node-registry-init --test measure_path_controls -- --ignored --nocapture`.

use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::Cook;
use ph2d_nodegraph::graph::{Edge, Graph};

/// A curva DESENHADA que as duas rotas percorrem: um "L" de comprimento 8, cujo
/// canto torna arco e parâmetro coisas diferentes e cuja normal é bem definida
/// em cada perna.
const TRACK: [[f32; 2]; 3] = [[-4.0, 0.0], [0.0, 0.0], [0.0, 4.0]];
const N: usize = 12;

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("todo nó registra");
    reg
}

fn cook_with_track(g: &Graph, sink: ph2d_nodegraph::graph::NodeId) -> Vec<[f32; 2]> {
    let reg = registry();
    let mut cook = Cook::new();
    cook.set_external(
        ph2d_nodegraph::external::curve_of("Track"),
        Stream::new(TRACK.len()).with("P", Column::Vec2(TRACK.to_vec())),
    );
    let v = cook.cook(g, &reg, sink, 0.0).expect("coza");
    match v[0].as_stream().get("P") {
        Some(Column::Vec2(p)) => p.clone(),
        _ => Vec::new(),
    }
}

/// O CONTROLE: o nó de três controles.
fn path_chain(count: usize, offset: f32) -> Vec<[f32; 2]> {
    let mut g = Graph::new();
    let p = g.add_node("motion.path");
    g.set_text_param(p, "path", "Track");
    g.set_param(p, "count", count as f32);
    g.set_param(p, "offset", offset);
    g.set_param(p, "align", 1.0);
    cook_with_track(&g, p)
}

/// A COMPOSIÇÃO: a fila que o `spline_wrap` embrulha na MESMA curva.
fn wrap_chain(count: usize, from: f32, to: f32, offset: f32, dy: f32) -> Vec<[f32; 2]> {
    let mut g = Graph::new();
    let grid = g.add_node("motion.grid");
    g.set_param(grid, "rows", 1.0);
    g.set_param(grid, "cols", count as f32);
    g.set_param(grid, "gap_x", 1.0);
    let mv = g.add_node("motion.move");
    g.set_param(mv, "dy", dy);
    let sw = g.add_node("motion.spline_wrap");
    g.set_text_param(sw, "path", "Track");
    g.set_param(sw, "from", from);
    g.set_param(sw, "to", to);
    g.set_param(sw, "offset", offset);
    g.set_param(sw, "height_scale", 1.0);
    g.set_param(sw, "follow_rotation", 1.0);
    for (a, b) in [(grid, mv), (mv, sw)] {
        g.connect(Edge {
            from: (a, 0),
            to: (b, 0),
            delayed: false,
        })
        .expect("aresta");
    }
    cook_with_track(&g, sw)
}

/// Projeta um ponto na polilinha: devolve `(distância COM SINAL, fração de arco)`.
/// O sinal é o lado — positivo à ESQUERDA do sentido de percurso, que é a
/// convenção da normal dos dois nós (`un = [-ut.y, ut.x]`).
fn project(q: [f32; 2]) -> (f32, f32) {
    let mut cum = vec![0.0f32];
    for w in TRACK.windows(2) {
        let d = ((w[1][0] - w[0][0]).powi(2) + (w[1][1] - w[0][1]).powi(2)).sqrt();
        cum.push(cum.last().unwrap() + d);
    }
    let total = *cum.last().unwrap();
    let (mut best_d, mut best_s, mut best_sign) = (f32::MAX, 0.0f32, 1.0f32);
    for (i, w) in TRACK.windows(2).enumerate() {
        let (ax, ay) = (w[0][0], w[0][1]);
        let (bx, by) = (w[1][0], w[1][1]);
        let (ex, ey) = (bx - ax, by - ay);
        let len2 = ex * ex + ey * ey;
        let t = (((q[0] - ax) * ex + (q[1] - ay) * ey) / len2).clamp(0.0, 1.0);
        let (px, py) = (ax + ex * t, ay + ey * t);
        let d = ((q[0] - px).powi(2) + (q[1] - py).powi(2)).sqrt();
        if d < best_d {
            best_d = d;
            best_s = (cum[i] + t * len2.sqrt()) / total;
            // cross(tangente, q − p): positivo ⇒ à esquerda.
            best_sign = if ex * (q[1] - py) - ey * (q[0] - px) >= 0.0 {
                1.0
            } else {
                -1.0
            };
        }
    }
    (best_d * best_sign, best_s)
}

fn span(pts: &[[f32; 2]]) -> (f32, f32) {
    pts.iter().fold((f32::MAX, f32::MIN), |(lo, hi), q| {
        let (_, s) = project(*q);
        (lo.min(s), hi.max(s))
    })
}

fn worst_off_curve(pts: &[[f32; 2]]) -> f32 {
    pts.iter().fold(0.0f32, |m, q| m.max(project(*q).0.abs()))
}

#[test]
#[ignore = "sonda: imprime numeros, nao afirma"]
fn measure_what_composition_already_expresses() {
    println!("\n=== a curva: L de comprimento 8, {N} elementos ===\n");

    // ---- 1. As duas rotas pousam na MESMA curva? ----
    let ctrl = path_chain(N, 0.0);
    let comp = wrap_chain(N, 0.0, 1.0, 0.0, 0.0);
    println!("1. NEUTRO — as duas pousam na curva?");
    println!(
        "   controle  n={:2}  fora da curva {:.6}  arco [{:.3} .. {:.3}]",
        ctrl.len(),
        worst_off_curve(&ctrl),
        span(&ctrl).0,
        span(&ctrl).1
    );
    println!(
        "   composto  n={:2}  fora da curva {:.6}  arco [{:.3} .. {:.3}]",
        comp.len(),
        worst_off_curve(&comp),
        span(&comp).0,
        span(&comp).1
    );

    // ---- 2. START/END: o composto RECORTA o intervalo? ----
    println!("\n2. START/END — o intervalo do arco");
    for (f, t) in [(0.0, 1.0), (0.25, 0.75), (0.5, 1.0)] {
        let c = wrap_chain(N, f, t, 0.0, 0.0);
        let (lo, hi) = span(&c);
        println!(
            "   composto from={f:.2} to={t:.2}  ⇒  arco [{lo:.3} .. {hi:.3}]  vao {:.3}",
            hi - lo
        );
    }
    println!("   — e o CONTROLE, varrendo o unico knob que ele tem:");
    for off in [0.0f32, 0.25, 0.5] {
        let c = path_chain(N, off);
        let (lo, hi) = span(&c);
        println!(
            "   controle offset={off:.2}          ⇒  arco [{lo:.3} .. {hi:.3}]  vao {:.3}",
            hi - lo
        );
    }

    // ---- 3. OFFSET PERPENDICULAR e LADO ----
    println!("\n3. PERPENDICULAR + LADO — distancia COM SINAL a curva");
    for dy in [0.0f32, 0.5, -0.5] {
        let c = wrap_chain(N, 0.0, 1.0, 0.0, dy);
        let d: Vec<f32> = c.iter().map(|q| project(*q).0).collect();
        let lo = d.iter().cloned().fold(f32::MAX, f32::min);
        let hi = d.iter().cloned().fold(f32::MIN, f32::max);
        println!("   composto dy={dy:+.2}  ⇒  distancia com sinal [{lo:+.3} .. {hi:+.3}]");
    }
    let cd: Vec<f32> = ctrl.iter().map(|q| project(*q).0).collect();
    println!(
        "   controle          ⇒  distancia com sinal [{:+.3} .. {:+.3}]  (nao ha knob)",
        cd.iter().cloned().fold(f32::MAX, f32::min),
        cd.iter().cloned().fold(f32::MIN, f32::max)
    );

    // ---- 4. SLIDE: os dois deslizam? ----
    println!("\n4. SLIDE — o conjunto anda pelo arco");
    for off in [0.0f32, 0.25] {
        let a = path_chain(N, off);
        let b = wrap_chain(N, 0.0, 1.0, off, 0.0);
        println!(
            "   offset={off:.2}  controle arco[0]={:.3}  composto arco[0]={:.3}",
            project(a[0]).1,
            project(b[0]).1
        );
    }

    // ---- 5. SPACING: a CONTAGEM automatica ----
    println!("\n5. SPACING — a contagem derivada do comprimento");
    println!("   o comprimento do arco NAO e publicado por nenhuma coluna nem canal:");
    println!(
        "   as duas rotas pedem a CONTAGEM em numero, e nenhum `value.*` sabe o tamanho da curva."
    );
}
