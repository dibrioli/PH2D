// ph2d-chrome-sync:z=178 (dispatch priority, ADR-0107; lower = earlier)
//! **Os quatro diálogos do Pixel P6** (Color Range, Modify, Canvas Size, Sprite Size): as escolhas
//! (âncora, pincel, método, Lock Ratio) guardam-se; um campo que muda refaz os irmãos (bordas ↔
//! tamanho, px ↔ %); o OK arma o pedido e fecha; o Cancel só fecha. A família drena
//! `take_pixel_edit_request`.

use crate::ids;
use crate::interaction::{ContextMenuKind, WidgetEvent};
use crate::screens::hero::HeroScreen;

pub fn apply(hero: &mut HeroScreen, event: WidgetEvent) -> bool {
    let open = hero
        .store
        .context_menu()
        .is_some_and(|r| r.kind == ContextMenuKind::PixelEditDialog);
    let id = match event {
        WidgetEvent::ValueChanged(id) if open => return hero.store.pixel_edit_value_changed(id),
        WidgetEvent::Click(id) => id,
        _ => return false,
    };
    if id == ids::PIXEL_EDIT_OK {
        hero.store.request_pixel_edit();
        return true;
    }
    if id == ids::PIXEL_EDIT_CANCEL {
        hero.store.close_context_menu();
        return true;
    }
    if id == ids::PIXEL_EDIT_LOCK {
        let on = !hero.store.pixel_edit_dialog().lock_ratio;
        hero.store.set_pixel_edit_lock_ratio(on);
        return true;
    }
    if let Some(&(a, ..)) = ids::PIXEL_EDIT_ANCHORS.iter().find(|(_, b, _)| *b == id) {
        hero.store.set_pixel_edit_anchor(a);
        return true;
    }
    if let Some(&(b, ..)) = ids::PIXEL_EDIT_BRUSHES.iter().find(|(_, x, _)| *x == id) {
        hero.store.set_pixel_edit_brush(b);
        return true;
    }
    if let Some(&(m, ..)) = ids::PIXEL_EDIT_METHODS.iter().find(|(_, x, _)| *x == id) {
        hero.store.set_pixel_edit_method(m);
        return true;
    }
    filter_click(hero, id)
}

/// P8: as escolhas dos diálogos de filtro (Preview, Tiled, canais, Outline).
fn filter_click(hero: &mut HeroScreen, id: ph2d_a11y::NodeId) -> bool {
    let s = &mut hero.store;
    if id == ids::PIXEL_FILTER_PREVIEW {
        s.toggle_pixel_filter_preview();
    } else if id == ids::PIXEL_FILTER_TILED {
        s.toggle_pixel_filter_tiled();
    } else if let Some(&(k, ..)) = ids::PIXEL_FILTER_CHANNELS.iter().find(|(_, x, _)| *x == id) {
        s.toggle_pixel_filter_channel(k);
    } else if let Some(&(p, ..)) = ids::PIXEL_FILTER_PLACES.iter().find(|(_, x, _)| *x == id) {
        s.set_pixel_filter_place(p);
    } else if let Some(&(m, ..)) = ids::PIXEL_FILTER_MATRICES.iter().find(|(_, x, _)| *x == id) {
        s.set_pixel_filter_matrix(m);
    } else if let Some(&(b, ..)) = ids::PIXEL_FILTER_BACKGROUNDS
        .iter()
        .find(|(_, x, _)| *x == id)
    {
        s.set_pixel_filter_background(b);
    } else {
        return false;
    }
    true
}
