//! ⭐⭐ **QUAIS PAINÉIS PINTAM ESTE QUADRO, E ONDE** — a travessia do registry em ordem z.
//!
//! ⚠️ **Cortado do `paint.rs` em 2026-08-30 pelo tecto de LOC (703/700), e o corte é por
//! RESPONSABILIDADE:** aquele ficheiro responde *«o que se pinta neste quadro?»* e este responde
//! *«qual painel, e em que rectângulo»* — que é a pergunta que a decisão **D4** tornou móvel.
//!
//! As três coisas que ele faz, e que só juntas fazem sentido:
//!
//! 1. **quem está à frente** de cada encaixe (os outros ocupantes não pintam — é isso que faz de
//!    `n > 1` **abas** em vez de painéis empilhados);
//! 2. **onde cada painel fica** ([`crate::panel::PaintCtx::slot`]), resolvido uma vez por quadro;
//! 3. **as filas de abas e as zonas de largada**, que são a superfície do gesto que move um painel.

use super::HeroScreen;
use super::PANEL_Z_ORDER_FALLBACK;
use crate::screens::layout::HeroLayout;
use crate::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_vector::VectorScene;

/// Ver o cabeçalho do módulo.
pub(super) fn walk(
    hero: &mut HeroScreen,
    layout: &HeroLayout,
    viewport: Rect,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
) {
    // Wave 5 stage D — paint each panel via the PanelRegistry in
    // z-order. Bottom-first, so the panel most recently clicked /
    // dragged / opened sits on top. Panels that haven't been touched
    // yet inherit a default order at the bottom (fallback list below
    // also covers floating panels that have their own panel rects:
    // GAL_PANEL + GS_PANEL).
    //
    // INSP_BLENDER_PICKER is intentionally NOT in the panel
    // registry — it's painted out-of-band AFTER every floating panel
    // (see `paint_blender_picker_demo` below) so it sits on top of
    // every other panel regardless of z order.
    //
    // Each manifest's `paint_fn` owns its full per-frame logic:
    // visibility check + lazy default rect + drag/resize clamp +
    // chrome publish + actual paint + content_h publish + scroll
    // clamp + stale-rect cleanup on hide. Adding a new panel needs
    // zero edits to this iteration — drop `PANEL_MANIFEST` in the
    // panel module + 1 line in `panel_registry::PANEL_REGISTRY`.
    let mut z_order: Vec<ph2d_a11y::NodeId> = hero.store.panel_z_order().to_vec();
    for &fallback in PANEL_Z_ORDER_FALLBACK {
        if !z_order.contains(&fallback) {
            z_order.push(fallback);
        }
    }
    // ADR-0029 Phase D: legacy fn-pointer dispatch deleted. Every
    // in-tree panel lives in `crate::panel::PANEL_REGISTRY` as a
    // typed `Panel<State>`. The z-order walk resolves each id to its
    // typed entry; ids that don't match (e.g. `INSP_BLENDER_PICKER`,
    // painted out-of-band below) are silently skipped.
    // ⭐⭐⭐ **Os ocupantes que não estão à frente do seu encaixe não pintam** — é isto que faz de
    // `n > 1` num encaixe **abas** em vez de painéis empilhados. ⚠️ Vazio enquanto cada encaixe
    // tiver no máximo um ocupante, que é o estado de omissão do app.
    let hidden = super::slot_tabs::hidden_by_tabs(hero);
    // ⭐⭐ **O rect do encaixe de CADA painel, resolvido uma vez** — é o que o `PaintCtx::slot`
    // entrega. ⚠️ Já sem a faixa de abas: o `reserve_slot_tabs` empurrou as colunas antes de o
    // layout ser publicado, e o `slot_rects` lê-as de lá.
    let slot_rects = layout.slot_rects(super::slot_tabs::occupied(hero));
    let panel_slots: std::collections::BTreeMap<ph2d_a11y::NodeId, Rect> =
        crate::panel::with_registry_opt(|reg| {
            reg.panels()
                .iter()
                .map(|p| {
                    (
                        p.manifest.panel_node_id,
                        slot_rects.get(super::slot_tabs::slot_of(hero, &p.manifest)),
                    )
                })
                .collect()
        })
        .unwrap_or_default();
    // ⛔⛔ **Quem não pinta tem de LARGAR o rect que publicou.** O `paint_fn` de cada painel é
    // quem chama `clear_panel_rect` ao ficar invisível — saltá-lo deixaria o rect do quadro
    // anterior no store, e ele alimenta o `DockSides::from_published` (a largura da área de
    // desenho) e o gate da D1. *Um painel escondido por uma aba continuaria a reservar coluna.*
    for id in &hidden {
        hero.store.clear_panel_rect(*id);
    }
    crate::panel::with_registry_opt(|reg| {
        for panel_id in z_order {
            if hidden.contains(&panel_id) {
                continue;
            }
            if let Some(idx) = reg.find_by_panel_node_id(panel_id) {
                // Hit barrier: register the panel rect BEFORE the
                // widgets inside `panel.paint()` so the gizmo's hit
                // rects (registered earlier this frame) don't bleed
                // through the panel surface. `HitIndex::hit()` walks
                // back-to-front, so internal panel widgets registered
                // by `paint()` below still outrank this barrier — only
                // empty panel area falls back to it. Enio 2026-05-25:
                // "alças do gizmo da sprite podem ser acessadas
                // através dos painéis. Isso não pode acontecer."
                if let Some(panel_rect) = hero.store.panel_rect(panel_id) {
                    hero.hit_index.register(panel_id, panel_rect);
                }
                let mut typed_ctx = crate::panel::PaintCtx {
                    host: hero,
                    layout,
                    slot: panel_slots
                        .get(&panel_id)
                        .copied()
                        .unwrap_or(layout.draw_area),
                    viewport,
                    scene,
                    text_system,
                };
                reg.panels_mut()[idx].paint(&mut typed_ctx);
            }
        }
    });
    // ⭐⭐⭐ **AS ABAS, depois dos painéis** — o `HitIndex` caminha de trás para a frente, então
    // registá-las aqui põe-nas acima da barreira de hit que o painel da frente instalou. A faixa
    // é zero-altura em todo encaixe com menos de dois ocupantes.
    for slot in crate::screens::slot::Slot::ALL {
        let bar = layout.slot_tabs[slot as usize];
        if bar.h <= 0.0 {
            continue;
        }
        let occ = super::slot_tabs::occupants(hero, slot);
        let selected = occ.last().map(|o| o.node);
        super::slot_tabs::paint_slot_tabs(
            bar,
            &occ,
            selected,
            scene,
            text_system,
            hero.theme,
            &mut hero.hit_index,
            &hero.store,
        );
    }
    // ⭐⭐⭐ **AS ZONAS DE LARGADA, por cima das colunas** — elas só existem com um arrasto de aba em
    // curso, e o encaixe que o painel não permite **não é pintado**: é assim que a D1 torna o gesto
    // errado inexprimível em vez de recusado.
    super::slot_tabs::paint_drag_overlay(hero, scene, text_system, hero.theme);
}
