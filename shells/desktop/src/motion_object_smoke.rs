//! **A cena pronta para o smoke de A1/A2** (`PH2D_MOTION_OBJ_SMOKE`, doc 86 §7):
//! um OBJETO da engine, trazido para o grafo e **carimbado numa grade**.
//!
//! Até A o sistema de nós só sabia produzir ONDE (posições); nunca O QUE. Esta
//! cena monta as duas metades da resposta:
//!
//! 1. **o objeto** chamado **`Object`** na Hierarchy (o nome é a referência
//!    inteira, doc 86 §2) — um **sprite** (`=1`, A1), uma **forma vetorial**
//!    (`=2`, A2: a estrela é rasterizada numa tile pela membrana), um **objeto
//!    Flip** (`=3`, A3: as camadas do Flip são compostas no frame atual e assadas
//!    numa tile pela membrana) ou um **grupo de mídia mista** (`=4`, A4: o grupo
//!    emite os filhos como N instâncias vivas no seu layout relativo);
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

// A cena das DUAS FASES (`=7`, o `time_offset` do doc 89 folha 14) mora num irmão:
// ela é um ASSUNTO — *o mesmo objeto em dois tempos* — e traz a própria fixture
// animada, que nenhum dos outros modos precisa. Cortado pelo teto de LOC da shell.
#[path = "motion_object_smoke_times.rs"]
mod times;
use times::{build_two_times_graph, spawn_flip_walk_named};

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
    graph.set_pos(
        dup,
        Pos {
            x: 210.0,
            y: -200.0,
        },
    );
    graph.set_pos(
        out,
        Pos {
            x: 420.0,
            y: -200.0,
        },
    );
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

/// Como [`build_stamp_graph`], mas com um **`motion.oscillator`** (um deformer
/// GPU por-elemento) entre o duplicator e o output: `source.object → duplicator
/// ← grid → oscillator → output`. É esse sufixo GPU que torna o grafo **Hybrid**
/// — o `duplicator` (CPU) vira o boundary, o oscillator roda no device, e o
/// lowering carrega o `texture_id` do objeto até a word 41 (esta wave). Sem o
/// oscillator o sufixo é só o `output` passthrough (0 dispatch → rota CPU).
fn build_stamp_graph_osc(graph: &mut Graph, name: &str) -> NodeId {
    let out = build_stamp_graph(graph, name);
    // `out`'s input 0 is the duplicator; splice an oscillator in between.
    let dup = graph
        .edges()
        .iter()
        .find(|e| e.to == (out, 0))
        .map(|e| e.from.0)
        .expect("duplicator -> output edge");
    let osc = graph.add_node("motion.oscillator");
    graph.set_pos(
        osc,
        Pos {
            x: 315.0,
            y: -200.0,
        },
    );
    graph.set_param(osc, "channel", 1.0); // Y
    graph.set_param(osc, "amplitude", 0.5);
    graph.set_param(osc, "frequency", 0.4);
    graph.set_label(osc, "Wave");
    // Re-route dup → out into dup → osc → out.
    graph.disconnect(out, 0);
    let wire = |g: &mut Graph, a: NodeId, ap: u16, b: NodeId, bp: u16| {
        g.connect(Edge {
            from: (a, ap),
            to: (b, bp),
            delayed: false,
        })
        .expect("connect");
    };
    wire(graph, dup, 0, osc, 0);
    wire(graph, osc, 0, out, 0);
    out
}

