//! **A cena para o smoke da auto-correção de setup** (`PH2D_AUTOFIX_SMOKE`, ADR-0155):
//! uma `force.wind` colocada DIRETO na cadeia horizontal, **sem integrador**. O
//! artista faz o gesto errado (ligar a força ao output); no instante em que a aresta
//! conecta, o app **auto-conserta o setup** — reencaminha a força por um
//! `motion.integrate` (a grid passa a alimentar o integrador, a força vira o ramo de
//! `forces`, e o `reconcile` plumba o laço `pre`), e os pontos **passam a DERIVAR**
//! com o vento. Antes do conserto, nada se movia.
//!
//! No frame 90 o artista **apaga o integrador** (gesto destrutivo): os pontos
//! CONGELAM e o app **NÃO** re-insere — apagar para religar à mão nunca é combatido.
//!
//! **`=2` (REORDER):** um sim que JÁ FUNCIONA (`grid -> integrate -> output`) e o
//! artista solta uma `force.wind` SOBRE o fio de saída (um Splice) — a força cai
//! DEPOIS do integrador, onde o accel dela nunca é consumido. O app **REUSA** o
//! integrador que já está ali (não insere um segundo): a força vira o ramo de forças,
//! e os pontos passam a DERIVAR. Um Ctrl+Z reverte só o reorder.

use ph2d_nodegraph::graph::{Edge, Pos};
use ph2d_panel_motion_graph::GraphIntent;

/// O modo: `0` off, `1` inserir (força na cadeia horizontal), `2` reorder (força
/// spliceada depois de um integrador que já existe).
fn mode() -> u32 {
    static M: std::sync::OnceLock<u32> = std::sync::OnceLock::new();
    *M.get_or_init(|| {
        std::env::var("PH2D_AUTOFIX_SMOKE")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(0)
    })
}

/// O frame corrente do roteiro (o hook não pode acrescentar campo em `App`).
static FRAME: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

