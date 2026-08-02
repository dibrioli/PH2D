//! **A cena pronta para o smoke de A1/A2** (`PH2D_MOTION_OBJ_SMOKE`, doc 86 §7):
//! um OBJETO da engine, trazido para o grafo e **carimbado numa grade**.
//!
//! Até A o sistema de nós só sabia produzir ONDE (posições); nunca O QUE. Esta
//! cena monta as duas metades da resposta:
//!
//! 1. **o objeto** chamado **`Object`** na Hierarchy (o nome é a referência
//!    inteira, doc 86 §2) — um **sprite** (`=1`, A1) ou uma **forma vetorial**
//!    (`=2`, A2: a estrela é rasterizada numa tile pela membrana);
//! 2. **o grafo**: `source.object → motion.duplicator ← motion.grid → output` —
//!    o MESMO grafo nos dois modos (o `source.object` é media-agnóstico).
//!
//! O que se vê: a arte do objeto carimbada em cada ponto da grade — a MESMA
//! figura repetida, não um quad chapado. É a "conexão (renderização) com os
//! objetos da game engine" que o Enio pediu, ponta a ponta.
//!
//! **E o campo `Object` é um PICKER** (não uma caixa de texto): o nó nasce
//! SELECIONADO. Renomeie o objeto na Hierarchy e o nó para de achá-lo — o nome
//! É a referência. No modo vetor, **edite a forma** e as cópias re-assam (LIVE).

use ph2d_core::Vec2;
use ph2d_ecs::{Name, Transform};
use ph2d_nodegraph::graph::{Edge, Graph, NodeId, Pos};
use ph2d_render::Sprite;
use ph2d_vec_scene::{Paint, Rgba8, VecPath};

/// O nome que o artista daria ao objeto — e que ele escolhe no campo `Object`.
const OBJECT: &str = "Object";

/// Um tile de demo com cor (o átlas do boot empacota `0..16` como tiles HSV), pra
/// a arte carimbada ser inconfundivelmente a do sprite (modo `=1`).
const DEMO_TILE_KEY: u32 = 5;

/// O grafo comum aos dois modos: `source.object(name) → duplicator ← grid →
/// output`. A fonte traz a APARÊNCIA (na origem, um template), a grade dá os
/// PONTOS, e o duplicator cruza os dois. Devolve o sink.
fn build_stamp_graph(graph: &mut Graph, name: &str) -> NodeId {
    let src = graph.add_node("source.object");
    let grid = graph.add_node("motion.grid");
    let dup = graph.add_node("motion.duplicator");
    let out = graph.add_node("motion.output");
    // Layout acima do grafo da neve (que ocupa a faixa 0..).
    graph.set_pos(src, Pos { x: 0.0, y: -260.0 });
    graph.set_pos(grid, Pos { x: 0.0, y: -140.0 });
    graph.set_pos(dup, Pos { x: 210.0, y: -200.0 });
    graph.set_pos(out, Pos { x: 420.0, y: -200.0 });
    // shape → duplicator.shape (input 0); grid → duplicator.points (input 1).
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
    wire(graph, dup, 0, out, 0);

    graph.set_text_param(src, "object", name);
    // Uma grade folgada 4×4 = 16 carimbos.
    graph.set_param(grid, "rows", 4.0);
    graph.set_param(grid, "cols", 4.0);
    graph.set_param(grid, "gap_x", 1.3);
    graph.set_param(grid, "gap_y", 1.3);
    graph.set_label(src, "The Object");
    graph.set_label(dup, "Stamp On Grid");

    // Nasce SELECIONADO: o artista cai no PICKER (um chip "Object").
    ph2d_panel_motion_graph::request_graph_selection(vec![src.0]);
    out
}

/// Modo `=1`: um sprite direto (entidade com `Name`, não precisa do `sync`).
fn spawn_sprite(sim: &mut ph2d_ecs::SimWorld) {
    sim.world_mut().spawn((
        Transform::from_translation(Vec2::new(0.0, 0.0)),
        Sprite::atlas(DEMO_TILE_KEY, [0.8, 0.8], [1.0, 1.0, 1.0, 1.0]),
        Name::new(OBJECT),
    ));
}

