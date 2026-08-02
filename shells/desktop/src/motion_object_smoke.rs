//! **A cena pronta para o smoke de A1** (`PH2D_MOTION_OBJ_SMOKE=1`, doc 86 §7):
//! um SPRITE da engine, trazido para o grafo e **carimbado numa grade**.
//!
//! Até A1 o sistema de nós só sabia produzir ONDE (posições); nunca O QUE. Esta
//! cena monta as duas metades da resposta:
//!
//! 1. **o objeto**: um sprite (um tile colorido do átlas de demo) chamado
//!    **`Object`** na Hierarchy — porque **o nome é a referência inteira** (doc
//!    86 §2: não há id pra copiar);
//! 2. **o grafo**: `source.object → motion.duplicator ← motion.grid → output` —
//!    um segundo sink, ao lado da neve.
//!
//! O que se vê: a arte do sprite carimbada em cada ponto da grade — a MESMA
//! figura repetida, não um quad chapado. É a "conexão (renderização) com os
//! objetos da game engine" que o Enio pediu, ponta a ponta: a membrana lê o
//! sprite vivo (`source.object` o encontra pelo nome), o `motion.duplicator`
//! cruza-o com a grade, e o sink desenha o objeto de verdade.
//!
//! **E o campo `Object` é um PICKER** (não uma caixa de texto): o nó nasce
//! SELECIONADO, e o painel de params mostra o sprite como um chip **`Object`**.
//! Renomeie o sprite na Hierarchy e o nó para de achá-lo — o nome É a referência.

use ph2d_core::Vec2;
use ph2d_ecs::{Name, Transform};
use ph2d_nodegraph::graph::{Edge, Graph, NodeId, Pos};
use ph2d_render::Sprite;

/// O nome que o artista daria ao sprite — e que ele digita/escolhe no campo
/// `Object` do nó.
const OBJECT: &str = "Object";

/// Um tile de demo com cor (o átlas do boot empacota `0..16` como tiles HSV), pra
/// a arte carimbada ser inconfundivelmente a do sprite, não um retângulo liso.
const DEMO_TILE_KEY: u32 = 5;

/// Spawna o sprite nomeado (o WHAT) e monta o grafo que o carimba (o WHERE).
///
/// O sprite é uma entidade direta com `Name` — ao contrário de uma forma
/// vetorial, ela não depende do `vec_entities::sync`, então tudo cabe num frame.
/// O spawn roda no prólogo, ANTES da fase do motion (`publish_objects`), então a
/// membrana enxerga o sprite no MESMO frame e o `source.object` já resolve.
fn spawn_and_wire(sim: &mut ph2d_ecs::SimWorld, graph: &mut Graph) -> NodeId {
    sim.world_mut().spawn((
        Transform::from_translation(Vec2::new(0.0, 0.0)),
        Sprite::atlas(DEMO_TILE_KEY, [0.8, 0.8], [1.0, 1.0, 1.0, 1.0]),
        Name::new(OBJECT),
    ));

    // `source.object → duplicator ← grid → output`: a fonte traz a APARÊNCIA (na
    // origem, um template), a grade dá os PONTOS, e o duplicator cruza os dois.
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

    graph.set_text_param(src, "object", OBJECT);
    // Uma grade folgada 4×4 = 16 carimbos, espaçados o bastante pra o tile caber.
    graph.set_param(grid, "rows", 4.0);
    graph.set_param(grid, "cols", 4.0);
    graph.set_param(grid, "gap_x", 1.3);
    graph.set_param(grid, "gap_y", 1.3);
    graph.set_label(src, "The Object");
    graph.set_label(dup, "Stamp On Grid");

    // Nasce SELECIONADO: o artista cai no PICKER (um chip "Object"), não numa
    // caixa de texto vazia.
    ph2d_panel_motion_graph::request_graph_selection(vec![src.0]);
    eprintln!(
        "[motion.obj smoke] O sprite 'Object' (tile colorido) esta carimbado numa grade 4x4 = 16 \
         copias pelo grafo 'source.object -> duplicator <- grid -> output'. O no 'The Object' esta \
         SELECIONADO: o painel de params mostra o campo Object como PICKER (um chip 'Object'). \
         Renomeie o sprite na Hierarchy e as copias somem (o nome E a referencia). A arte de CADA \
         copia e a do sprite -- e uma so figura, nao um quad chapado."
    );
    out
}

/// O frame corrente do roteiro (o hook não pode acrescentar campo em `App`).
static FRAME: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

/// Ligado? Lido UMA vez (o prólogo do frame não paga um `getenv` por quadro).
fn on() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| std::env::var("PH2D_MOTION_OBJ_SMOKE").is_ok_and(|v| v != "0"))
}

impl crate::App {
    /// Roda no prólogo do frame, ao lado dos outros smokes. No-op sem a env.
    pub(crate) fn motion_object_smoke(&mut self) {
        use std::sync::atomic::Ordering;
        if !on() || self.gfx.is_none() {
            return;
        }
        // Frame 3: deixa o boot assentar, então monta tudo de uma vez (o sprite é
        // entidade direta, não precisa do `sync` do vetor de um frame anterior).
        if FRAME.fetch_add(1, Ordering::Relaxed) == 3 {
            let gfx = self.gfx.as_mut().expect("gfx");
            let out = spawn_and_wire(&mut gfx.sim, &mut gfx.motion.doc.graph);
            gfx.motion.sinks.push(out);
            let _ = gfx.tools.set_active(&ph2d_editor::ToolId::new("motion"));
        }
    }
}
