//! **Fase do quadro: O DRENO DO BARRAMENTO** — os pedidos do quadro (`pending_*` e os seus irmãos) e o
//! `for action in bus.drain()` que os enche, devolvidos juntos (OBRA 2 da `line/render-loop`, 2026-09-13).
//!
//! ⭐ **Os pedidos são UM valor e os braços são sub-drenos** (`line/render-bodies`, 2026-09-13): cada pedido é um
//! campo do [`DrainOut`] (o `Default` é o valor com que a declaração o criava, e o comentário dela mora no campo),
//! e o `match` partiu-se por ASSUNTO em `fase_bus_*`, chamadas pela ordem dos braços. Cada uma devolve à seguinte
//! o pedido que não é dela, e por isso um pedido cai num braço só, como no `match` (as guardas incluídas).
//! ⚠️ No corpo de um braço a única troca é `pd.<pedido>` onde ele escrevia `<pedido>`, e só em CÓDIGO. A nota que
//! esta fase trazia — *partir por braço obrigaria o corpo a escrever `*pedido = …` por referências* — media a
//! partição por referências soltas; um contexto nomeado não as pede.

use super::*;

/// A timeline docada, os pedidos que esperam outro ponto do quadro e a barra do topo.
#[path = "fase_bus_chrome.rs"]
mod bus_chrome;
/// As três terças da cadeia do CLIQUE de um painel de ferramenta.
#[path = "fase_bus_clicks.rs"]
mod bus_clicks;
/// A Hierarquia: os interruptores, a árvore, os verbos de instância e de linha, a selecção e o renomear.
#[path = "fase_bus_hierarchy.rs"]
mod bus_hierarchy;
/// O Inspector: a sprite, as secções, o cartão da instância, a física, a visibilidade e os nomes.
#[path = "fase_bus_inspector.rs"]
mod bus_inspector;
/// O canal painel → ferramenta: a activação, o `PanelEvent` de uma ferramenta e as cadeias do campo.
#[path = "fase_bus_tool_panel.rs"]
mod bus_tool_panel;

#[path = "fase_bus_drain_out.rs"]
mod pedidos;
pub(super) use pedidos::DrainOut;

impl crate::App {
    /// Ver o cabeçalho do módulo.
    pub(super) fn fase_bus_drain(&mut self) -> Option<DrainOut> {
        // O `gfx` re-derivado; os guardas do quadro já correram na `fase_chrome_clock`.
        let gfx = self.gfx.as_mut()?;
        let FrameGfx {
            sim, hero_screen, ..
        } = FrameGfx::of(gfx);
        // O bloco do quadro só chama esta fase com o `HeroScreen` vivo.
        let hero = hero_screen.as_mut()?;
        // ─────────────────────────────────────────────────────────
        // Wave 2.5 PR 11.8 closeout — consolidated bus drain.
        // ─────────────────────────────────────────────────────────
        //
        // Previously, each of the 18 EditorAction variants had its
        // own filter-and-replace block (one per drain site, ~20 LOC
        // of "drain, capture this variant, push others back" each).
        // Now we drain the bus ONCE at the top of this section,
        // categorize every variant into per-kind locals, and the
        // dispatch sites further down just read `if let Some(x) = X`.
        //
        // First-wins for most variants (matches the old
        // `found.is_none()` short-circuit). Latest-wins for
        // `InspectorNameEdit` (preserves the pre-bus Option
        // coalescing that drained at most one SetComponent per
        // frame). `Bgremoval` is NOT categorized here — it keeps
        // a separate filter-and-replace at its original site so
        // its `bgremoval_active` gate runs AFTER any same-frame
        // `ActivateTool { tool_id: "bgremoval" }` fires (1-frame
        // defer edge case).
        //
        // Audit 2026-05-26 F1: 6 flags hardcoded per-tool (`activate_bgremoval`
        // etc.) substituídas por uma única option `pending_image_tool_activation`.
        // O drain único abaixo usa `installed_registry().cluster("image_tools")`
        // + `Tool::label()` para dispatch data-driven. Painter + os 5 image-tools
        // pré-existentes flow pelo mesmo canal — anti-padrão Image Tools Bugs
        // §2.b fechado neste ponto da render loop.
        let mut pd = DrainOut {
            // ⚠️ **O osso seleccionado lê-se AQUI, antes de o mundo ser emprestado mutável** — os
            // verbos lá em baixo já seguram `sim`, e uma leitura de `self` no meio deles não
            // compila. O valor é do QUADRO, e é o mesmo que o gesto e o overlay usam.
            osso_selecionado: crate::bone_gesture::selected_bone(sim, hero.gizmo.iter_selected()),
            // ⭐ **A selecção CRUA, guardada aqui pela mesma razão que o `osso_selecionado`**: o
            // `hero` é uma vista do `gfx`, e quem a lê lá em baixo (o *Bind* da 2.ª mídia) já o
            // tem emprestado de outra maneira. *Ler o valor uma vez é o que torna a pergunta
            // alcançável nos dois sítios.*
            selecao_bits: hero.gizmo.iter_selected().collect(),
            // BulkSelect (T2.0): the live selection (primary + extras),
            // captured before the drain so an Inspector sprite edit can
            // fan out to every selected sprite. Only allocated for a
            // MULTI-selection; single-select takes the empty path and the
            // edit's own `entity_bits` (no per-frame alloc — audit D-5).
            inspector_selection: if hero.gizmo.selected_len() > 1 {
                hero.gizmo.iter_selected().collect()
            } else {
                Vec::new()
            },
            ..DrainOut::default()
        };
        // ⚠️ **A fila sai do `HeroScreen` pela duração do dreno**: o `drain` emprestava-a, e os sub-drenos são
        // métodos da `App`. Nenhum braço lhe escreve, e ela volta com a capacidade dela.
        let mut bus = std::mem::take(&mut hero.bus);
        for action in bus.drain() {
            // Pela ORDEM dos braços do `match` de sempre: cada sub-dreno devolve o pedido que não é dele, e o `_` da
            // barra do topo cala o que nenhum braço toma.
            let _ = self
                .fase_bus_tool_panel(action, &mut pd)
                .and_then(|a| self.fase_bus_timeline_panel(a))
                .and_then(|a| self.fase_bus_tool_requests(a, &mut pd))
                .and_then(|a| self.fase_bus_hierarchy_tree(a, &mut pd))
                .and_then(|a| self.fase_bus_hierarchy_rows(a, &mut pd))
                .and_then(|a| self.fase_bus_sprite_ops(a, &mut pd))
                .and_then(|a| self.fase_bus_inspector_sections(a, &mut pd))
                .and_then(|a| self.fase_bus_inspector_instance(a, &mut pd))
                .and_then(|a| self.fase_bus_physics_sections(a, &mut pd))
                .and_then(|a| self.fase_bus_inspector_identity(a, &mut pd))
                .and_then(|a| self.fase_bus_topbar(a));
        }
        let gfx = self.gfx.as_mut()?;
        FrameGfx::of(gfx).hero_screen.as_mut()?.bus = bus;
        Some(pd)
    }
}
