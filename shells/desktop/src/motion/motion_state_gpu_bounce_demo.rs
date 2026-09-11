//! **NEM TODA BATIDA DEVOLVE O MESMO** (`PH2D_GPU_COOK_DEMO=99`) — a célula do
//! `restitution_randomness` (doc 89, folha 13) montada como documento pronto para smoke.
//!
//! ⚠️ **Esta cena PRECISA de Play** — ela é uma simulação, e o que se compara é a ALTURA a que
//! cada disco volta depois de bater. Uma foto parada não a mostra.
//!
//! ```text
//!   ESQUERDA   Restitution Randomness = 0   todos quicam a' MESMA altura (o que sempre foi)
//!   DIREITA    Restitution Randomness = 1   cada um quica a` sua
//! ```
//!
//! ⚠️ **Os discos são todos IGUAIS aqui, ao contrário da cena `=28`** — e é o oposto da razão
//! dela. Lá a variação de tamanho era o oráculo (só um raio por-elemento alinha bordas de
//! tamanhos diferentes); aqui ela seria RUÍDO: um disco maior cai com outra história e o olho
//! atribuiria a diferença de altura ao tamanho em vez de ao acaso. *A mesma decisão — variar ou
//! não — troca de lado quando a pergunta troca.*
//!
//! ⚠️ **E é a MESMA cadeia dos dois lados**: mesma grade, mesma gravidade, mesmo chão, mesma
//! restituição autorada, mesma semente. Os dois `sim.collide` diferem em **UM** param.

use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::NodeId;

/// A altura do chão, nas duas metades — as duas fileiras batem na MESMA linha.
pub(super) const FLOOR: f32 = -2.6;
/// Quantos discos por fileira. **Nove**: o acaso é uma distribuição, e três amostras não
/// mostram uma distribuição — mostram três números.
pub(super) const COLS: f32 = 9.0;
/// O espaçamento e o tamanho de cada disco. Iguais de propósito (ver o doc do módulo).
const GAP_X: f32 = 0.75;
const SIZE: f32 = 0.34;
/// De que altura eles caem.
const DROP_Y: f32 = 3.2;
/// **A restituição autorada** — o TETO da lei, e o que a fileira da esquerda inteira usa.
/// `0,75` porque abaixo disso a 2.ª quicada já não se vê e a cena deixa de ter o que comparar.
pub(super) const RESTITUTION: f32 = 0.75;
/// A semente — a MESMA nos dois lados, para o que muda ser só o param.
const SEED: f32 = 4.0;

/// **NEM TODA BATIDA DEVOLVE O MESMO** (`PH2D_GPU_COOK_DEMO=99`).
pub(super) fn build_gpu_bounce_demo_document(
    doc: &mut MotionDoc,
    reg: &NodeRegistry,
) -> Option<Vec<NodeId>> {
    use ph2d_nodegraph::graph::{Edge, Pos};
    let g = &mut doc.graph;

    // Uma metade: grade -> alto -> tamanho -> zona -> gravidade -> passo -> colisor.
    // `restitution_randomness` é o ÚNICO param que difere entre as duas chamadas.
    let mut half = |x_offset: f32, randomness: f32, row: f32| -> Option<NodeId> {
        let grid = g.add_node("motion.grid");
        g.set_param(grid, "rows", 1.0);
        g.set_param(grid, "cols", COLS);
        g.set_param(grid, "gap_x", GAP_X);
        let lift = g.add_node("motion.transform");
        g.set_param(lift, "offset_x", x_offset);
        g.set_param(lift, "offset_y", DROP_Y);
        let size = g.add_node("motion.scale");
        g.set_param(size, "amount", SIZE);

        let zone = g.add_node("sim.zone");
        let wind = g.add_node("force.wind");
        g.set_param(wind, "angle", 270.0); // para baixo
        g.set_param(wind, "strength", 3.0);
        // ⚠️ **A RAJADA VAI A ZERO, e sem isto a cena mentia.** O `force.wind` tem `gust = 0,3`
        // por omissão — um ruído por elemento — e com ele a fileira de CONTROLO já caía
        // desalinhada: medido, os nove discos separavam-se `1,12` unidade **antes de tocarem no
        // chão**. O olho leria essa diferença como a lei que a cena existe para mostrar.
        // *Uma cena que compara duas metades tem de zerar tudo o que já as separava.*
        g.set_param(wind, "gust", 0.0);
        let step = g.add_node("sim.step");
        // ⚠️ **Sem amortecimento**, ao contrário da `=28`: ali o oráculo era o REPOUSO e o
        // damping ajudava a chegar lá; aqui ele comeria a diferença de altura que é a cena.
        g.set_param(step, "damping", 1.0);
        let ground = g.add_node("sim.collide");
        g.set_param(ground, "shape", 0.0); // Floor
        g.set_param(ground, "height", FLOOR);
        g.set_param(ground, "restitution", RESTITUTION);
        g.set_param(ground, "friction", 0.0);
        g.set_param(ground, "restitution_randomness", randomness);
        g.set_param(ground, "seed", SEED);

        for (i, n) in [grid, lift, size, zone].into_iter().enumerate() {
            g.set_pos(
                n,
                Pos {
                    x: 80.0 + i as f32 * 160.0,
                    y: row,
                },
            );
        }
        for (i, n) in [wind, step, ground].into_iter().enumerate() {
            g.set_pos(
                n,
                Pos {
                    x: 260.0 + i as f32 * 160.0,
                    y: row + 130.0,
                },
            );
        }
        for (a, ap, b, bp, delayed) in [
            (grid, 0u16, lift, 0u16, false),
            (lift, 0, size, 0, false),
            (size, 0, zone, 0, false),
            // A entrada de estado que o motor gerencia.
            (zone, 0, wind, 0, true),
            (wind, 0, step, 0, false),
            (step, 0, ground, 0, false),
            (ground, 0, zone, 1, false),
        ] {
            g.connect(Edge {
                from: (a, ap),
                to: (b, bp),
                delayed,
            })
            .ok()?;
        }
        let out = g.add_node("motion.output");
        g.set_pos(
            out,
            Pos {
                x: 740.0,
                y: row + 130.0,
            },
        );
        g.connect(Edge {
            from: (zone, 0),
            to: (out, 0),
            delayed: false,
        })
        .ok()?;
        Some(out)
    };

    let span = (COLS - 1.0) * GAP_X;
    let left = half(-span * 0.5 - 1.2, 0.0, 0.0)?;
    let right = half(span * 0.5 + 1.2, 1.0, 420.0)?;
    g.validate(reg).ok()?;
    Some(vec![left, right])
}

#[cfg(test)]
#[path = "motion_state_gpu_bounce_demo_tests.rs"]
mod tests;
