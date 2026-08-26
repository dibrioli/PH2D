//! **UM OBSTÁCULO COM QUINAS** (`PH2D_GPU_COOK_DEMO=101`) — a célula *mais formas de
//! colisor* (doc 89, folha 13).
//!
//! ⚠️ **Esta cena PRECISA de Play.**
//!
//! ```text
//!   ESQUERDA   Disc   o obstaculo que ja' existia -- tudo escorrega por uma curva
//!   DIREITA    Box    o novo, INCLINADO -- ha' um telhado plano e duas quinas
//! ```
//!
//! ⚠️ **A célula dizia que encadear colisores dava «a união» das formas, e a sonda mostrou
//! que dá o CONTRÁRIO** (`measure_collider_shapes`): quatro planos encadeados põem as 121
//! peças de uma grelha **dentro** do rectângulo — encadear é uma **conjunção** de respostas,
//! logo um CONTENTOR. O que faltava era o **obstáculo**, que é a operação oposta e não se
//! compõe a partir dela.
//!
//! ⚠️ **A pré-condição desta célula tinha caído sem ninguém reconferir:** ela dizia *"depois
//! do ângulo"*, e o `angle` do plano já existia havia dias. A caixa nasce inclinada aqui de
//! propósito — é a metade que estava bloqueada.

use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::NodeId;

/// As formas: a que já existia e a nova.
const SHAPE_DISC: f32 = 1.0;
const SHAPE_BOX: f32 = 3.0;
/// Quantas peças caem em cada lado.
pub(super) const COLS: f32 = 13.0;
const GAP_X: f32 = 0.34;
const SIZE: f32 = 0.2;
/// De que altura elas caem, e onde o obstáculo está.
const DROP_Y: f32 = 3.4;
const OBSTACLE_Y: f32 = -0.4;
/// **A INCLINAÇÃO da caixa**, em graus. `20°` é o suficiente para o telhado escoar para um
/// lado sem que as peças passem por cima da quina sem a tocar.
pub(super) const TILT: f32 = 20.0;
/// O tamanho do obstáculo. O disco tem o raio; a caixa tem a largura e a altura, e a largura
/// é o DIÂMETRO do disco para os dois lados estorvarem a mesma faixa da chuva.
pub(super) const DISC_R: f32 = 1.1;
const BOX_W: f32 = DISC_R * 2.0;
const BOX_H: f32 = 0.55;
/// Um chão bem em baixo, para as peças pararem em vez de caírem para sempre.
const FLOOR: f32 = -3.2;

/// **UM OBSTÁCULO COM QUINAS** (`PH2D_GPU_COOK_DEMO=101`).
pub(super) fn build_gpu_box_demo_document(
    doc: &mut MotionDoc,
    reg: &NodeRegistry,
) -> Option<Vec<NodeId>> {
    use ph2d_nodegraph::graph::{Edge, Pos};
    let g = &mut doc.graph;

    let mut half = |x_offset: f32, shape: f32, row: f32| -> Option<NodeId> {
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
        g.set_param(wind, "angle", 270.0);
        g.set_param(wind, "strength", 3.0);
        // A rajada a zero pela mesma razão da cena `=99`: ela separava a chuva antes de ela
        // chegar ao obstáculo, e o olho atribuiria a diferença à forma.
        g.set_param(wind, "gust", 0.0);
        let step = g.add_node("sim.step");
        g.set_param(step, "damping", 0.9);

        // O OBSTÁCULO — a única diferença entre os dois lados.
        let obst = g.add_node("sim.collide");
        g.set_param(obst, "shape", shape);
        g.set_param(obst, "center_x", x_offset);
        g.set_param(obst, "center_y", OBSTACLE_Y);
        g.set_param(obst, "radius", DISC_R);
        g.set_param(obst, "box_width", BOX_W);
        g.set_param(obst, "box_height", BOX_H);
        g.set_param(obst, "angle", TILT);
        g.set_param(obst, "restitution", 0.1);
        g.set_param(obst, "friction", 0.2);
        // E o chão, igual dos dois lados.
        let ground = g.add_node("sim.collide");
        g.set_param(ground, "shape", 0.0); // Plane
        g.set_param(ground, "height", FLOOR);
        g.set_param(ground, "restitution", 0.0);
        g.set_param(ground, "friction", 0.6);

        for (i, n) in [grid, lift, size, zone].into_iter().enumerate() {
            g.set_pos(
                n,
                Pos {
                    x: 80.0 + i as f32 * 150.0,
                    y: row,
                },
            );
        }
        for (i, n) in [wind, step, obst, ground].into_iter().enumerate() {
            g.set_pos(
                n,
                Pos {
                    x: 240.0 + i as f32 * 150.0,
                    y: row + 130.0,
                },
            );
        }
        for (a, ap, b, bp, delayed) in [
            (grid, 0u16, lift, 0u16, false),
            (lift, 0, size, 0, false),
            (size, 0, zone, 0, false),
            (zone, 0, wind, 0, true),
            (wind, 0, step, 0, false),
            (step, 0, obst, 0, false),
            (obst, 0, ground, 0, false),
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
                x: 900.0,
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

    let left = half(-2.6, SHAPE_DISC, 0.0)?;
    let right = half(2.6, SHAPE_BOX, 420.0)?;
    g.validate(reg).ok()?;
    Some(vec![left, right])
}

#[cfg(test)]
#[path = "motion_state_gpu_box_demo_tests.rs"]
mod tests;
