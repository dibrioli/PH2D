//! **ONDE AS COISAS NASCEM** — a cena `=93` (doc 89, folha 01).
//!
//! Três pares. As duas primeiras fileiras são **paradas**; só a última anda.
//!
//! | par | esquerda | direita |
//! |---|---|---|
//! | `motion.distribute_poisson` | densidade uniforme | **`Density Falloff`** — a borda fica mais RALA, não esburacada |
//! | `motion.voronoi` | `Distance = Euclidean` | **`Chebyshev`** — as células puxam para quadrados |
//! | `motion.emitter` | toda partícula vive o mesmo | **`Life Random`** — cada uma morre na sua hora |
//!
//! ## ⛔⛔ Duas fileiras SAÍRAM daqui, e a razão é o produto e não a cena
//!
//! Ela tinha **cinco** pares, e os dois primeiros eram `motion.grid` (retângulo contra
//! `Shape = Circle`) e `motion.scatter` (retângulo contra `Shape = Ring`). Juntos ensinavam a lei
//! que o §5.0 do roteador guarda — *um RETICULADO recorta e um AMOSTRADOR redistribui* —, e eram
//! a melhor metade desta cena.
//!
//! **Ordem do dono, 2026-09-19: *«tire de todos»*.** O param `Shape` saiu dos quatro
//! distribuidores, logo **a lei deixou de existir no produto** e as duas fileiras passariam a
//! mostrar a mesma figura dos dois lados.
//!
//! ⚠️ **Elas foram CORTADAS e não substituídas por um contraste novo.** Inventar aqui uma
//! diferença que ninguém pediu seria autorar produto por conta própria — e uma cena mais curta que
//! é toda verdade vale mais do que uma do tamanho antigo com uma lição fabricada dentro. *Uma cena
//! que ensina o contrário do que acontece é pior que uma cena ausente* (§5.0).

use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId, Pos};

/// O vão entre as duas colunas e entre as cinco linhas.
const GAP_X: f32 = 5.0;
const GAP_Y: f32 = 3.6;
/// A extensão que as quatro distribuições partilham — um lado só, para as formas se
/// compararem entre si.
const EXTENT: f32 = 3.0;
/// A gradação que o par do Poisson autora.
const FALLOFF: f32 = 1.0;
/// A variância de vida que o par do emissor autora.
const LIFE_RANDOM: f32 = 0.8;

fn wire(g: &mut Graph, from: NodeId, fp: u16, to: NodeId, tp: u16, delayed: bool) -> Option<()> {
    g.connect(Edge {
        from: (from, fp),
        to: (to, tp),
        delayed,
    })
    .ok()
}

/// Leva a banda ao quadrante, pinta-a e fecha.
fn finish(g: &mut Graph, head: NodeId, rgb: [f32; 3], at: [f32; 2], ey: f32) -> Option<NodeId> {
    let mv = g.add_node("motion.move");
    g.set_pos(mv, Pos { x: 700.0, y: ey });
    g.set_param(mv, "dx", at[0]);
    g.set_param(mv, "dy", at[1]);
    wire(g, head, 0, mv, 0, false)?;
    let tint = g.add_node("motion.tint");
    g.set_pos(tint, Pos { x: 840.0, y: ey });
    g.set_param(tint, "r", rgb[0]);
    g.set_param(tint, "g", rgb[1]);
    g.set_param(tint, "b", rgb[2]);
    wire(g, mv, 0, tint, 0, false)?;
    let out = g.add_node("motion.output");
    g.set_pos(out, Pos { x: 980.0, y: ey });
    wire(g, tint, 0, out, 0, false)?;
    Some(out)
}

/// **O POISSON** — a caixa onde a densidade vira o RAIO.
fn poisson(g: &mut Graph, ey: f32, graded: bool) -> NodeId {
    let n = g.add_node("motion.distribute_poisson");
    g.set_pos(n, Pos { x: 400.0, y: ey });
    g.set_param(n, "radius", 0.16);
    g.set_param(n, "width", EXTENT);
    g.set_param(n, "height", EXTENT);
    g.set_param(n, "seed", 2.0);
    // ⚠️ **As DUAS são a MESMA caixa** — o que muda entre elas é só a gradação. Medir duas
    // coisas ao mesmo tempo era o que um par de formas diferentes faria; até 2026-09-19 as duas
    // eram discos pela mesma razão, e hoje são rectângulos porque o `Shape` saiu.
    if graded {
        g.set_param(
            n,
            ph2d_node_motion_distribute_poisson::DENSITY_FALLOFF,
            FALLOFF,
        );
    }
    n
}

