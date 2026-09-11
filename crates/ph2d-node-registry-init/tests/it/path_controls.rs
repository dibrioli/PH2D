//! **OS CONTROLES DO *pattern along path* — a composição já exprime quatro dos
//! cinco, e este gate é o que impede a célula de renascer.**
//!
//! A folha 06 linha 46 marcava `P1` no `motion.path` e chamava-o *"o caso mais
//! forte da conferência: o mesmo app responde a mesma pergunta com 6 controles
//! num módulo e 3 no outro"* — o `pattern_along_path` do módulo Vector shipa
//! **Spacing · Start/End · Slide · Offset perpendicular · Side** desde
//! 2026-07-23.
//!
//! ⚠️ **A célula é de 2026-08-10 e ENVELHECEU — a oitava desta conferência.** A
//! razão escrita nela é o que a derruba: *"o deslocamento perpendicular
//! precisaria da NORMAL, que nada publica (`motion.move` é mundo)"*. O irmão
//! **`motion.spline_wrap`** computa a normal (`un = [-ut.y, ut.x]`) e a expõe
//! como `height_scale` — e ele lê a **MESMA curva desenhada** pelo mesmo canal
//! (`external::curve_of`), com a mesma `ph2d_arc_length::at` por baixo. O doc do
//! próprio `motion.path` já o chamava de *"o segundo consumidor da mesma curva
//! desenhada"*; ninguém tinha perguntado a ele.
//!
//! ```text
//! CONTROLE     motion.path(path="Track", count, offset, align)
//! COMPOSIÇÃO   motion.grid(1×N) → motion.move(dy) →
//!                  motion.spline_wrap(path="Track", from, to, offset, height_scale)
//! ```
//!
//! ⇒ **`P1` → `P2`** para quatro: não falta capacidade, falta o GESTO. O
//! **Slide** nem isso — ele **é** o `offset` que o `motion.path` sempre teve, e
//! a célula o listava como ausente. O quinto (**Spacing**) era vão real e fechou
//! no próprio nó (`copies_that_fit`), porque o comprimento já estava na mão dele.
//!
//! Os números vivem na sonda irmã (`measure_path_controls.rs`,
//! `-- --ignored --nocapture`).

use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::Cook;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};

/// A curva DESENHADA que as duas rotas percorrem: um "L" de comprimento 8. O
/// canto torna arco e parâmetro coisas diferentes, e as duas pernas dão uma
/// normal bem definida.
const TRACK: [[f32; 2]; 3] = [[-4.0, 0.0], [0.0, 0.0], [0.0, 4.0]];
const N: usize = 12;

fn cook_with_track(g: &Graph, sink: NodeId) -> Vec<[f32; 2]> {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("todo nó registra");
    let mut cook = Cook::new();
    cook.set_external(
        ph2d_nodegraph::external::curve_of("Track"),
        Stream::new(TRACK.len()).with("P", Column::Vec2(TRACK.to_vec())),
    );
    match cook.cook(g, &reg, sink, 0.0).expect("coza")[0]
        .as_stream()
        .get("P")
    {
        Some(Column::Vec2(p)) => p.clone(),
        _ => Vec::new(),
    }
}

/// O CONTROLE: o nó de três controles.
fn path_chain(offset: f32) -> Vec<[f32; 2]> {
    let mut g = Graph::new();
    let p = g.add_node("motion.path");
    g.set_text_param(p, "path", "Track");
    g.set_param(p, "count", N as f32);
    g.set_param(p, "offset", offset);
    cook_with_track(&g, p)
}