impl crate::App {
    /// Roda no prólogo do frame, ao lado dos outros smokes. No-op sem a env.
    pub(crate) fn motion_autofix_smoke(&mut self) {
        use std::sync::atomic::Ordering;
        if mode() == 0 || self.gfx.is_none() {
            return;
        }
        let f = FRAME.fetch_add(1, Ordering::Relaxed);
        match (mode(), f) {
            // =1, frame 3: monta grid -> force.wind + output SOLTO, abre a tool Motion,
            // e empurra o gesto ERRADO (Connect force -> output, sem integrador). O
            // bridge drena o intent, conecta e auto-conserta no mesmo frame.
            (1, 3) => {
                let gfx = self.gfx.as_mut().expect("gfx");
                let (grid, force, out) = {
                    let g = &mut gfx.motion.doc.graph;
                    let grid = g.add_node("motion.grid");
                    let force = g.add_node("force.wind");
                    let out = g.add_node("motion.output");
                    g.set_pos(
                        grid,
                        Pos {
                            x: -220.0,
                            y: -260.0,
                        },
                    );
                    g.set_pos(force, Pos { x: 0.0, y: -260.0 });
                    g.set_pos(
                        out,
                        Pos {
                            x: 260.0,
                            y: -260.0,
                        },
                    );
                    g.connect(Edge {
                        from: (grid, 0),
                        to: (force, 0),
                        delayed: false,
                    })
                    .expect("grid -> force");
                    // Vento forte e para baixo, para a deriva ser óbvia na tela.
                    g.set_param(force, "strength", 8.0);
                    g.set_param(force, "angle", -90.0);
                    (grid, force, out)
                };
                let _ = grid;
                gfx.motion.sinks.push(out);
                let _ = gfx.tools.set_active(&ph2d_editor::ToolId::new("motion"));
                // Nasce SELECIONADA a força, para o artista cair no card dela.
                ph2d_panel_motion_graph::request_graph_selection(vec![force.0]);
                // O GESTO ERRADO: liga a força direto ao output.
                ph2d_panel_motion_graph::push_intent(GraphIntent::Connect {
                    from_node: force.0,
                    from_port: 0,
                    to_node: out.0,
                    to_port: 0,
                });
                eprintln!(
                    "[autofix smoke =1] force.wind ligada DIRETO ao output, SEM integrador \
                     (o setup que nao move nada). O app deve AUTO-INSERIR motion.integrate: a \
                     grid passa a alimentar o integrador, a forca vira o ramo de forcas, e os \
                     pontos passam a DERIVAR para baixo com o vento. Um Ctrl+Z remove SO o \
                     conserto (a forca fica onde estava); outro Ctrl+Z desfaz a conexao. No \
                     frame 90 o integrador e' apagado."
                );
            }
            // =1, frame 90: apaga o integrador — gesto destrutivo, o app NAO re-insere.
            (1, 90) => {
                let gfx = self.gfx.as_mut().expect("gfx");
                let integ = gfx
                    .motion
                    .doc
                    .graph
                    .nodes()
                    .iter()
                    .find(|n| n.type_name == "motion.integrate")
                    .map(|n| n.id.0);
                if let Some(id) = integ {
                    ph2d_panel_motion_graph::push_intent(GraphIntent::DeleteSelection {
                        nodes: vec![id],
                    });
                    eprintln!(
                        "[autofix smoke =1] apaguei o integrador (gesto DESTRUTIVO): os pontos \
                         CONGELAM e o app NAO re-insere — apagar para religar a' mao nunca e' \
                         combatido. Refazer (Ctrl+Shift+Z) devolve o movimento."
                    );
                } else {
                    eprintln!(
                        "[autofix smoke =1] AVISO: nenhum motion.integrate encontrado — o \
                         auto-heal nao rodou. Pare e investigue."
                    );
                }
            }
            // =2, frame 3: um sim que JA FUNCIONA (grid -> integrate -> output) e o
            // artista solta uma force.wind SOBRE o fio de saida (SpliceNode). A forca cai
            // DEPOIS do integrador, onde o accel dela nunca e' consumido. O app REUSA
            // esse integrador (nao insere um segundo) e a forca vira o ramo de forcas ->
            // os pontos DERIVAM (+X, com o strength 3 default do wind).
            (2, 3) => {
                let gfx = self.gfx.as_mut().expect("gfx");
                let out = {
                    let g = &mut gfx.motion.doc.graph;
                    let grid = g.add_node("motion.grid");
                    let integ = g.add_node("motion.integrate");
                    let out = g.add_node("motion.output");
                    g.set_pos(
                        grid,
                        Pos {
                            x: -260.0,
                            y: -260.0,
                        },
                    );
                    g.set_pos(integ, Pos { x: 0.0, y: -260.0 });
                    g.set_pos(
                        out,
                        Pos {
                            x: 260.0,
                            y: -260.0,
                        },
                    );
                    g.connect(Edge {
                        from: (grid, 0),
                        to: (integ, 0), // grid -> integrate.rest
                        delayed: false,
                    })
                    .expect("grid -> integrate.rest");
                    g.connect(Edge {
                        from: (integ, 0),
                        to: (out, 0), // integrate -> output (o sim que ja' roda)
                        delayed: false,
                    })
                    .expect("integrate -> output");
                    out
                };
                gfx.motion.sinks.push(out);
                let _ = gfx.tools.set_active(&ph2d_editor::ToolId::new("motion"));
                // O GESTO: soltar a forca SOBRE o fio integrate -> output.
                ph2d_panel_motion_graph::push_intent(GraphIntent::SpliceNode {
                    to_node: out.0,
                    to_port: 0,
                    to_type: "force.wind",
                    x: 130.0,
                    y: -260.0,
                });
                eprintln!(
                    "[autofix smoke =2] sim que JA FUNCIONA (grid -> integrate -> output); \
                     solto uma force.wind SOBRE o fio de saida. A forca cai DEPOIS do \
                     integrador (o accel dela nunca e' consumido ali). O app deve REUSAR o \
                     integrador que ja' esta' ali (NAO inserir um segundo): a forca vira o \
                     ramo de forcas e os pontos DERIVAM para +X. Confira que existe UM SO' \
                     motion.integrate. Um Ctrl+Z reverte so' o reorder."
                );
            }
            _ => {}
        }
    }
}
