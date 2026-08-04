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

use ph2d_nodegraph::graph::{Edge, Pos};
use ph2d_panel_motion_graph::GraphIntent;

/// O modo: `0` off, `1` ligado.
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
        match f {
            // Frame 3: monta grid -> force.wind + output SOLTO, abre a tool Motion, e
            // empurra o gesto ERRADO (Connect force -> output, sem integrador). O
            // bridge drena o intent, conecta e auto-conserta no mesmo frame.
            3 => {
                let gfx = self.gfx.as_mut().expect("gfx");
                let (grid, force, out) = {
                    let g = &mut gfx.motion.doc.graph;
                    let grid = g.add_node("motion.grid");
                    let force = g.add_node("force.wind");
                    let out = g.add_node("motion.output");
                    g.set_pos(grid, Pos { x: -220.0, y: -260.0 });
                    g.set_pos(force, Pos { x: 0.0, y: -260.0 });
                    g.set_pos(out, Pos { x: 260.0, y: -260.0 });
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
            // Frame 90: apaga o integrador — gesto destrutivo, o app NAO re-insere.
            90 => {
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
            _ => {}
        }
    }
}