/// O grafo do FREEZE (report do Enio, 2026-08-05): `source.object → dup1 ← grid1`,
/// `dup1 → dup2 ← grid2 → output` — DOIS duplicators em série, cada grade 20×20, para
/// **20×20 × 20×20 = 160.000** estrelas. É a cena que congelava como vetor vivo (160k
/// Vello fills/frame) e que agora deve renderizar como TILES instanciados (o LOD).
fn build_stamp_graph_2dup(graph: &mut Graph, name: &str) -> NodeId {
    let src = graph.add_node("source.object");
    let grid1 = graph.add_node("motion.grid");
    let dup1 = graph.add_node("motion.duplicator");
    let grid2 = graph.add_node("motion.grid");
    let dup2 = graph.add_node("motion.duplicator");
    let out = graph.add_node("motion.output");
    graph.set_pos(src, Pos { x: 0.0, y: -320.0 });
    graph.set_pos(grid1, Pos { x: 0.0, y: -200.0 });
    graph.set_pos(
        dup1,
        Pos {
            x: 200.0,
            y: -260.0,
        },
    );
    graph.set_pos(
        grid2,
        Pos {
            x: 200.0,
            y: -140.0,
        },
    );
    graph.set_pos(
        dup2,
        Pos {
            x: 400.0,
            y: -200.0,
        },
    );
    graph.set_pos(
        out,
        Pos {
            x: 600.0,
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
    // shape → dup1.shape; grid1 → dup1.points; dup1 → dup2.shape; grid2 → dup2.points.
    wire(graph, src, 0, dup1, 0);
    wire(graph, grid1, 0, dup1, 1);
    wire(graph, dup1, 0, dup2, 0);
    wire(graph, grid2, 0, dup2, 1);
    wire(graph, dup2, 0, out, 0);
    graph.set_text_param(src, "object", name);
    // grid1 = 20×20 = 400 (a inner block); grid2 = 20×20 = 400 (the outer tiling,
    // wider gaps so the 400 blocks spread out).
    for (g, gap) in [(grid1, 0.6f32), (grid2, 14.0f32)] {
        graph.set_param(g, "rows", 20.0);
        graph.set_param(g, "cols", 20.0);
        graph.set_param(g, "gap_x", gap);
        graph.set_param(g, "gap_y", gap);
    }
    graph.set_label(src, "The Object");
    graph.set_label(dup1, "Inner 20x20");
    graph.set_label(dup2, "Outer 20x20 = 160k");
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
    p.stroke = Some(ph2d_vec_scene::StrokeSpec::new(
        Rgba8::new(60, 40, 10, 255),
        0.02,
    ));
    p
}

/// Modo `=2` frame 6: a entidade da forma já existe (o `sync` do frame a criou),
/// então ela ganha o **nome** que o `source.object` procura. A única forma da
/// cena é a nossa.
fn name_vector_entity(
    sim: &mut ph2d_ecs::SimWorld,
    map: &crate::vec_entities::VecEntityMap,
) -> bool {
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

/// Modo `=3`: um objeto FLIP de 2 camadas (BG azul + FG laranja), empurrado no
/// `FlipDoc`. A ENTIDADE dele (com `Name` "Object") é criada pelo
/// `flip_entities::sync` — que copia o nome do objeto —, então o grafo o acha pelo
/// nome sem o smoke precisar nomear nada (≠ do vetor). A membrana compõe as DUAS
/// camadas no frame atual numa tile.
fn spawn_flip_object(flip: &mut ph2d_flip::FlipDoc) {
    spawn_flip_object_named(flip, OBJECT);
}

/// Como `spawn_flip_object`, mas com um nome dado (o filho Flip de um grupo precisa
/// de um nome distinto do grupo, doc 86 §2 A4).
fn spawn_flip_object_named(flip: &mut ph2d_flip::FlipDoc, name: &str) {
    use ph2d_flip::{Hold, KeyKind, Rgba};
    let oid = flip.push_object(name);
    let obj = flip.object_mut(oid).expect("objeto Flip recém-criado");
    obj.fps = 12.0;
    // BG: um retângulo azul preenchido (o campo).
    let bg = obj.add_layer("BG");
    if let Some(d) = obj.insert_frame(bg, 0, Hold::Implicit, KeyKind::Keyframe) {
        obj.drawing_mut(d)
            .expect("desenho BG")
            .strokes
            .push(flip_rect(
                Vec2::new(-0.9, -0.6),
                Vec2::new(0.9, 0.6),
                Rgba::new(0.2, 0.5, 0.95, 1.0),
            ));
    }
    // FG: um quadrado laranja menor por cima — a arte que torna a tile assada
    // INCONFUNDÍVEL (duas camadas compostas, não um quad chapado).
    let fg = obj.add_layer("FG");
    if let Some(d) = obj.insert_frame(fg, 0, Hold::Implicit, KeyKind::Keyframe) {
        obj.drawing_mut(d)
            .expect("desenho FG")
            .strokes
            .push(flip_rect(
                Vec2::new(-0.35, -0.35),
                Vec2::new(0.35, 0.35),
                Rgba::new(0.98, 0.7, 0.15, 1.0),
            ));
    }
}

/// Um retângulo Flip FECHADO e PREENCHIDO (espelha `flip_demo::filled_rect`).
fn flip_rect(min: Vec2, max: Vec2, color: ph2d_flip::Rgba) -> ph2d_flip::FlipStroke {
    use ph2d_flip::{Fill, FlipStroke, Point};
    let mut s = FlipStroke::new();
    for corner in [min, Vec2::new(max.x, min.y), max, Vec2::new(min.x, max.y)] {
        s.push_point(Point {
            pos: corner,
            width: 0.04,
            opacity: 1.0,
            color,
        });
    }
    s.closed = true;
    s.hardness = 1.0;
    s.fill = Some(Fill {
        color,
        opacity: 1.0,
    });
    s
}

/// Um `Transform` LOCAL (relativo ao grupo) numa posição, resto identidade (A4).
fn child_at(x: f32, y: f32) -> Transform {
    Transform {
        translation: Vec2::new(x, y),
        ..Transform::IDENTITY
    }
}

/// Acha a entidade-GRUPO pelo nome (`Name` + `GroupedChildren`), doc 86 §2 A4.
fn find_group(sim: &mut ph2d_ecs::SimWorld, name: &str) -> Option<ph2d_ecs::Entity> {
    let mut q = sim
        .world_mut()
        .query_filtered::<(ph2d_ecs::Entity, &Name), ph2d_ecs::With<ph2d_ecs::GroupedChildren>>();
    let world = sim.world();
    q.iter(world).find(|(_, n)| n.0 == name).map(|(e, _)| e)
}

/// O frame corrente do roteiro (o hook não pode acrescentar campo em `App`).
static FRAME: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

/// O modo: `0` off · `1` sprite (A1) · `2` vetor (A2) · `3` Flip (A3) · `4` grupo
/// (A4) · `5` A WAVE (objeto vetor + oscillator GPU, cozido no device).
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
            // A3 — Flip: o objeto entra no FlipDoc (frame 3); a ENTIDADE dele (Name
            // "Object") e criada pelo `flip_entities::sync`, entao o grafo o acha pelo
            // nome no frame 6 (sem nomear a mao — o sync copia o nome do objeto).
            3 if f == 3 => {
                let gfx = self.gfx.as_mut().expect("gfx");
                spawn_flip_object(&mut gfx.flip);
            }
            3 if f == 6 => {
                let gfx = self.gfx.as_mut().expect("gfx");
                let out = build_stamp_graph(&mut gfx.motion.doc.graph, OBJECT);
                gfx.motion.sinks.push(out);
                let _ = gfx.tools.set_active(&ph2d_editor::ToolId::new("motion"));
                eprintln!(
                    "[motion.obj smoke =3] O OBJETO Flip 'Object' (BG azul + FG laranja, 2 camadas) \
                     foi COMPOSTO no frame atual e ASSADO numa tile pela membrana, carimbado numa \
                     grade 4x4 = 16 copias. A arte de CADA copia e o objeto composto (as duas \
                     camadas, nao um quad chapado). Cacheada por conteudo (LIVE): editar/animar o \
                     Flip re-assa; renomear na Hierarchy e as copias somem."
                );
            }
            // A4 — grupo: um GRUPO 'Object' com filhos de MIDIA MISTA (sprite + vetor
            // + flip) nos seus lugares relativos. O grupo emite os filhos como N
            // instancias VIVAS; carimbar o grupo replica o layout inteiro.
            4 if f == 3 => {
                let gfx = self.gfx.as_mut().expect("gfx");
                // O grupo (Name "Object" + GroupedChildren) + um sprite filho a esquerda.
                let group = gfx
                    .sim
                    .world_mut()
                    .spawn((
                        Name::new(OBJECT),
                        ph2d_ecs::GroupedChildren,
                        Transform::IDENTITY,
                    ))
                    .id();
                gfx.sim.world_mut().spawn((
                    Sprite::atlas(DEMO_TILE_KEY, [0.7, 0.7], [1.0, 1.0, 1.0, 1.0]),
                    child_at(-1.1, 0.0),
                    ph2d_ecs::ChildOf(group),
                ));
                // O vetor + o flip entram agora; suas ENTIDADES nascem no sync dos
                // frames seguintes, quando serao nomeadas e parenteadas ao grupo.
                gfx.vec_scene.push_path(star_shape());
                spawn_flip_object_named(&mut gfx.flip, "GFlip");
            }
            4 if f == 6 => {
                let vec_map = self.vec_entities.clone();
                let flip_map = self.flip_entities.clone();
                let gfx = self.gfx.as_mut().expect("gfx");
                if let Some(group) = find_group(&mut gfx.sim, OBJECT) {
                    // O vetor (a unica forma da cena) vira filho SEM NOME no centro — o
                    // caso do item 3 (doc 86 §9.6): um filho vetor/flip de grupo sem Name
                    // continua carimbado, resolvido pelo seu DRAWING id (`VecPathRef`), nao
                    // pelo nome. O bake o tila porque ele esta num grupo NOMEADO.
                    if let Some((_, &bits)) = vec_map.iter().next() {
                        let e = ph2d_ecs::Entity::from_bits(bits);
                        if let Ok(mut ent) = gfx.sim.world_mut().get_entity_mut(e) {
                            ent.insert((child_at(0.0, 0.0), ph2d_ecs::ChildOf(group)));
                        }
                    }
                    // O flip (o unico objeto Flip) vira filho a direita. O sync ja
                    // carimbou a entidade com Name "GFlip" (do push_object), entao so
                    // parenteamos + posicionamos.
                    if let Some((_, &bits)) = flip_map.iter().next() {
                        let e = ph2d_ecs::Entity::from_bits(bits);
                        if let Ok(mut ent) = gfx.sim.world_mut().get_entity_mut(e) {
                            ent.insert((child_at(1.1, 0.0), ph2d_ecs::ChildOf(group)));
                        }
                    }
                }
            }
            4 if f == 9 => {
                let gfx = self.gfx.as_mut().expect("gfx");
                let out = build_stamp_graph(&mut gfx.motion.doc.graph, OBJECT);
                gfx.motion.sinks.push(out);
                let _ = gfx.tools.set_active(&ph2d_editor::ToolId::new("motion"));
                eprintln!(
                    "[motion.obj smoke =4] O GRUPO 'Object' (sprite SEM NOME + estrela vetor SEM \
                     NOME + objeto Flip 'GFlip', MIDIA MISTA) esta carimbado numa grade 4x4 = 16 \
                     copias. Cada copia mostra o GRUPO INTEIRO (3 filhos lado a lado) e cada filho \
                     continua VIVO (o Flip anima independente). A estrela vetor do CENTRO NAO tem \
                     nome (item 3): ela e tilada porque esta num grupo nomeado e resolvida pelo seu \
                     drawing id, nao pelo nome. SE ela aparecer nas 16 copias, o item 3 passou; se \
                     o centro sair em branco, FALHOU. Renomeie o grupo e as copias somem."
                );
            }
            // =5 — VETOR VIVO (decisao do Enio): um `source.object` de VETOR nao
            // assa mais uma tile raster — ele emite `geometry_id` e o vector pass o
            // desenha CRISP em qualquer zoom (ADR-0154 reusado para objetos). Uma
            // estrela vetorial carimbada numa grade 4x4 + um `motion.oscillator` que
            // a ondula. ⚠️ Um grafo com vetor vivo RECUSA para a CPU (a GPU nao tem
            // rota `geometry_id`), entao `gpu_live` deve ser FALSE — o stamp de GPU
            // sobrevive para objetos de SPRITE PURO (=1). A entra a estrela (frame
            // 3), a entidade e nomeada + o grafo montado (frame 6), e o diagnostico
            // e lido depois que o pump cozinhou algumas vezes (frame 40).
            5 if f == 3 => {
                let gfx = self.gfx.as_mut().expect("gfx");
                gfx.vec_scene.push_path(star_shape());
            }
            5 if f == 6 => {
                let map = self.vec_entities.clone();
                let gfx = self.gfx.as_mut().expect("gfx");
                if name_vector_entity(&mut gfx.sim, &map) {
                    let out = build_stamp_graph_osc(&mut gfx.motion.doc.graph, OBJECT);
                    gfx.motion.sinks.push(out);
                }
                let _ = gfx.tools.set_active(&ph2d_editor::ToolId::new("motion"));
            }
            5 if f == 40 => {
                let gfx = self.gfx.as_ref().expect("gfx");
                let m = &gfx.motion;
                // Um grafo com vetor vivo RECUSA para a CPU (a GPU nao tem rota
                // `geometry_id`) — `gpu_live` deve ser FALSE. O pump lowerou as N
                // copias como VectorInstances (crisp), cada uma com o handle da
                // estrela no `geometry_id`.
                let live = m.gpu_live;
                let vecs = m.pump.vector_instances.len();
                let gid = m
                    .pump
                    .vector_instances
                    .first()
                    .map(|v| v.geometry_id)
                    .unwrap_or(0);
                eprintln!(
                    "[motion.obj smoke =5] A ESTRELA vetorial 'Object' agora e VETOR VIVO \
                     (`geometry_id`, NAO uma tile raster) carimbada numa grade 4x4 e ONDULADA por \
                     um oscillator. gpu_live={live} vector_instances={vecs} geometry_id={gid}. \
                     O QUE OLHAR: as 16 estrelas ondulando no Y, NITIDAS em QUALQUER zoom (o ponto \
                     da wave — de perto E de longe, sem a suavizada da tile raster de antes), com \
                     o preenchimento laranja E o contorno marrom (a arte AUTORADA, nao branca). Se \
                     vector_instances=16 E geometry_id>0 E as estrelas aparecem crisp e coloridas, \
                     passou. gpu_live DEVE ser false: um grafo com vetor vivo recusa para a CPU de \
                     proposito — o stamp de GPU sobrevive para objetos de SPRITE PURO (=1). Se as \
                     copias sairem BRANCAS/sem cor, o draw_shape_instance ignorou o fill — PARE."
                );
            }
            // =6 — O FREEZE DAS 160k (report do Enio): dois duplicators em série →
            // 400×400 = 160.000 estrelas. Como vetor vivo isso congela (32,6 ms/frame
            // só de CPU + GPU); o LOD move as instâncias acima do joelho (20k) para
            // TILES instanciados (o caminho pré-Parte-1, que escalava a milhões),
            // deixando o resto crisp. A estrela entra (frame 3), é nomeada + o grafo de
            // 2 duplicators montado (frame 6), e o diagnóstico é lido depois que o pump
            // cozinhou e o LOD particionou (frame 40).
            6 if f == 3 => {
                let gfx = self.gfx.as_mut().expect("gfx");
                gfx.vec_scene.push_path(star_shape());
            }
            6 if f == 6 => {
                let map = self.vec_entities.clone();
                let gfx = self.gfx.as_mut().expect("gfx");
                if name_vector_entity(&mut gfx.sim, &map) {
                    let out = build_stamp_graph_2dup(&mut gfx.motion.doc.graph, OBJECT);
                    gfx.motion.sinks.push(out);
                }
                let _ = gfx.tools.set_active(&ph2d_editor::ToolId::new("motion"));
            }
            6 if f == 40 => {
                let gfx = self.gfx.as_ref().expect("gfx");
                let m = &gfx.motion;
                // Pós-LOD: as instâncias acima do joelho foram para `instances` (tiles
                // GPU-instanciados); só o que ficou abaixo do joelho segue em
                // `vector_instances` (crisp). Numa cena de 160k, TUDO vira tile ⇒
                // `instances ~= 160000` e `vector_instances ~= 0`, e o app NÃO congela.
                let tiles = m.pump.instances.len();
                let vecs = m.pump.vector_instances.len();
                eprintln!(
                    "[motion.obj smoke =6] 160.000 estrelas (2 duplicators em serie, 20x20 x 20x20). \
                     O LOD moveu as instancias acima do joelho ({} > LOD_COUNT) para TILES \
                     instanciados: instances={} vector_instances={}. \
                     O QUE OLHAR: a grade inteira aparece e o app RODA LISO (nao congela como \
                     antes) — as estrelas sao tiles instanciados na GPU (o caminho que escalava a \
                     milhoes). Reduza a grade (ou os duplicators) para poucas copias e elas voltam \
                     a ser VETOR VIVO crisp (=5). Se instances~=160000 e vector_instances~=0 e a \
                     tela nao trava, o LOD funcionou; se o app CONGELAR, o LOD nao particionou \
                     (verifique que o tile do objeto foi assado — texture_id).",
                    crate::render_loop::motion_bridge::objects_lod_count(),
                    tiles,
                    vecs
                );
            }
            7 if f == 3 => {
                let gfx = self.gfx.as_mut().expect("gfx");
                spawn_flip_walk_named(&mut gfx.flip, OBJECT);
            }
            7 if f == 6 => {
                let gfx = self.gfx.as_mut().expect("gfx");
                let outs = build_two_times_graph(&mut gfx.motion.doc.graph, OBJECT);
                gfx.motion.sinks.extend(outs);
                let _ = gfx.tools.set_active(&ph2d_editor::ToolId::new("motion"));
                // O relógio ANDA: um offset de tempo só é visível numa animação que
                // corre. Uma cena parada mostraria dois desenhos diferentes e não
                // diria se a diferença é de FASE.
                self.playhead.play();
                eprintln!(
                    "[motion.obj smoke =7] o MESMO objeto Flip ({OBJECT}, 12 fps, 4 desenhos \
                     em 0/3/6/9) trazido DUAS vezes: a grade da ESQUERDA em time_offset = 0 \
                     (o quadro atual) e a da DIREITA em +0,25 s, que a 12 fps sao TRES \
                     desenhos a frente. O QUE OLHAR: as duas grades desenham a MESMA arte \
                     (campo azul + quadrado laranja) e o quadrado da direita esta sempre UM \
                     PASSO a frente do da esquerda — se os dois andarem juntos, o offset nao \
                     chegou ao bake; se a direita SUMIR, o canal deslocado nao foi publicado. \
                     Pare o play e faca scrub: a diferenca de fase tem de se manter em \
                     qualquer quadro."
                );
            }
            _ => {}
        }
    }
}
