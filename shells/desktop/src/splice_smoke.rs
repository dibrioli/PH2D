//! **A cena pronta para o smoke do SPLICE** (`PH2D_SPLICE_SMOKE=1`, W-I5).
//!
//! Uma cadeia `motion.grid → motion.scale → motion.output` com a ferramenta Motion ativa (o
//! grafo à vista). O gesto que a wave adiciona: **R-click SOBRE um fio → o menu de nós → o nó
//! escolhido é INSERIDO no fio** (fonte → novo → alvo), a generalização do double-click-reroute
//! (doc 45) de um reroute fixo para qualquer tipo. O shell valida e RECUSA (com toast, o fio
//! fica intacto) se o tipo não faz a ponte.
//!
//! TESTE: R-click no fio entre **scale** e **output** (ou entre grid e scale) → o menu abre →
//! escolha, por exemplo, **`motion.noise`** ou **`force.wind`** → ele entra NO fio (a cadeia
//! vira grid → scale → noise → output), com UM Ctrl+Z desfazendo a inserção inteira. R-click em
//! canvas VAZIO ainda só adiciona o nó solto (sem fio embaixo, sem splice). Um tipo que não
//! encaixa (um source como `motion.grid`) é recusado e o fio original fica.

use ph2d_nodegraph::graph::{Edge, Graph, NodeId};

/// Monta `grid → scale → output`. Devolve o `motion.output` (o sink a avaliar). Os fios entre
/// os nós são exatamente o que o R-click vai costurar.
fn chain(g: &mut Graph) -> NodeId {
    let grid = g.add_node("motion.grid");
    let scale = g.add_node("motion.scale");
    let out = g.add_node("motion.output");
    g.set_param(grid, "rows", 3.0);
    g.set_param(grid, "cols", 6.0);
    g.set_param(scale, "amount", 0.4);
    for (from, to) in [(grid, scale), (scale, out)] {
        g.connect(Edge {
            from: (from, 0),
            to: (to, 0),
            delayed: false,
        })
        .expect("splice-smoke edge");
    }
    out
}

/// Ligado? Lido UMA vez (o prólogo do frame não paga um `getenv` por quadro).
fn on() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| std::env::var("PH2D_SPLICE_SMOKE").is_ok_and(|v| v != "0"))
}

static FRAME: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

impl crate::App {
    /// Roda no prólogo do frame, ao lado do `gradient_smoke`. No-op sem a env.
    pub(crate) fn splice_smoke(&mut self) {
        use std::sync::atomic::Ordering;
        if !on() || self.gfx.is_none() || FRAME.fetch_add(1, Ordering::Relaxed) != 4 {
            return;
        }
        let gfx = self.gfx.as_mut().expect("gfx");
        let sink = chain(&mut gfx.motion.doc.graph);
        // Arruma o layout (sem marcar nó nenhum — o gesto é sobre um FIO, não um nó).
        crate::smoke_layout::arrange_and_mark(&mut gfx.motion.doc, &[]);
        gfx.motion.sinks.push(sink);
        let _ = gfx.tools.set_active(&ph2d_editor::ToolId::new("motion"));
        eprintln!(
            "[splice smoke] Uma cadeia grid -> scale -> output, a ferramenta Motion ativa (o \
             grafo a vista).\n  \
             TESTE: R-CLICK SOBRE UM FIO (entre scale e output, ou grid e scale) -> o menu de \
             nos abre -> escolha por exemplo 'motion.noise' ou 'force.wind' -> ele e INSERIDO no \
             fio (grid -> scale -> noise -> output), com UM Ctrl+Z desfazendo tudo. R-click em \
             canvas VAZIO ainda so adiciona o no solto (sem splice). Um source que nao encaixa \
             (motion.grid) e recusado e o fio original fica."
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_node_registry::NodeRegistry;

    /// The splice-smoke scene is well-typed and carries the two wires the R-press aims at —
    /// a typo'd chain (or a missing edge) would leave nothing to splice into, which is what the
    /// artist would report as "the smoke is broken". FALSIFIED if it does not validate or the
    /// scale → output wire is absent.
    #[test]
    fn the_smoke_chain_is_well_typed_and_has_a_wire_to_splice() {
        let mut reg = NodeRegistry::new();
        ph2d_node_registry_init::register_all_nodes(&mut reg).expect("registry builds");
        let mut g = Graph::new();
        let out = chain(&mut g);
        g.validate(&reg)
            .expect("the splice-smoke chain is well-typed");
        // The wire the smoke tells the artist to R-click: something feeds `output`'s input 0.
        assert!(
            g.edges()
                .iter()
                .any(|e| e.to.0 == out && e.to.1 == 0 && !e.delayed),
            "output has a wire in — the one to splice onto"
        );
    }
}
