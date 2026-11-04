//! ⭐ **A barra do grafo em cartões** (`docs/CartoesUI/14_CHROME_TIMELINE_MOTION.md`): os mesmos nove chips,
//! os mesmos ids e o mesmo canal `Chrome`, numa pílula flutuante com a tinta e a sombra da lista.

use super::{TOOLBAR_INSET, chip_tooltip, chrome_hit_id};
use ph2d_a11y::NodeId;
use ph2d_editor_core::IconId;
use ph2d_editor_core::interaction::GraphHitKind;
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::panel::cards::CardCtx;
use ph2d_editor_core::panel::cards::banda::{self, Icone, metrics as m};
use ph2d_editor_core::widget::card_ink::CardInk;
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::Theme;

/// O rect da pílula, no canto de baixo à esquerda do grafo, acima da faixa do HUD: centrado na janela,
/// ele tapava cinco dos nove chips (foto de 04/11), e o `layout` do painel ainda não sabe da timeline.
pub(super) fn pilula(rect: Rect, n: usize, hud_h: f32) -> Rect {
    let h = m::FIELD_H + m::SEG_PAD * 2.0;
    let w = banda::icones_w(n) + m::SEG_PAD * 2.0;
    let y = rect.y + rect.h - TOOLBAR_INSET * 2.0 - hud_h - h;
    Rect::new(rect.x + TOOLBAR_INSET, y.max(rect.y), w, h)
}

pub(super) fn draw(
    ctx: &mut PaintCtx,
    rect: Rect,
    theme: Theme,
    hits: &mut Vec<(NodeId, GraphHitKind, Rect)>,
    chips: &[(u16, IconId, bool)],
) {
    let p = pilula(rect, chips.len(), ctx.layout.bottom_hud.h);
    banda::pinta_pilula(ctx, p);
    let icones: Vec<Icone> = chips
        .iter()
        .map(|&(id, icon, on)| Icone {
            id: Some(chrome_hit_id(id)),
            icon,
            on,
        })
        .collect();
    let (x, y) = (p.x + m::SEG_PAD, p.y + m::SEG_PAD);
    let (store, hit) = ctx.host.store_and_hit_index_mut();
    CardCtx::flutuante(
        ctx.scene,
        ctx.text_system,
        store,
        hit,
        CardInk::of_list(theme),
        p.x,
        p.w,
    )
    .icones_em_fila(&icones, x, y);
    for (i, &(id, _, _)) in chips.iter().enumerate() {
        let hit_id = chrome_hit_id(id);
        ctx.host.store_mut().set_tooltip(hit_id, chip_tooltip(id));
        let r = Rect::new(x + m::FIELD_H * i as f32, y, m::FIELD_H, m::FIELD_H);
        hits.push((hit_id, GraphHitKind::Chrome { id }, r));
    }
}

#[cfg(test)]
#[path = "paint_chrome_cartoes_tests.rs"]
mod tests;
