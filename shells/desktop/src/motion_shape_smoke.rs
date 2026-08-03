//! **A cena pronta para o smoke de `source.shape`** (`PH2D_SHAPE_SMOKE`, ADR-0154):
//! uma FORMA parametrica, gerada no grafo e **carimbada numa grade** — vetor VIVO
//! na GPU (nitida em qualquer zoom), nao uma tile assada.
//!
//! O grafo: `source.shape -> motion.duplicator <- motion.grid -> output`. O
//! `source.shape` NAO precisa de objeto nenhum na cena — a shell constroi o
//! `VecPath` a partir dos PARAMS do no (`kind`, `size`, `sides`, ...) e o publica
//! sob a chave de conteudo que o no le; o `motion.duplicator` carimba a MESMA
//! geometria em cada ponto da grade, e o `geometry_id` (o irmao do `texture_id`)
//! atravessa o duplicator ate cada copia. **16 estrelas nitidas.**
//!
//! No frame 90 o `kind` vira ENGRENAGEM: as 16 copias RE-CONSTROEM ao vivo — o
//! `kind` e um param, entao mudar um numero re-coze o no e so o que esta a jusante.

use ph2d_nodegraph::graph::{Edge, Graph, NodeId, Pos};

/// Star = 5, Gear = 7 (indices de `ShapeKind`; ver `ph2d_node_motion_shape`).
const STAR: f32 = 5.0;
const GEAR: f32 = 7.0;

/// O grafo comum: `source.shape -> duplicator <- grid -> output`. O `source.shape`
/// nasce com o `kind` dado. Devolve `(o no da forma, o sink)`.
fn build_shape_graph(graph: &mut Graph, kind: f32) -> (NodeId, NodeId) {
    let src = graph.add_node("source.shape");
    let grid = graph.add_node("motion.grid");
    let dup = graph.add_node("motion.duplicator");
    let out = graph.add_node("motion.output");
    // Layout acima do grafo da neve (que ocupa a faixa 0..).
    graph.set_pos(src, Pos { x: 0.0, y: -260.0 });
    graph.set_pos(grid, Pos { x: 0.0, y: -140.0 });
    graph.set_pos(dup, Pos { x: 210.0, y: -200.0 });
    graph.set_pos(out, Pos { x: 420.0, y: -200.0 });
    {
        let mut wire = |a: NodeId, ap: u16, b: NodeId, bp: u16| {
            graph
                .connect(Edge {
                    from: (a, ap),
                    to: (b, bp),
                    delayed: false,
                })
                .expect("connect");
        };
        wire(src, 0, dup, 0); // shape -> duplicator.shape (input 0)
        wire(grid, 0, dup, 1); // grid  -> duplicator.points (input 1)
        wire(dup, 0, out, 0); // duplicator -> output
    }
    graph.set_param(src, "kind", kind);
    graph.set_param(src, "size", 0.4);
    graph.set_param(src, "sides", 6.0);
    // Uma grade folgada 4x4 = 16 carimbos.
    graph.set_param(grid, "rows", 4.0);
    graph.set_param(grid, "cols", 4.0);
    graph.set_param(grid, "gap_x", 1.3);
    graph.set_param(grid, "gap_y", 1.3);
    graph.set_label(src, "The Shape");
    graph.set_label(dup, "Stamp On Grid");
    // Nasce SELECIONADO: o artista cai no card de params da forma.
    ph2d_panel_motion_graph::request_graph_selection(vec![src.0]);
    (src, out)
}

/// O modo: `0` off, `1` ligado (Star -> Gear ao vivo).
fn mode() -> u32 {
    static M: std::sync::OnceLock<u32> = std::sync::OnceLock::new();
    *M.get_or_init(|| {
        std::env::var("PH2D_SHAPE_SMOKE")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(0)
    })
}

/// O frame corrente do roteiro (o hook nao pode acrescentar campo em `App`).
static FRAME: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
/// O id do no `source.shape` criado no frame 3, para a troca de `kind` no frame 90.
/// `u32::MAX` = ainda nao criado.
static SHAPE_NODE: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(u32::MAX);

impl crate::App {
    /// Roda no prologo do frame, ao lado dos outros smokes. No-op sem a env.
    pub(crate) fn motion_shape_smoke(&mut self) {
        use std::sync::atomic::Ordering;
        if mode() == 0 || self.gfx.is_none() {
            return;
        }
        let f = FRAME.fetch_add(1, Ordering::Relaxed);
        match f {
            // Frame 3: monta o grafo, empurra o sink, abre a tool Motion.
            3 => {
                let gfx = self.gfx.as_mut().expect("gfx");
                let (src, out) = build_shape_graph(&mut gfx.motion.doc.graph, STAR);
                gfx.motion.sinks.push(out);
                SHAPE_NODE.store(src.0, Ordering::Relaxed);
                let _ = gfx.tools.set_active(&ph2d_editor::ToolId::new("motion"));
                eprintln!(
                    "[shape smoke =1] source.shape (STAR) carimbada numa grade 4x4 = 16 COPIAS \
                     nitidas (vetor VIVO na GPU, nao uma tile assada). O no nasce SELECIONADO: o \
                     painel de params mostra SO os controles que a estrela usa (Shape, Size, \
                     Sides/Points, Corner, Point Depth) — nao os da engrenagem. Edite-os e as 16 \
                     copias mudam SEM PISCAR. No frame 90 o kind vira ENGRENAGEM ao vivo."
                );
            }
            // Frame 90: troca o kind para Gear — as 16 copias re-constroem.
            90 => {
                let src = SHAPE_NODE.load(Ordering::Relaxed);
                if src != u32::MAX {
                    let gfx = self.gfx.as_mut().expect("gfx");
                    gfx.motion.doc.graph.set_param(NodeId(src), "kind", GEAR);
                    eprintln!(
                        "[shape smoke =1] kind -> ENGRENAGEM: as 16 copias RE-CONSTROEM ao vivo, \
                         SEM PISCAR. Agora o painel troca para os controles da engrenagem (Teeth, \
                         Tooth Depth, Hole) e ela tem um FURO central de verdade. Mudar um numero \
                         re-coze so o que esta a jusante; cada copia continua nitida em qualquer zoom."
                    );
                }
            }
            _ => {}
        }
    }
}
