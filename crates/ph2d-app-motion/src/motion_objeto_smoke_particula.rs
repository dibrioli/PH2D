//! `PH2D_CONSTRAINT_SMOKE=12` — o grafo da **Fonte** (a cena «Partículas e mola» das constraints,
//! `ph2d_app_constraints::smoke_particula`): a cena das constraints cria o objecto Motion pelo nome
//! [`FONTE`]; aqui o grafo dele nasce quando a entidade aparece. Formas numa simulação com campos:
//!
//! ```text
//! emitter → integrate ⇄ (vortex → wind)      source.shape (círculo)
//!               └──────────────→ duplicator ←──┘ → output
//! ```
//!
//! O `id` de cada partícula atravessa o duplicador (a forma não tem `id`: a coluna do ponto chega)
//! e é a que o Copy Particle lê.

use ph2d_ecs::{MotionGenerator, Name, StableId};
use ph2d_nodegraph::graph::{Edge, Graph, NodeId, Pos};

/// O nome do objecto Motion que a cena 12 das constraints cria.
pub const FONTE: &str = "Fonte";
/// Quanto vive cada partícula (s) — menos que o laço de 4 s da cena: a Bola vê a sua morrer.
pub const VIDA_S: f32 = 3.0;
/// Partículas por segundo.
pub const TAXA: f32 = 4.0;

static FEITO: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

fn liga(g: &mut Graph, a: NodeId, b: NodeId, porta: u16, delayed: bool) {
    let _ = g.connect(Edge {
        from: (a, 0),
        to: (b, porta),
        delayed,
    });
}

/// Monta o grafo da Fonte em `g`; devolve os nós (o último é o `motion.output`).
pub fn grafo(g: &mut Graph) -> Vec<NodeId> {
    let em = g.add_node("motion.emitter");
    for (p, v) in [
        ("rate", TAXA),
        ("life", VIDA_S),
        ("speed", 2.8),
        ("angle", 90.0),
        ("spread", 30.0),
        ("seed", 7.0),
    ] {
        g.set_param(em, p, v);
    }
    let ig = g.add_node("motion.integrate");
    let redemoinho = g.add_node("force.vortex");
    for (p, v) in [
        ("center_x", 0.0),
        ("center_y", 2.5),
        ("strength", 2.5),
        ("radius", 3.0),
    ] {
        g.set_param(redemoinho, p, v);
    }
    let gravidade = g.add_node("force.wind");
    for (p, v) in [("angle", 270.0), ("strength", 1.6), ("gust", 0.0)] {
        g.set_param(gravidade, p, v);
    }
    let forma = g.add_node("source.shape");
    let circulo = ph2d_node_motion_shape::ShapeKind::Circle.index();
    #[expect(clippy::cast_precision_loss, reason = "um índice pequeno")]
    g.set_param(forma, ph2d_node_motion_shape::param::KIND, circulo as f32);
    g.set_param(forma, ph2d_node_motion_shape::param::SIZE, 0.13);
    for (p, v) in [
        (ph2d_node_motion_shape::param::FILL, 1.0),
        (ph2d_node_motion_shape::param::FILL_R, 0.35),
        (ph2d_node_motion_shape::param::FILL_G, 0.75),
        (ph2d_node_motion_shape::param::FILL_B, 1.0),
        (ph2d_node_motion_shape::param::FILL_A, 1.0),
    ] {
        g.set_param(forma, p, v);
    }
    let dup = g.add_node("motion.duplicator");
    let out = g.add_node("motion.output");
    liga(g, em, ig, 0, false);
    liga(g, ig, redemoinho, 0, true);
    liga(g, redemoinho, gravidade, 0, false);
    liga(g, gravidade, ig, 1, false);
    liga(g, forma, dup, 0, false);
    liga(g, ig, dup, 1, false);
    liga(g, dup, out, 0, false);
    let nos = vec![em, ig, redemoinho, gravidade, forma, dup, out];
    for (i, n) in nos.iter().enumerate() {
        #[expect(clippy::cast_precision_loss, reason = "poucos nós")]
        g.set_pos(
            *n,
            Pos {
                x: i as f32 * 200.0,
                y: if (2..4).contains(&i) { 160.0 } else { 0.0 },
            },
        );
    }
    nos
}

/// Uma vez, quando a Fonte da cena 12 nasce: monta o grafo dela.
pub fn cena_12(cx: &mut crate::motion_scene_ctx::MotionSceneCtx<'_>) {
    static ARMADA: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    let armada = *ARMADA
        .get_or_init(|| std::env::var("PH2D_CONSTRAINT_SMOKE").is_ok_and(|v| v.trim() == "12"));
    if !armada || FEITO.load(std::sync::atomic::Ordering::Relaxed) {
        return;
    }
    let w = cx.sim.world_mut();
    let mut q = w.query_filtered::<(&Name, &StableId), ph2d_ecs::With<MotionGenerator>>();
    let Some(dono) = q.iter(w).find(|(n, _)| n.0 == FONTE).map(|(_, s)| s.0) else {
        return;
    };
    FEITO.store(true, std::sync::atomic::Ordering::Relaxed);
    for n in grafo(&mut cx.motion.doc.graph) {
        cx.motion.doc.set_owner(n, dono);
    }
    cx.motion.pump.mark_dirty();
}

#[cfg(test)]
#[path = "motion_objeto_smoke_particula_tests.rs"]
mod tests;