/// Modo `=2`: uma estrela FILLED (a arte, inconfundível quando assada numa tile).
fn star_shape() -> VecPath {
    let mut p = ph2d_vec_scene::star([0.0, 0.0], 0.5, 0.5, 5, 0.45);
    p.fill = Some(Paint::solid(Rgba8::new(255, 170, 40, 255)));
    p.stroke = Some(ph2d_vec_scene::StrokeSpec::new(Rgba8::new(60, 40, 10, 255), 0.02));
    p
}

/// Modo `=2` frame 6: a entidade da forma já existe (o `sync` do frame a criou),
/// então ela ganha o **nome** que o `source.object` procura. A única forma da
/// cena é a nossa.
fn name_vector_entity(sim: &mut ph2d_ecs::SimWorld, map: &crate::vec_entities::VecEntityMap) -> bool {
    let Some((_, &bits)) = map.iter().next() else {
        return false;
    };
    let e = ph2d_ecs::Entity::from_bits(bits);
    match sim.world_mut().get_entity_mut(e) {
        Ok(mut ent) => {
            ent.insert(Name(OBJECT.to_string()));
            true
        }
        Err(_) => false,
    }
}

/// O frame corrente do roteiro (o hook não pode acrescentar campo em `App`).
static FRAME: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

/// O modo, lido UMA vez: `0` = off · `1` = sprite (A1) · `2` = vetor (A2).
fn mode() -> u32 {
    static M: std::sync::OnceLock<u32> = std::sync::OnceLock::new();
    *M.get_or_init(|| {
        std::env::var("PH2D_MOTION_OBJ_SMOKE")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(0)
    })
}

impl crate::App {
    /// Roda no prólogo do frame, ao lado dos outros smokes. No-op sem a env.
    pub(crate) fn motion_object_smoke(&mut self) {
        use std::sync::atomic::Ordering;
        let mode = mode();
        if mode == 0 || self.gfx.is_none() {
            return;
        }
        let f = FRAME.fetch_add(1, Ordering::Relaxed);
        match mode {
            // A1 — sprite: entidade direta, tudo num frame.
            1 if f == 3 => {
                let gfx = self.gfx.as_mut().expect("gfx");
                spawn_sprite(&mut gfx.sim);
                let out = build_stamp_graph(&mut gfx.motion.doc.graph, OBJECT);
                gfx.motion.sinks.push(out);
                let _ = gfx.tools.set_active(&ph2d_editor::ToolId::new("motion"));
                eprintln!(
                    "[motion.obj smoke =1] O SPRITE 'Object' (tile colorido) esta carimbado numa \
                     grade 4x4 = 16 copias. A arte de CADA copia e a do sprite. Renomeie o sprite \
                     na Hierarchy e as copias somem (o nome E a referencia)."
                );
            }
            // A2 — vetor: a forma entra primeiro (frame 3); a ENTIDADE dela só
            // existe depois do `vec_entities::sync`, e e nela que o nome mora.
            2 if f == 3 => {
                let gfx = self.gfx.as_mut().expect("gfx");
                gfx.vec_scene.push_path(star_shape());
            }
            2 if f == 6 => {
                let map = self.vec_entities.clone();
                let gfx = self.gfx.as_mut().expect("gfx");
                if name_vector_entity(&mut gfx.sim, &map) {
                    let out = build_stamp_graph(&mut gfx.motion.doc.graph, OBJECT);
                    gfx.motion.sinks.push(out);
                }
                let _ = gfx.tools.set_active(&ph2d_editor::ToolId::new("motion"));
                eprintln!(
                    "[motion.obj smoke =2] A ESTRELA vetorial 'Object' foi ASSADA numa tile pela \
                     membrana e esta carimbada numa grade 4x4 = 16 copias. A arte de CADA copia e a \
                     estrela (assada uma vez, cacheada por conteudo). Edite a forma com a tool \
                     Vector e as copias RE-ASSAM (LIVE); renomeie e as copias somem."
                );
            }
            _ => {}
        }
    }
}