/// A COMPOSIÇÃO: a fila que o `spline_wrap` embrulha na MESMA curva.
fn wrap_chain(from: f32, to: f32, dy: f32) -> Vec<[f32; 2]> {
    let mut g = Graph::new();
    let grid = g.add_node("motion.grid");
    g.set_param(grid, "rows", 1.0);
    g.set_param(grid, "cols", N as f32);
    g.set_param(grid, "gap_x", 1.0);
    let mv = g.add_node("motion.move");
    g.set_param(mv, "dy", dy);
    let sw = g.add_node("motion.spline_wrap");
    g.set_text_param(sw, "path", "Track");
    g.set_param(sw, "from", from);
    g.set_param(sw, "to", to);
    g.set_param(sw, "height_scale", 1.0);
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

/// Projeta um ponto na polilinha: `(distância COM SINAL, fração de arco)`. O
/// sinal é o LADO — positivo à esquerda do sentido de percurso, que é a
/// convenção da normal dos dois nós.
fn project(q: [f32; 2]) -> (f32, f32) {
    let mut cum = vec![0.0f32];
    for w in TRACK.windows(2) {
        let d = ((w[1][0] - w[0][0]).powi(2) + (w[1][1] - w[0][1]).powi(2)).sqrt();
        cum.push(cum.last().unwrap() + d);
    }
    let total = *cum.last().unwrap();
    let (mut best_d, mut best_s, mut sign) = (f32::MAX, 0.0f32, 1.0f32);
    for (i, w) in TRACK.windows(2).enumerate() {
        let (ax, ay, bx, by) = (w[0][0], w[0][1], w[1][0], w[1][1]);
        let (ex, ey) = (bx - ax, by - ay);
        let len2 = ex * ex + ey * ey;
        let t = (((q[0] - ax) * ex + (q[1] - ay) * ey) / len2).clamp(0.0, 1.0);
        let (px, py) = (ax + ex * t, ay + ey * t);
        let d = ((q[0] - px).powi(2) + (q[1] - py).powi(2)).sqrt();
        if d < best_d {
            best_d = d;
            best_s = (cum[i] + t * len2.sqrt()) / total;
            sign = if ex * (q[1] - py) - ey * (q[0] - px) >= 0.0 {
                1.0
            } else {
                -1.0
            };
        }
    }
    (best_d * sign, best_s)
}

fn span(pts: &[[f32; 2]]) -> f32 {
    let (lo, hi) = pts.iter().fold((f32::MAX, f32::MIN), |(lo, hi), q| {
        let s = project(*q).1;
        (lo.min(s), hi.max(s))
    });
    hi - lo
}

/// **O CONTROLE que torna os dois gates abaixo legíveis:** as duas rotas põem os
/// elementos **na mesma curva desenhada**. Sem isto, *"o composto recorta"*
/// poderia ser satisfeito por uma rota que simplesmente não pousa na curva.
#[test]
fn both_routes_walk_the_very_same_drawn_curve() {
    for (tag, pts) in [
        ("controle", path_chain(0.0)),
        ("composto", wrap_chain(0.0, 1.0, 0.0)),
    ] {
        assert_eq!(pts.len(), N, "{tag}: {N} elementos");
        let worst = pts.iter().fold(0.0f32, |m, q| m.max(project(*q).0.abs()));
        assert!(worst < 1e-3, "{tag} pousa na curva (pior desvio {worst})");
    }
}

/// **START/END sai por COMPOSIÇÃO, e o controle NÃO consegue.**
///
/// ⚠️ As duas metades são necessárias. Sozinha, a primeira ficaria verde se o
/// `motion.path` também soubesse recortar — e aí a célula estaria certa sobre um
/// vão que não existe. A segunda varre o ÚNICO knob que ele tem e mostra que o
/// vão dele não se move.
#[test]
fn the_interval_is_trimmed_by_composition_and_not_by_the_node() {
    let full = span(&wrap_chain(0.0, 1.0, 0.0));
    let half = span(&wrap_chain(0.25, 0.75, 0.0));
    assert!(full > 0.99, "a curva inteira mede ~1,0 de arco: {full}");
    assert!(
        (half - 0.5).abs() < 0.02,
        "from=0,25 to=0,75 recorta para ~0,5 de arco, não {half}"
    );

    // E o controle, varrendo o único knob que ele tem: o vão NÃO muda.
    let base = span(&path_chain(0.0));
    for off in [0.25f32, 0.5, 0.75] {
        let s = span(&path_chain(off));
        assert!(
            (s - base).abs() < 1e-3,
            "o `offset` DESLIZA e não recorta: em {off} o vão é {s}, e em 0 era {base}"
        );
    }
}

/// **O DESLOCAMENTO PERPENDICULAR e o LADO saem por COMPOSIÇÃO** — a normal que
/// a célula dizia que *"nada publica"*.
///
/// O oráculo é a distância **COM SINAL** à curva: a magnitude prova o
/// deslocamento e o sinal prova o lado. Uma régua sem sinal ficaria verde com os
/// dois lados colapsados num só.
#[test]
fn the_normal_offset_and_the_side_come_out_of_composition() {
    let on = wrap_chain(0.0, 1.0, 0.0);
    let left = wrap_chain(0.0, 1.0, 0.5);
    let right = wrap_chain(0.0, 1.0, -0.5);

    assert!(
        on.iter().all(|q| project(*q).0.abs() < 1e-3),
        "sem deslocamento os elementos ficam NA curva"
    );
    // ⚠️ O lado côncavo do canto encolhe a distância medida (o ponto mais
    // próximo passa a ser o VÉRTICE), então a barra é `> 0,3` e não `≈ 0,5`:
    // é geometria de curva paralela, não folga de gate. O que é categórico —
    // e o que o gate afirma — é o SINAL.
    assert!(
        left.iter().all(|q| project(*q).0 > 0.3),
        "dy positivo põe TODOS de um lado: {:?}",
        left.iter().map(|q| project(*q).0).collect::<Vec<_>>()
    );
    assert!(
        right.iter().all(|q| project(*q).0 < -0.3),
        "dy negativo põe TODOS do OUTRO lado"
    );

    // E o controle não tem knob nenhum para isto: ele fica na curva, sempre.
    assert!(
        path_chain(0.0).iter().all(|q| project(*q).0.abs() < 1e-3),
        "o `motion.path` sozinho não desloca da curva"
    );
}
