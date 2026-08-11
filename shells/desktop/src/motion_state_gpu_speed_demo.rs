//! **O TETO DE VELOCIDADE** (`PH2D_GPU_COOK_DEMO=31`) — o P1 do speed limit da folha 13 do doc 89
//! montado como documento pronto para smoke.
//!
//! A folha mediu a assimetria que o tornava inexprimível: o `value.attribute` **LÊ** `vel` como
//! *Speed*, e escrever velocidade por-elemento não era possível em cadeia nenhuma — o
//! `motion.drive` não tem canal de `vel`, e o `force.drag`, que é o único que a ESCALA, recebe o
//! coeficiente como *param*, ou seja um número por tique e não por elemento.
//!
//! A cena é um **atrator forte**: sem teto, tudo o que passa perto do centro é estilingado a uma
//! velocidade absurda e some de quadro. Com teto, a mesma nuvem fica.
//!
//! ⚠️ **O limite mora no `sim.step` e não num nó a jusante, e essa colocação é a feature:** ele
//! roda ENTRE a atualização da velocidade e a da posição, então capa a **distância que o elemento
//! anda neste tique**. Um `sim.speed_limit` depois do passo caparia o número que o elemento
//! *reporta* e deixaria a posição já ter andado — um tique atrasado, por construção, e é
//! exatamente o tique em que o arremessado atravessa a parede.
//!
//! ⚠️ **O A/B é o próprio controle que a wave shipou:** selecione o `Simulation Step` e ponha
//! *Speed Limit* em **0**. Zero é DESLIGADO (não "congele"), então a nuvem volta a se estilingar.

use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::NodeId;

/// O teto, em unidades por segundo. **MEDIDO** contra o que a cena de fato produz
/// (`probe_speed_ceiling`, varrendo 0 / 6 / 20): sem teto o pico é **18,20 u/s** e o elemento
/// mais distante chega a **24,77** do centro — fora de quadro. Com **6** o pico é exatamente
/// 6,00 e o mais distante fica em **2,86**: um rodamoinho contido, **8,7× menos alcance**.
///
/// ⚠️ **E a varredura trouxe um CONTROLE de graça:** com teto **20** os números são idênticos aos
/// de teto 0 até o dígito (18,20 e 24,77) — *um teto acima do que a sim produz é
/// indistinguível de teto nenhum*. É exatamente por isso que `max_speed` não tem `ParamHardMax`:
/// um teto maior é menos teto, e degrada para o default em vez de disfuncionar.
pub(super) const LIMIT: f32 = 6.0;
/// A força do atrator. Alta de propósito: é a vizinhança do centro que produz a velocidade
/// absurda, e um atrator manso não teria o que capar.
pub(super) const PULL: f32 = 60.0;
/// A força do vórtice — a componente tangencial que faz disto um rodamoinho em vez de um poço.
pub(super) const SWIRL: f32 = 40.0;
pub(super) const ROWS: f32 = 12.0;
pub(super) const COLS: f32 = 12.0;

/// **O TETO DE VELOCIDADE** (`PH2D_GPU_COOK_DEMO=31`).
///
/// `limit` existe para o GATE e para a sonda: com `0.0` a cena é a mesma sem teto nenhum, que é
/// o braço contra o qual o número medido tem sentido.
pub(super) fn build_gpu_speed_demo_document(
    doc: &mut MotionDoc,
    reg: &NodeRegistry,
    limit: f32,
) -> Option<Vec<NodeId>> {
    use ph2d_nodegraph::graph::{Edge, Pos};
    let g = &mut doc.graph;

    let grid = g.add_node("motion.grid");
    g.set_param(grid, "rows", ROWS);
    g.set_param(grid, "cols", COLS);
    g.set_param(grid, "gap_x", 0.7);
    g.set_param(grid, "gap_y", 0.7);

    let size = g.add_node("motion.scale");
    g.set_param(size, "amount", 0.18);

    let zone = g.add_node("sim.zone");

    // O ATRATOR, no centro — é a vizinhança dele que produz a velocidade absurda.
    let pull = g.add_node("force.attractor");
    g.set_param(pull, "target_x", 0.0);
    g.set_param(pull, "target_y", 0.0);
    g.set_param(pull, "strength", PULL);
    g.set_param(pull, "radius", 6.0);

    // O VÓRTICE, no mesmo centro: ele dá a componente TANGENCIAL que transforma a queda num
    // rodamoinho. Sem ele o atrator sozinho só junta tudo num ponto, e "um ponto" não é uma
    // leitura de nada — o que se quer ver é a nuvem RODANDO contida contra a nuvem que escapa.
    let swirl = g.add_node("force.vortex");
    g.set_param(swirl, "center_x", 0.0);
    g.set_param(swirl, "center_y", 0.0);
    g.set_param(swirl, "strength", SWIRL);
    g.set_param(swirl, "radius", 6.0);

    let step = g.add_node("sim.step");
    g.set_param(step, "max_speed", limit);

    let out = g.add_node("motion.output");

    for (i, n) in [grid, size, zone, pull, swirl, step, out]
        .into_iter()
        .enumerate()
    {
        g.set_pos(
            n,
            Pos {
                x: 80.0 + i as f32 * 180.0,
                y: 300.0,
            },
        );
    }

    for (a, ap, b, bp, delayed) in [
        (grid, 0u16, size, 0u16, false),
        (size, 0, zone, 0, false),
        (zone, 0, pull, 0, true),
        (pull, 0, swirl, 0, false),
        (swirl, 0, step, 0, false),
        (step, 0, zone, 1, false),
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
#[path = "motion_state_gpu_speed_demo_tests.rs"]
mod tests;
