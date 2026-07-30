//! **A cena pronta para o smoke do SPLICE** (`PH2D_SPLICE_SMOKE=1`, W-I5).
//!
//! Uma cadeia `motion.grid → motion.scale → motion.output` com a ferramenta Motion ativa (o
//! grafo à vista). O gesto que a wave adiciona: **R-click SOBRE um fio → o menu de nós → o nó
//! escolhido é INSERIDO no fio** (fonte → novo → alvo), a generalização do double-click-reroute
//! (doc 45) de um reroute fixo para qualquer tipo. O shell valida e RECUSA (com toast, o fio
//! fica intacto) se o tipo não faz a ponte.
//!
//! TESTE: R-click no fio entre **scale** e **output** (ou entre grid e scale) → o menu abre →
//! escolha **`motion.twist`** (deformer, `angle` default 90°) ou **`motion.spherize`** → ele
//! entra NO fio (grid → scale → twist → output) e **o grid se deforma NA HORA**, com UM Ctrl+Z
//! desfazendo a inserção inteira. R-click em canvas VAZIO ainda só adiciona o nó solto (sem
//! splice). Um source que não encaixa (`motion.grid`) é recusado e o fio original fica.
//!
//! O INVERSO do splice também está aqui: SELECIONE o nó do meio (ex. **scale**, ou o **twist**
//! recém-splicado) e aperte **Delete** — a cadeia se RE-CONECTA (grid -> output), em vez de
//! ficar cortada (delete-and-reconnect, o Ctrl+X do Blender). Deletar um nó de PONTA (grid ou
//! output) só o remove, sem religar (não há o que carregar).
//!
//! ⊙ SNAP MAGNÉTICO: arraste um fio de um socket e SOLTE PERTO (não exatamente em cima) de um
//! socket compatível — o fio PULA para ele e conecta (raio ~22 px). Soltar em canvas realmente
//! VAZIO ainda abre o smart-connect; um card colapsado ainda oferece o menu de portas.
//!
//! ⊙ TROCA-NO-DROP: um fio solto SOBRE um input JÁ ocupado SUBSTITUI o que o alimentava (em vez de
//! recusar com "input already wired") — arraste um fio do socket de SAÍDA do **grid** e solte no
//! input do **output** (hoje alimentado pelo **scale**): o output passa a vir direto do grid e o
//! scale se solta. Um `pre` (feedback) expert é preservado; re-soltar o mesmo fio é no-op.
//!
//! ⊙ DROP NO CORPO: um fio solto no CORPO de um nó (não num socket) conecta ao 1º input LIVRE
//! compatível — não precisa acertar o socket. Todos os inputs desta cadeia estão ocupados, então
//! R-click no canvas → adicione um nó com input livre (ex. **`motion.twist`**) e arraste um fio do
//! **grid** até o CORPO dele: conecta. Um card colapsado ainda vai pelo menu de portas, e soltar no
//! CORPO do próprio nó-fonte é ignorado (não auto-conecta).
//!
//! ⚠️ **Um nó de FORÇA (`force.wind`/`force.attractor`/…) NÃO move nada sozinho** — ele só
//! ACUMULA na coluna `accel`, e quem a aplica é o `motion.integrate` (semântica Houdini, entradas
//! `rest`+`forces`). Splicar uma força numa cadeia linear sem integrador é inerte por DESIGN do
//! motor, não falha do splice. Por isso o exemplo aqui é um **deformer** (efeito direto na
//! posição), não uma força.

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
             nos abre -> escolha 'motion.twist' (deformer, angle 90 por default) ou \
             'motion.spherize' -> ele e INSERIDO no fio (grid -> scale -> twist -> output) e o \
             GRID SE DEFORMA NA HORA, com UM Ctrl+Z desfazendo tudo. R-click em canvas VAZIO ainda \
             so adiciona o no solto (sem splice). Um source que nao encaixa (motion.grid) e \
             recusado e o fio original fica.\n  \
             INVERSO: selecione o no do MEIO (scale, ou o twist splicado) e aperte DELETE -> a \
             cadeia se RE-CONECTA (grid -> output), em vez de ficar cortada (o Ctrl+X do Blender). \
             Deletar uma PONTA (grid/output) so remove.\n  \
             SNAP: arraste um fio e SOLTE PERTO (nao em cima) de um socket compativel -> o fio \
             PULA e conecta. Soltar em canvas VAZIO ainda abre o smart-connect.\n  \
             TROCA: solte um fio sobre um input JA ocupado -> ele SUBSTITUI o que alimentava. \
             Arraste do output do grid e solte no input do output (alimentado pelo scale) -> o \
             output passa a vir do grid e o scale se solta.\n  \
             CORPO: solte um fio no CORPO de um no (nao no socket) -> conecta ao 1o input LIVRE. \
             Como a cadeia esta toda ocupada, adicione um no com input livre (motion.twist) via \
             R-click e arraste um fio do grid ate o corpo dele.\n  \
             NOTA: uma FORCA (force.wind/attractor) NAO move nada sozinha -- ela so acumula em \
             'accel', e quem aplica e o motion.integrate. Por isso o exemplo e um deformer, nao \
             uma forca."
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_node_registry::NodeRegistry;
    use ph2d_nodegraph::attr::Column;
    use ph2d_nodegraph::cook::Cook;

    fn built_registry() -> NodeRegistry {
        let mut reg = NodeRegistry::new();
        ph2d_node_registry_init::register_all_nodes(&mut reg).expect("registry builds");
        reg
    }

    /// The instance positions the sink renders — the `P` (Vec2) column, cooked at t=0.
    fn positions(g: &Graph, reg: &NodeRegistry, out: NodeId) -> Vec<[f32; 2]> {
        let mut cook = Cook::new();
        cook.cook(g, reg, out, 0.0)
            .expect("scene cooks")
            .iter()
            .next()
            .and_then(|s| match s.as_stream().get("P") {
                Some(Column::Vec2(v)) => Some(v.clone()),
                _ => None,
            })
            .expect("the output stream carries a P (position) column")
    }

    /// The splice-smoke scene is well-typed and carries the two wires the R-press aims at —
    /// a typo'd chain (or a missing edge) would leave nothing to splice into, which is what the
    /// artist would report as "the smoke is broken". FALSIFIED if it does not validate or the
    /// scale → output wire is absent.
    #[test]
    fn the_smoke_chain_is_well_typed_and_has_a_wire_to_splice() {
        let reg = built_registry();
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

    /// **The suggested example VISIBLY changes the render** — splicing `motion.twist` (its `angle`
    /// defaults to 90°) into the chain, wiring only its primary port the way the real splice does,
    /// deforms every instance. This is the guard against the smoke naming an INERT node again: the
    /// `force.wind` it used to suggest only writes `accel` and moves NOTHING without an integrator
    /// (Enio's report). FALSIFIED if the spliced deformer leaves the positions untouched — which is
    /// exactly what a force would do here.
    #[test]
    fn splicing_the_suggested_deformer_visibly_changes_the_render() {
        let reg = built_registry();

        let mut base = Graph::new();
        let out_a = chain(&mut base);
        let before = positions(&base, &reg, out_a);

        // The same chain with a twist spliced into scale → output — connecting ONLY the primary
        // input (port 0), exactly what `splice_into_wire` does. If `motion.twist` needed a second
        // wire to work, the real splice would leave it just as unconnected, so this mirrors it.
        let mut spliced = Graph::new();
        let grid = spliced.add_node("motion.grid");
        let scale = spliced.add_node("motion.scale");
        let twist = spliced.add_node("motion.twist");
        let out_b = spliced.add_node("motion.output");
        spliced.set_param(grid, "rows", 3.0);
        spliced.set_param(grid, "cols", 6.0);
        spliced.set_param(scale, "amount", 0.4);
        for (from, to) in [(grid, scale), (scale, twist), (twist, out_b)] {
            spliced
                .connect(Edge {
                    from: (from, 0),
                    to: (to, 0),
                    delayed: false,
                })
                .expect("edge");
        }
        spliced
            .validate(&reg)
            .expect("the spliced chain is well-typed (the splice would commit)");
        let after = positions(&spliced, &reg, out_b);

        assert_eq!(before.len(), after.len(), "same instance count");
        assert_ne!(
            before, after,
            "the spliced twist deformed the grid — a force would have moved nothing"
        );
    }
}