/// **O VORONOI** — o CVT cuja métrica decide de que célula cada ponto do plano é.
fn voronoi(g: &mut Graph, ey: f32, chebyshev: bool) -> NodeId {
    let n = g.add_node("motion.voronoi");
    g.set_pos(n, Pos { x: 400.0, y: ey });
    g.set_param(n, "count", 90.0);
    g.set_param(n, "width", EXTENT);
    g.set_param(n, "height", EXTENT);
    g.set_param(n, "seed", 9.0);
    g.set_param(n, "iterations", 14.0);
    if chebyshev {
        g.set_param(n, ph2d_node_motion_voronoi::METRIC, 2.0);
    }
    n
}

/// **O EMISSOR** — a fileira que ANDA.
fn emitter(g: &mut Graph, ey: f32, varied: bool) -> NodeId {
    let n = g.add_node("motion.emitter");
    g.set_pos(n, Pos { x: 400.0, y: ey });
    g.set_param(n, "rate", 60.0);
    g.set_param(n, "life", 1.8);
    g.set_param(n, "speed", 1.1);
    g.set_param(n, "angle", 90.0);
    g.set_param(n, "spread", 150.0);
    g.set_param(n, "max", 256.0);
    g.set_param(n, "size", 0.09);
    g.set_param(n, "seed", 3.0);
    if varied {
        g.set_param(n, ph2d_node_motion_emitter::LIFE_RANDOM, LIFE_RANDOM);
    }
    n
}

/// Monta a cena. Devolve os seis sinks, em pares.
pub fn build_born_demo_document(
    doc: &mut MotionDoc,
    registry: &NodeRegistry,
) -> Option<Vec<NodeId>> {
    let g = &mut doc.graph;
    let rgb = [[0.62, 1.0, 0.66], [1.0, 0.6, 0.72], [0.82, 0.7, 1.0]];
    let mut sinks = Vec::with_capacity(6);
    for (row, colour) in rgb.iter().enumerate() {
        for col in 0..2 {
            let ey = (row * 2 + col) as f32 * 260.0;
            let on = col == 1;
            let head = match row {
                0 => poisson(g, ey, on),
                1 => voronoi(g, ey, on),
                _ => emitter(g, ey, on),
            };
            let at = [
                if col == 0 { -GAP_X } else { GAP_X },
                GAP_Y - row as f32 * GAP_Y,
            ];
            sinks.push(finish(g, head, *colour, at, ey)?);
        }
    }
    g.validate(registry).ok()?;
    Some(sinks)
}

/// Os rótulos das seis bandas, na ordem em que a cena as monta.
pub fn band_labels() -> impl Iterator<Item = (usize, &'static str)> {
    [
        "POISSON uniforme -- todo par a' mesma distancia minima",
        "POISSON Density Falloff -- a borda fica mais RALA (espacamento maior), nao esburacada",
        "VORONOI Euclidean -- as celulas arredondam",
        "VORONOI Chebyshev -- a mesma semente, e as celulas puxam para quadrados",
        "EMISSOR de vida unica -- todas as particulas morrem juntas, na mesma borda",
        "EMISSOR Life Random -- cada uma morre na sua hora, e a borda desmancha-se",
    ]
    .into_iter()
    .enumerate()
}

/// **As fichas desta cena, no canvas** — função PURA, medida pelo gate da legenda.
pub fn captions() -> Vec<crate::motion_demo_legend::Caption> {
    band_labels()
        .map(|(k, label)| {
            let (row, col) = (k / 2, k % 2);
            let at = [
                if col == 0 { -GAP_X } else { GAP_X },
                GAP_Y - row as f32 * GAP_Y + GAP_Y * 0.44,
            ];
            crate::motion_demo_legend::Caption::new(at, short_of(label))
        })
        .collect()
}

/// A ficha curta: o que está ANTES do primeiro `--`.
fn short_of(label: &'static str) -> &'static str {
    match label.find(" --") {
        Some(i) => &label[..i],
        None => label,
    }
}

/// Os números que a mensagem do smoke cita.
pub fn authored() -> (f32, f32) {
    (FALLOFF, LIFE_RANDOM)
}

#[cfg(test)]
#[path = "motion_state_conferencia_demos_born_tests.rs"]
mod tests;
