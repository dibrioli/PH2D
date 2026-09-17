//! **DEIXAR A FÍSICA DECIDIR** (`PH2D_GPU_COOK_DEMO=113`) — a cena de smoke do ciclo 5
//! ([doc 108](../../docs/Motion%20Nodes/108_ciclo_5_simulacao.md)).
//!
//! ## O que se vê
//!
//! Uma chuva de peças a cair sobre **um bloco**, a empilhar-se em cima dele e a escorregar
//! pelos lados. E ela **recomeça sozinha**, porque o `sim.zone` está em `Loop` — sem isso o
//! monte assentava uma vez e todo passo do smoke daí em diante mexeria num knob sem nada
//! para mexer.
//!
//! ## ⚠️ Por que o colisor é um `Box`, e não o chão
//!
//! A alça de canvas do ciclo (W1a) faz **três** coisas — mover, girar e redimensionar — e
//! num `Plane` só duas delas são lidas: um plano é infinito, então a alça de tamanho escreve
//! num param que aquela forma ignora (*inerte, não mentiroso*, e está declarado no
//! [`crate::field_gizmo`]). ⛔ **Um tutorial não aponta para um controlo inerte.** Num `Box`
//! as três são vivas, e cada uma tem efeito visível: o monte muda de sítio, escorrega, ou
//! passa a caber.
//!
//! ## O que ela está de facto a demonstrar
//!
//! 1. **O `sim.collide` tem ALÇA DE CANVAS** (W1a) — ele era literalmente a coisa que se põe
//!    num sítio, e punha-se com dois sliders. ⚠️ E é o primeiro nó cuja alça **muda de forma
//!    com um param**: as quatro formas medem-se com params diferentes.
//! 2. **O `Acts As` diz o que o `Mode` não dizia** (W2) — `Force` acelera para sempre;
//!    `Target Velocity` empurra cada vez menos e **pára** quando a peça alcança o vento. É a
//!    diferença entre cair e cair a velocidade constante, e vê-se de relance.
//! 3. **O cartão do `Wind` tem SECÇÕES** (W3) — `Gust` e `Timing`, com quatro params soltos
//!    em cima que são a razão de existir do nó.
//!
//! ⚠️ **Se nada cair**, o transporte está parado: esta cena é uma simulação e precisa de
//! **Play**, como a `=99`.

use crate::motion_demo_legend::Caption;
use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::NodeId;

/// Quantas peças caem: `12 × 5` = 60. Poucas o bastante para se ver **uma** peça a bater, e
/// muitas o bastante para o monte ser um monte.
const COLS: f32 = 12.0;
const ROWS: f32 = 5.0;
/// O vão entre elas, e o tamanho de cada uma. O vão é `2,6 ×` a peça para a chuva ler como
/// chuva em vez de um bloco sólido a descer.
const GAP: f32 = 0.42;
const PECA: f32 = 0.16;
/// De que altura a chuva parte.
const ALTURA: f32 = 2.4;

/// **O BLOCO** — onde ele está e quanto mede. A largura é pouco mais de **metade** do vão da
/// chuva de propósito: com ele tão largo como ela, nada escorrega pelos lados e a cena
/// ensinaria que um colisor apanha tudo.
const BLOCO_Y: f32 = -1.7;
const BLOCO_W: f32 = 1.5;
const BLOCO_H: f32 = 0.3;

/// A gravidade — `force.wind` apontado para baixo. ⚠️ **Com `gust = 0` este nó É gravidade**,
/// e o doc dele di-lo; a rajada entra no passo 3 do smoke, não antes.
const GRAVIDADE: f32 = 5.0;
/// A rajada nasce a ZERO: com o default (`0,3`) a chuva já cai torta, e o passo 3 do smoke não
/// teria diferença nenhuma para mostrar. *Uma cena que existe para acender um knob tem de o
/// entregar apagado.*
const RAJADA: f32 = 0.0;

/// Quanto tempo dura cada queda, e a pausa antes da seguinte.
const DURACAO: f32 = 2.6;
const PAUSA: f32 = 0.5;

/// O índice de `Loop` na linha `Life Cycle` do `sim.zone` (`Forever | Once | Loop`).
const LIFE_LOOP: f32 = 2.0;

