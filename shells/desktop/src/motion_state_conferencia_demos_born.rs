//! **ONDE AS COISAS NASCEM** — a cena `=93` (doc 89, folha 01, as oito últimas).
//!
//! Cinco pares. As quatro primeiras fileiras são **paradas**; só a última anda.
//!
//! | par | esquerda | direita |
//! |---|---|---|
//! | `motion.grid` | retângulo | **`Shape = Circle`** — a forma RECORTA, e a contagem cai |
//! | `motion.scatter` | retângulo | **`Shape = Ring`** — o amostrador EMPACOTA: a mesma contagem |
//! | `motion.distribute_poisson` | densidade uniforme | **`Density Falloff`** — a borda fica mais RALA, não esburacada |
//! | `motion.voronoi` | `Distance = Euclidean` | **`Chebyshev`** — as células puxam para quadrados |
//! | `motion.emitter` | toda partícula vive o mesmo | **`Life Random`** — cada uma morre na sua hora |
//!
//! ⚠️ **As duas primeiras fileiras são a mesma pergunta com respostas OPOSTAS, e é de
//! propósito:** um reticulado não se dobra para caber num círculo (só pode perder
//! pontos), e um amostrador só muda onde o dardo cai (não perde nenhum). *Uma cena que
//! mostrasse só uma delas ensinaria a lei errada sobre a outra.*

use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId, Pos};

/// O vão entre as duas colunas e entre as cinco linhas.
const GAP_X: f32 = 5.0;
const GAP_Y: f32 = 3.6;
/// A extensão que as quatro distribuições partilham — um lado só, para as formas se
/// compararem entre si.
const EXTENT: f32 = 3.0;
/// O buraco que o anel do `motion.scatter` autora.
const RING_HOLE: f32 = 0.45;
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

/// **A GRADE** — a rede que a forma RECORTA.
fn grid(g: &mut Graph, ey: f32, circular: bool) -> NodeId {
    let n = g.add_node("motion.grid");
    g.set_pos(n, Pos { x: 400.0, y: ey });
    g.set_param(n, "rows", 15.0);
    g.set_param(n, "cols", 15.0);
    let gap = EXTENT / 14.0;
    g.set_param(n, "gap_x", gap);
    g.set_param(n, "gap_y", gap);
    if circular {
        g.set_param(n, ph2d_motion_region::SHAPE, 1.0);
    }
    n
}

/// **O ESPALHAMENTO** — o amostrador que a forma REDISTRIBUI.
fn scatter(g: &mut Graph, ey: f32, ring: bool) -> NodeId {
    let n = g.add_node("motion.scatter");
    g.set_pos(n, Pos { x: 400.0, y: ey });
    g.set_param(n, "count", 200.0);
    g.set_param(n, "width", EXTENT);
    g.set_param(n, "height", EXTENT);
    g.set_param(n, "seed", 5.0);
    if ring {
        g.set_param(n, ph2d_motion_region::SHAPE, 2.0);
        g.set_param(n, ph2d_motion_region::INNER, RING_HOLE);
    }
    n
}

/// **O POISSON** — o disco onde a densidade vira o RAIO.
fn poisson(g: &mut Graph, ey: f32, graded: bool) -> NodeId {
    let n = g.add_node("motion.distribute_poisson");
    g.set_pos(n, Pos { x: 400.0, y: ey });
    g.set_param(n, "radius", 0.16);
    g.set_param(n, "width", EXTENT);
    g.set_param(n, "height", EXTENT);
    g.set_param(n, "seed", 2.0);
    // ⚠️ **As DUAS são círculos** — o que muda entre elas é só a gradação. Comparar um
    // quadrado com um disco graduado mediria duas coisas ao mesmo tempo.
    g.set_param(n, ph2d_motion_region::SHAPE, 1.0);
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

/// Monta a cena. Devolve os dez sinks, em pares.
pub(crate) fn build_born_demo_document(
    doc: &mut MotionDoc,
    registry: &NodeRegistry,
) -> Option<Vec<NodeId>> {
    let g = &mut doc.graph;
    let rgb = [
        [0.46, 0.72, 1.0],
        [1.0, 0.74, 0.3],
        [0.62, 1.0, 0.66],
        [1.0, 0.6, 0.72],
        [0.82, 0.7, 1.0],
    ];
    let mut sinks = Vec::with_capacity(10);
    for (row, colour) in rgb.iter().enumerate() {
        for col in 0..2 {
            let ey = (row * 2 + col) as f32 * 260.0;
            let on = col == 1;
            let head = match row {
                0 => grid(g, ey, on),
                1 => scatter(g, ey, on),
                2 => poisson(g, ey, on),
                3 => voronoi(g, ey, on),
                _ => emitter(g, ey, on),
            };
            let at = [
                if col == 0 { -GAP_X } else { GAP_X },
                GAP_Y * 2.0 - row as f32 * GAP_Y,
            ];
            sinks.push(finish(g, head, *colour, at, ey)?);
        }
    }
    g.validate(registry).ok()?;
    Some(sinks)
}

/// Os rótulos das dez bandas, na ordem em que a cena as monta.
pub(crate) fn band_labels() -> impl Iterator<Item = (usize, &'static str)> {
    [
        "GRADE retangular -- a rede de sempre, 225 pontos",
        "GRADE Shape=Circle -- a forma RECORTA: os cantos caem e a contagem baixa",
        "ESPALHAMENTO retangular -- 200 pontos bem repartidos",
        "ESPALHAMENTO Shape=Ring -- os MESMOS 200 pontos, agora dentro do anel",
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
pub(crate) fn captions() -> Vec<crate::motion_demo_legend::Caption> {
    band_labels()
        .map(|(k, label)| {
            let (row, col) = (k / 2, k % 2);
            let at = [
                if col == 0 { -GAP_X } else { GAP_X },
                GAP_Y * 2.0 - row as f32 * GAP_Y + GAP_Y * 0.44,
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
pub(crate) fn authored() -> (f32, f32, f32) {
    (RING_HOLE, FALLOFF, LIFE_RANDOM)
}

#[cfg(test)]
#[path = "motion_state_conferencia_demos_born_tests.rs"]
mod tests;
