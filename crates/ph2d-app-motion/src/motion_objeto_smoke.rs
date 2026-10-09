//! `PH2D_MOTION_OBJETO_SMOKE=1|2` — **o Motion como OBJECTO** (doc ObjectsAndComponents 01 §13, F5).
//!
//! Dois objectos Motion na Hierarquia, cada um com o grafo DELE:
//!
//! - **Pilha** — formas que caem e se empilham em duas taças (a cena `=114`: formas + simulação);
//! - **Enxame** — cópias da **Bola** puxadas por forças de campo (íman, ruído, arrasto) numa
//!   simulação; o objecto está deslocado para a direita e rodado, e o enxame vai com ele. A Bola é
//!   filha dele (o molde) e não se desenha sozinha.
//!
//! `=1`: modo Object, ferramenta Motion DESLIGADA — os dois desenham na mesma (sempre visível), o
//! Enxame seleccionado. `=2`: o Enxame em **Edit** — o editor de nós abre só com o grafo dele.

use ph2d_core::Vec2;
use ph2d_ecs::{ChildOf, MotionGenerator, Name, StableId, Transform};
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId, Pos};
use ph2d_render::Sprite;

static FEITO: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// O nome do molde.
const BOLA: &str = "Ball";

fn liga(g: &mut Graph, a: NodeId, ap: u16, b: NodeId, bp: u16, delayed: bool) {
    let _ = g.connect(Edge {
        from: (a, ap),
        to: (b, bp),
        delayed,
    });
}

/// O grafo do Enxame: `grade → integra ⇄ forças (íman · ruído · arrasto)`, e o duplicador veste cada
/// ponto com a Bola. Devolve os nós criados.
fn enxame(g: &mut Graph) -> Vec<NodeId> {
    let src = g.add_node("source.object");
    g.set_text_param(src, "object", BOLA);
    let grid = g.add_node("motion.grid");
    g.set_param(grid, "rows", 12.0);
    g.set_param(grid, "cols", 12.0);
    g.set_param(grid, "gap_x", 0.25);
    g.set_param(grid, "gap_y", 0.25);
    let ig = g.add_node("motion.integrate");
    let iman = g.add_node("force.attractor");
    g.set_param(iman, "strength", 3.5);
    g.set_param(iman, "radius", 20.0);
    let ruido = g.add_node("force.curl");
    g.set_param(ruido, "strength", 2.0);
    g.set_param(ruido, "scale", 0.8);
    g.set_param(ruido, "speed", 0.6);
    let arrasto = g.add_node("force.drag");
    g.set_param(arrasto, "coefficient", 1.0);
    let dup = g.add_node("motion.duplicator");
    let out = g.add_node("motion.output");
    liga(g, grid, 0, ig, 0, false);
    liga(g, ig, 0, iman, 0, true);
    liga(g, iman, 0, ruido, 0, false);
    liga(g, ruido, 0, arrasto, 0, false);
    liga(g, arrasto, 0, ig, 1, false);
    liga(g, src, 0, dup, 0, false);
    liga(g, ig, 0, dup, 1, false);
    liga(g, dup, 0, out, 0, false);
    let nos = vec![src, grid, ig, iman, ruido, arrasto, dup, out];
    for (i, n) in nos.iter().enumerate() {
        g.set_pos(
            *n,
            Pos {
                x: i as f32 * 200.0,
                y: if (3..6).contains(&i) { 160.0 } else { 0.0 },
            },
        );
    }
    g.set_label(out, "Swarm");
    nos
}

/// Ver o cabeçalho do módulo.
pub fn motion_objeto_smoke(cx: &mut crate::motion_scene_ctx::MotionSceneCtx<'_>) {
    let Some(nivel) = std::env::var("PH2D_MOTION_OBJETO_SMOKE").ok() else {
        return;
    };
    if FEITO.swap(true, std::sync::atomic::Ordering::Relaxed) {
        return;
    }
    let world = cx.sim.world_mut();
    let pilha = world
        .spawn((Name::new("Pile"), Transform::IDENTITY, MotionGenerator))
        .id();
    let mut pose = Transform::from_translation(Vec2::new(4.0, 1.5));
    pose.rotation = 20f32.to_radians();
    let enxame_e = world
        .spawn((Name::new("Swarm"), pose, MotionGenerator))
        .id();
    world.spawn((
        Name::new(BOLA),
        Transform::IDENTITY,
        Sprite::atlas(0, [0.12, 0.12], [1.0, 0.55, 0.15, 1.0]),
        ChildOf(enxame_e),
    ));
    ph2d_ecs::assign_missing_stable_ids(world);
    let sid = |e| world.get::<StableId>(e).map_or(0, |s| s.0);
    let (s_pilha, s_enxame) = (sid(pilha), sid(enxame_e));
    let antes: Vec<NodeId> = cx.motion.doc.graph.nodes().iter().map(|n| n.id).collect();
    crate::motion_state::demo_router::build_level(
        Some("114"),
        &mut cx.motion.doc,
        &cx.motion.registry,
    );
    let novos: Vec<NodeId> = cx
        .motion
        .doc
        .graph
        .nodes()
        .iter()
        .map(|n| n.id)
        .filter(|n| !antes.contains(n))
        .collect();
    for n in novos {
        cx.motion.doc.set_owner(n, s_pilha);
    }
    for n in enxame(&mut cx.motion.doc.graph) {
        cx.motion.doc.set_owner(n, s_enxame);
    }
    cx.motion.doc.adopt_unowned(s_pilha);
    cx.motion.objeto.editing = Some(s_enxame);
    cx.motion.pump.mark_dirty();
    cx.playhead.play();
    if let Some(hero) = cx.hero.as_mut() {
        hero.gizmo.replace_selection(Some(enxame_e.to_bits()));
        hero.set_panel_visible("hierarchy", true);
    }
    if nivel == "2" {
        let _ = cx
            .tools
            .set_active(&ph2d_editor_core::ToolId::new("motion"));
    }
    eprintln!(
        "[motion-objeto smoke ={nivel}] Two Motion OBJECTS in the Hierarchy: 'Pile' (shapes falling into \
         two bowls) and 'Swarm' (copies of 'Ball' pulled by force fields), moved right and rotated 20° — \
         the swarm goes with it. 'Ball' is Swarm's child (the template) and does not draw by itself. \
         =1: Object mode, Motion tool OFF, both still draw. =2: Swarm in Edit — the node editor shows \
         ONLY Swarm's nodes."
    );
}
