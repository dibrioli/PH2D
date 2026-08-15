//! **A DIREÇÃO** (`PH2D_GPU_COOK_DEMO=38`) — a cena da linha da [doc 89 §10.0] que **cinco
//! famílias** (1·4·5·6·15) citaram como inexprimível: *"vire para onde está indo"*.
//!
//! Irmão de `motion_state_conferencia_demos` (o pai bate no teto de LOC da shell); o assunto é
//! o mesmo, a casa é outra.
//!
//! ## O que a cena põe lado a lado
//!
//! **O MESMO redemoinho, duas vezes.** As duas nuvens correm a MESMA simulação a partir da
//! MESMA grade — mesmas forças, mesmo relógio, mesmo tudo —, e a única diferença é que a da
//! direita tem dois nós a mais: `value.attribute(Direction) → motion.drive(Rotation)`.
//!
//! ⚠️ **As peças são TRAÇOS e não pontos, e isto é o que torna a cena julgável.** Um quadrado
//! rodado 90° é idêntico a si mesmo e um disco é idêntico em toda rotação: sobre qualquer um
//! deles as duas metades desenhariam a mesma imagem e o smoke diria *"funciona"* sobre um canal
//! inerte. Com `motion.scale` de eixos DESTRAVADOS cada peça é um risco deitado, e aí:
//!
//! - **à ESQUERDA** os riscos ficam todos HORIZONTAIS enquanto deslizam — cartas a derrapar de
//!   lado, que é exactamente o defeito que todo sistema de partículas tem antes deste canal;
//! - **à DIREITA** cada risco aponta ao longo do próprio caminho, e o redemoinho vira um campo
//!   de linhas de fluxo.
//!
//! ## O que a cena NÃO prova, e por isso não finge
//!
//! Ela não distingue graus de radianos pelo olho (uma resposta em radianos giraria os riscos
//! para ângulos errados, mas ainda os giraria) — quem separa isso é o gate
//! `the_angle_points_along_the_velocity_once_the_consumer_converts_it`, que leva o número pela
//! conversão do consumidor. A cena responde a pergunta que só o olho responde: *o desenho lê
//! como movimento?*

use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::NodeId;

/// Quantas peças por nuvem, por lado da grade.
pub(crate) const SIDE: f32 = 26.0;
/// O deslocamento de cada nuvem a partir do centro (unidades de mundo).
pub(crate) const SPREAD: f32 = 3.4;

/// `grid → scale → integrate ← (vortex → drag) → [attr → drive] → transform → output`, duas
/// vezes. Devolve os DOIS sinks.
pub(super) fn build_direction_demo_document(
    doc: &mut MotionDoc,
    reg: &NodeRegistry,
) -> Option<Vec<NodeId>> {
    use ph2d_nodegraph::graph::Pos;
    let g = &mut doc.graph;
    let mut sinks = Vec::new();

    for k in 0..2u8 {
        let aligned = k == 1;
        let row = 120.0 + f32::from(k) * 320.0;

        let grid = g.add_node("motion.grid");
        g.set_param(grid, "rows", SIDE);
        g.set_param(grid, "cols", SIDE);
        g.set_param(grid, "gap_x", 0.20);
        g.set_param(grid, "gap_y", 0.20);

        // O TRAÇO: eixos destravados, largo e fino. Sem isto a cena não é julgável (ver o
        // doc do módulo) — um quadrado rodado é o mesmo quadrado.
        let dash = g.add_node("motion.scale");
        g.set_param(dash, "uniform", 0.0);
        g.set_param(dash, "amount", 1.7);
        g.set_param(dash, "amount_y", 0.22);

        let ig = g.add_node("motion.integrate");
        // O redemoinho: vórtice + arrasto. Sem o arrasto um vórtice puro espirala para fora
        // por deriva centrífuga (a mesma razão que a cena `=3` documenta).
        let vortex = g.add_node("force.vortex");
        g.set_param(vortex, "strength", 11.0);
        g.set_param(vortex, "radius", 9.0);
        g.set_param(vortex, "clockwise", 1.0);
        let drag = g.add_node("force.drag");
        g.set_param(drag, "coefficient", 0.5);

        // O deslocamento é o ÚLTIMO passo, depois da sim: as duas nuvens correm no MESMO
        // lugar, sob as MESMAS forças, e só o desenho é separado. Offsetar antes poria as
        // duas em pontos diferentes do vórtice e a comparação deixaria de ser uma.
        let place = g.add_node("motion.transform");
        g.set_param(place, "offset_x", if aligned { SPREAD } else { -SPREAD });
        let out = g.add_node("motion.output");

        for (i, n) in [grid, dash, ig, place, out].into_iter().enumerate() {
            g.set_pos(
                n,
                Pos {
                    x: 80.0 + i as f32 * 200.0,
                    y: row,
                },
            );
        }
        for (i, n) in [vortex, drag].into_iter().enumerate() {
            g.set_pos(
                n,
                Pos {
                    x: 120.0 + i as f32 * 180.0,
                    y: row + 150.0,
                },
            );
        }

        wire(g, grid, 0, dash, 0, false).ok()?;
        wire(g, dash, 0, ig, 0, false).ok()?;
        // O laço que o artista nunca desenha: o estado do tique anterior na cabeça da cadeia
        // de forças. `add_node` não o plumba — o editor o faz ao SOLTAR o nó.
        wire(g, ig, 0, vortex, 0, true).ok()?;
        wire(g, vortex, 0, drag, 0, false).ok()?;
        wire(g, drag, 0, ig, 1, false).ok()?;

        let tail = if aligned {
            let attr = g.add_node("value.attribute");
            g.set_text_param(attr, "attr", "vel");
            g.set_param(attr, "mode", ph2d_node_value_attribute::MODE_ANGLE as f32);
            let drive = g.add_node("motion.drive");
            g.set_param(drive, "channel", 2.0); // Rotation
            g.set_param(drive, "mode", 1.0); // Set — o ângulo É a rotação, não um acréscimo
            g.set_pos(
                attr,
                Pos {
                    x: 480.0,
                    y: row + 150.0,
                },
            );
            g.set_pos(
                drive,
                Pos {
                    x: 660.0,
                    y: row + 150.0,
                },
            );
            wire(g, ig, 0, attr, 0, false).ok()?;
            wire(g, ig, 0, drive, 0, false).ok()?;
            wire(g, attr, 0, drive, 1, false).ok()?;
            drive
        } else {
            ig
        };
        wire(g, tail, 0, place, 0, false).ok()?;
        wire(g, place, 0, out, 0, false).ok()?;
        sinks.push(out);
    }

    g.validate(reg).ok()?;
    Some(sinks)
}

/// Uma aresta. Função LIVRE e não closure: uma closure que captura `g` o empresta até ao fim
/// do escopo, e o corpo do laço ainda precisa dele para `add_node`/`set_param`.
fn wire(
    g: &mut ph2d_nodegraph::graph::Graph,
    a: NodeId,
    ap: u16,
    b: NodeId,
    bp: u16,
    delayed: bool,
) -> Result<(), ph2d_nodegraph::graph::EdgeError> {
    g.connect(ph2d_nodegraph::graph::Edge {
        from: (a, ap),
        to: (b, bp),
        delayed,
    })
}

#[cfg(test)]
#[path = "motion_state_conferencia_demos_direction_tests.rs"]
mod tests;
