//! ⭐⭐⭐ **A MISTURA NA PLACA** (`PH2D_MOTION_OBJ_SMOKE=13`) — a cena que faltava para se VER a
//! cura de 2026-09-22 (`ph2d-render`, gate `a_mistura_do_device_chega_ao_pixel`).
//!
//! ⛔⛔ **Ela existe porque NENHUMA cena do catálogo podia mostrar aquilo, e isso foi MEDIDO.**
//! O defeito vivia só na rota da PLACA, e as duas famílias de cenas falham por lados opostos:
//! - as do `PH2D_GPU_COOK_DEMO` que vão à placa são **grelhas sem forma** — desde a lei do dono
//!   de 19/09 elas desenham só os gizmos das posições, logo não há peça onde uma mistura se veja
//!   (o 1.º smoke desta cura foi dado numa delas, a `=6`, e o dono viu *«apenas gizmos do grid»*);
//! - as do `PH2D_MOTION_OBJ_SMOKE` têm imagem, e **nenhuma vai à placa**: o carimbo
//!   `source.object → duplicator → output` pára no duplicador, que não despacha estágio nenhum
//!   (`PH2D_MOTION_ROUTE_LOG=1` na `=1`: *«fronteira sem estagio de GPU que despache»*) — e na CPU
//!   a mistura sempre funcionou; a `=9`, que é a vitrine do estilo do sink, tem OITO saídas.
//!
//! ⇒ esta põe um estágio da placa DEPOIS do carimbo (`motion.move`), o que faz a rota ser a
//! HÍBRIDA: o carimbo na CPU, o resto e o desenho na placa — onde a mistura se perdia.
//!
//! ⚠️ **As cópias SOBREPÕEM-SE de propósito** (ladrilho `0,8`, passo `0,5`): sobre o fundo escuro,
//! uma cópia sozinha em `Add` e em `Normal` quase não se distinguem; é onde duas se empilham que o
//! `Add` clareia e o `Multiply` escurece.
//!
//! ⚠️ **E o ladrilho é do ÁTLAS** (`texture_id 0`), que é o caso subtil da cura: a partição de
//! texturas ficava VAZIA, e vazia queria dizer «em `Normal`».

use ph2d_nodegraph::graph::{Edge, Graph, NodeId, Pos};

/// O tag de `Add` — a posição dele no seletor do `motion.output` (`Normal · Add · …`).
const ADD: f32 = 1.0;

/// Monta a cena: o objecto, o grafo, e o cartão da SAÍDA já escolhido.
pub(super) fn run(cx: &mut crate::motion_scene_ctx::MotionSceneCtx<'_>) {
    super::art::spawn_sprite(cx.sim);
    let out = build(&mut cx.motion.doc.graph, super::OBJECT);
    cx.motion.sinks.push(out);
    let _ = cx
        .tools
        .set_active(&ph2d_editor_core::ToolId::new("motion"));
    eprintln!(
        "[motion.obj smoke =13] A MISTURA NA PLACA. A esquerda esta o 'Object' sozinho; a direita, \
         16 copias dele que se SOBREPOEM, e o cartao 'Output' ja escolhido com o 'Blend' em 'Add' \
         -- onde duas copias se empilham a cor CLAREIA. Troque o 'Blend' para 'Normal': as zonas \
         empilhadas deixam de clarear. 'Multiply' escurece-as. Se nada mudar ao trocar, e' o \
         defeito antigo (a mistura perdia-se no desenho). Com PH2D_MOTION_ROUTE_LOG=1 o terminal \
         diz a rota, e ela tem de ser a HIBRIDA."
    );
}

/// `source.object → duplicator ← grid → move → output`, com o `Blend` da saída em `Add`.
fn build(graph: &mut Graph, name: &str) -> NodeId {
    let src = graph.add_node("source.object");
    let grid = graph.add_node("motion.grid");
    let dup = graph.add_node("motion.duplicator");
    let mv = graph.add_node("motion.move");
    let out = graph.add_node("motion.output");
    graph.set_pos(src, Pos { x: 0.0, y: -260.0 });
    graph.set_pos(grid, Pos { x: 0.0, y: -140.0 });
    graph.set_pos(
        dup,
        Pos {
            x: 210.0,
            y: -200.0,
        },
    );
    graph.set_pos(
        mv,
        Pos {
            x: 420.0,
            y: -200.0,
        },
    );
    graph.set_pos(
        out,
        Pos {
            x: 630.0,
            y: -200.0,
        },
    );
    let wire = |g: &mut Graph, a: NodeId, ap: u16, b: NodeId, bp: u16| {
        g.connect(Edge {
            from: (a, ap),
            to: (b, bp),
            delayed: false,
        })
        .expect("connect");
    };
    wire(graph, src, 0, dup, 0);
    wire(graph, grid, 0, dup, 1);
    wire(graph, dup, 0, mv, 0);
    wire(graph, mv, 0, out, 0);

    graph.set_text_param(src, "object", name);
    // 4×4 com o passo MENOR que o ladrilho (0,8): cada cópia cobre um pedaço das vizinhas.
    graph.set_param(grid, "rows", 4.0);
    graph.set_param(grid, "cols", 4.0);
    graph.set_param(grid, "gap_x", GAP);
    graph.set_param(grid, "gap_y", GAP);
    // O estágio da PLACA, e ele tem um trabalho que se vê: põe as cópias ao lado do original.
    graph.set_param(mv, "dx", 2.5);
    graph.set_param(mv, "dy", 0.0);
    graph.set_param(out, ph2d_eval_motion::SINK_BLEND_PARAM, ADD);
    graph.set_label(src, "The Object");
    graph.set_label(dup, "Stamp On Grid");
    graph.set_label(mv, "Beside The Original");

    // Nasce com a SAÍDA escolhida: o `Blend` é a linha que o roteiro manda trocar.
    ph2d_panel_motion_graph::request_graph_selection(vec![out.0]);
    out
}

/// O passo da grelha, MENOR que o ladrilho — é o que faz as cópias se sobreporem.
pub(super) const GAP: f32 = 0.5;

const _: () = assert!(
    GAP < super::art::LADO,
    "sem sobreposicao a mistura nao se ve sobre o fundo escuro -- a cena deixa de ensinar"
);

#[cfg(test)]
#[path = "motion_object_smoke_blend_tests.rs"]
mod tests;
