//! **A CALHA** (`PH2D_GPU_COOK_DEMO=29`) — o 3º e último P0 da folha 13 do doc 89 montado como
//! documento pronto para smoke: *o plano do colisor era HORIZONTAL, sempre*.
//!
//! A folha tentou as duas composições e refutou as duas — `motion.rotate` escreve só a coluna
//! `rot` e nunca move `P`, e encadear colisores constrói uma **escada**, jamais uma rampa —,
//! então a forma passou a carregar a própria orientação. O que a cena mostra é a diferença entre
//! *segurar* e *transportar*:
//!
//! ```text
//!   a chuva cai  ->  a RAMPA (angulo -18) a leva para a direita  ->  a PAREDE (angulo 90) a para
//! ```
//!
//! ⚠️ **As duas superfícies são o MESMO `sim.collide`**, com o mesmo `shape` — só o ângulo
//! difere. Uma parede não é uma forma nova: é a mesma meia-reta virada um quarto de volta, e é
//! por isso que ela custou zero linha de kernel.
//!
//! ⚠️ **O `offset` é distância à ORIGEM ao longo da normal** (a forma de Hesse), não uma
//! coordenada `y` — e é essa escolha que torna a parede exprimível. A alternativa óbvia (*"o
//! plano pivota em torno de `(0, height)`"*) lê melhor numa rampa rasa e **prende toda parede em
//! `x = 0` para sempre**: a 90° aquele pivô está SOBRE o plano, então o botão desliza a parede ao
//! longo de si mesma e não faz nada.
//!
//! A rampa passa pela ORIGEM (`offset = 0`), o que a torna legível sem conta nenhuma: ela desce
//! `RAMP_SLOPE` unidade por unidade de `x`.

use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::NodeId;

/// A inclinação da rampa, em graus. **Negativa desce para a DIREITA** — a normal é o vetor
/// "para cima" do mundo girado pelo ângulo, então um ângulo negativo o inclina para a direita e
/// a superfície acompanha.
pub(super) const RAMP_DEG: f32 = -18.0;
/// Onde a parede fica, em `x`. O `sim.collide` a recebe como `offset = -WALL_X` porque a normal
/// de 90° aponta para a ESQUERDA (`n = (-1, 0)`), e o mundo é o lado para onde a normal aponta.
pub(super) const WALL_X: f32 = 4.0;
/// Quantos discos a chuva tem. **27**, o suficiente para a pilha na quina ser uma PILHA e não
/// dois discos empilhados.
pub(super) const ROWS: f32 = 3.0;
/// Ver [`ROWS`].
pub(super) const COLS: f32 = 9.0;
/// O tamanho de cada disco. O colisor roda em `Sprite Size`, então o raio é a metade disto —
/// as duas features da folha 13 na mesma cena, e o disco pousa SOBRE a rampa em vez de afundar.
pub(super) const DISC: f32 = 0.5;

/// **A CALHA** (`PH2D_GPU_COOK_DEMO=29`).
///
/// `ramp_deg` existe para o GATE: com `0.0` a cena vira o chão horizontal de ontem, e é a
/// diferença entre as duas corridas que prova que a rampa TRANSPORTA. O smoke usa
/// [`RAMP_DEG`].
pub(super) fn build_gpu_ramp_demo_document(
    doc: &mut MotionDoc,
    reg: &NodeRegistry,
    ramp_deg: f32,
) -> Option<Vec<NodeId>> {
    use ph2d_nodegraph::graph::{Edge, Pos};
    let g = &mut doc.graph;

    let grid = g.add_node("motion.grid");
    g.set_param(grid, "rows", ROWS);
    g.set_param(grid, "cols", COLS);
    g.set_param(grid, "gap_x", 0.8);
    g.set_param(grid, "gap_y", 0.8);

    // A chuva nasce ALTA e à ESQUERDA — acima da parte alta da rampa, para ter para onde
    // descer. Ver a queda é o que torna o repouso legível como pouso.
    let lift = g.add_node("motion.transform");
    g.set_param(lift, "offset_x", -1.5);
    g.set_param(lift, "offset_y", 4.0);

    let size = g.add_node("motion.scale");
    g.set_param(size, "amount", DISC);

    let zone = g.add_node("sim.zone");
    let wind = g.add_node("force.wind");
    g.set_param(wind, "angle", 270.0); // para baixo
    g.set_param(wind, "strength", 6.0);
    let step = g.add_node("sim.step");
    g.set_param(step, "damping", 0.85);

    // A RAMPA. `offset = 0` a faz passar pela origem do mundo, então a linha dela é
    // simplesmente `y = -RAMP_SLOPE * x` e a cena pode falar sobre ela sem re-derivar nada.
    let ramp = g.add_node("sim.collide");
    g.set_param(ramp, "shape", 0.0); // Plane
    g.set_param(ramp, "angle", ramp_deg);
    g.set_param(ramp, "height", 0.0);
    g.set_param(ramp, "restitution", 0.0); // desliza, não quica: o transporte é o oráculo
    g.set_param(ramp, "friction", 0.02); // pouco atrito, senão a rampa segura em vez de levar
    g.set_param(ramp, "radius_from", 2.0); // Sprite Size
    g.set_param(ramp, "size_scale", 1.0);

    // A PAREDE — o MESMO nó, um quarto de volta. `n = (-1, 0)` ⇒ o mundo é `x <= WALL_X`.
    let wall = g.add_node("sim.collide");
    g.set_param(wall, "shape", 0.0);
    g.set_param(wall, "angle", 90.0);
    g.set_param(wall, "height", -WALL_X);
    g.set_param(wall, "restitution", 0.0);
    g.set_param(wall, "friction", 0.4);
    g.set_param(wall, "radius_from", 2.0);
    g.set_param(wall, "size_scale", 1.0);

    let out = g.add_node("motion.output");

    for (i, n) in [grid, lift, size, zone].into_iter().enumerate() {
        g.set_pos(
            n,
            Pos {
                x: 80.0 + i as f32 * 170.0,
                y: 260.0,
            },
        );
    }
    for (i, n) in [wind, step, ramp, wall, out].into_iter().enumerate() {
        g.set_pos(
            n,
            Pos {
                x: 420.0 + i as f32 * 170.0,
                y: 400.0,
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
        (step, 0, ramp, 0, false),
        // Encadear colisores FUNCIONA (cada um é Pure e lê/escreve `P`/`vel`) — o que não
        // funcionava era a cadeia dar uma RAMPA, porque todo plano era horizontal.
        (ramp, 0, wall, 0, false),
        (wall, 0, zone, 1, false),
        (zone, 0, out, 0, false),
    ] {
        g.connect(Edge {
            from: (a, ap),
            to: (b, bp),
            delayed,
        })
        .ok()?;
    }
    g.validate(reg).ok()?;
    Some(vec![out])
}

#[cfg(test)]
#[path = "motion_state_gpu_ramp_demo_tests.rs"]
mod tests;