/// **O índice de um valor de enum, PERGUNTADO ao registo em vez de digitado.**
///
/// ⛔ As outras cenas de simulação desta casa escrevem o número à mão com o nome ao lado num
/// comentário (`set_param(ground, "shape", 0.0); // Floor`). Aqui não: o `sim.collide` tem
/// **quatro** formas e o ciclo 5 acabou de lhes dar uma alça que muda com a escolhida — um
/// literal que envelhecesse poria a cena a montar a forma errada **sem erro nenhum**, e o
/// smoke ensinaria a alça errada. *Um índice de enum é uma posição numa lista que outra
/// pessoa pode reordenar.*
pub(super) fn indice_de(reg: &NodeRegistry, no: &str, param: &str, valor: &str) -> Option<f32> {
    use ph2d_node_registry::ParamWidget;
    let tid = ph2d_nodegraph::node::NodeTypeId::of(no);
    let hint = reg.param_ui(tid)?.iter().find(|h| h.param == param)?;
    let ParamWidget::Enum { labels } = hint.widget else {
        return None;
    };
    // ⚠️⚠️ **A comparação é do TEXTO, e sem isto a falha é MUDA:** o array carrega chaves
    // desde a migração do HR-15, `position` não acharia nada, o `?` devolveria `None` e a cena
    // simplesmente não escreveria o param — sem erro, sem aviso, com a demo a abrir errada.
    let i = labels.iter().position(|l| ph2d_i18n::tr(l) == valor)?;
    #[expect(
        clippy::cast_precision_loss,
        reason = "um indice de enum, sempre pequeno"
    )]
    Some(i as f32)
}

/// A legenda que a cena pousa no canvas.
pub(super) fn captions() -> Vec<Caption> {
    vec![
        Caption::new([BLOCO_W + 0.35, BLOCO_Y], "arraste ESTE bloco"),
        Caption::new(
            [-(COLS - 1.0) * GAP * 0.5, BLOCO_Y - 1.0],
            "Wind ▸ Acts As: Force -> Target Velocity",
        ),
    ]
}

/// `grid → alto → tamanho → zone ⇄ (wind → step → collide) → output`.
///
/// ⚠️ **A aresta `zone → wind` é `delayed`** — é a entrada de estado que o motor gere, e é o
/// que fecha o laço da simulação. A cena `=99` usa exactamente a mesma forma.
pub(super) fn build(doc: &mut MotionDoc, reg: &NodeRegistry) -> Option<Vec<NodeId>> {
    use ph2d_nodegraph::graph::{Edge, Pos};
    let g = &mut doc.graph;

    let grid = g.add_node("motion.grid");
    g.set_param(grid, "rows", ROWS);
    g.set_param(grid, "cols", COLS);
    g.set_param(grid, "gap_x", GAP);
    g.set_param(grid, "gap_y", GAP);

    let alto = g.add_node("motion.transform");
    g.set_param(alto, "offset_y", ALTURA);

    let tamanho = g.add_node("motion.scale");
    g.set_param(tamanho, "amount", PECA);

    // ⭐ O RELÓGIO do grafo, em `Loop`: a queda recomeça sozinha (ver o cabeçalho).
    let zone = g.add_node("sim.zone");
    g.set_param(zone, "mode", LIFE_LOOP);
    g.set_param(zone, "duration", DURACAO);
    g.set_param(zone, "loop_delay", PAUSA);

    // ⭐ A GRAVIDADE, e o cartão onde o `Acts As` mora.
    let vento = g.add_node("force.wind");
    g.set_param(vento, "angle", 270.0);
    g.set_param(vento, "strength", GRAVIDADE);
    g.set_param(vento, "gust", RAJADA);

    let passo = g.add_node("sim.step");

    // ⭐ O COLISOR, em `Box` — a alça de canvas com as três metades vivas (ver o cabeçalho).
    let bloco = g.add_node("sim.collide");
    g.set_param(
        bloco,
        "shape",
        indice_de(reg, "sim.collide", "shape", "Box")?,
    );
    g.set_param(bloco, "center_x", 0.0);
    g.set_param(bloco, "center_y", BLOCO_Y);
    g.set_param(bloco, "box_width", BLOCO_W);
    g.set_param(bloco, "box_height", BLOCO_H);
    // Quase nada de quique e bastante atrito: o que se quer ver é um MONTE, e uma peça que
    // salta de volta ao ar sai da cena antes de o smoke chegar ao passo 2.
    g.set_param(bloco, "restitution", 0.1);
    g.set_param(bloco, "friction", 0.8);

    let out = g.add_node("motion.output");

    for (i, n) in [grid, alto, tamanho, zone].into_iter().enumerate() {
        g.set_pos(
            n,
            Pos {
                #[expect(clippy::cast_precision_loss, reason = "um indice de cartao")]
                x: 40.0 + i as f32 * 176.0,
                y: 120.0,
            },
        );
    }
    for (i, n) in [vento, passo, bloco, out].into_iter().enumerate() {
        g.set_pos(
            n,
            Pos {
                #[expect(clippy::cast_precision_loss, reason = "um indice de cartao")]
                x: 260.0 + i as f32 * 176.0,
                y: 400.0,
            },
        );
    }

    for (a, ap, b, bp, delayed) in [
        (grid, 0u16, alto, 0u16, false),
        (alto, 0, tamanho, 0, false),
        (tamanho, 0, zone, 0, false),
        (zone, 0, vento, 0, true),
        (vento, 0, passo, 0, false),
        (passo, 0, bloco, 0, false),
        (bloco, 0, zone, 1, false),
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
#[path = "motion_state_sim_demo_tests.rs"]
mod tests;
