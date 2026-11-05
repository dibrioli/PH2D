//! Widget Gallery panel paint — ADR-0029 Phase C.3 port of the legacy
//! `paint_thunk`.
//!
//! Full per-frame logic:
//! - Visibility gate via [`PanelHostInternal::panel_visible`].
//! - Stale-rect cleanup on hide so a closed gallery doesn't shadow
//!   `GS_PANEL` in `panel_at` lookups (BTreeMap key order, scroll
//!   wheel routing — the comment in the legacy thunk explains the
//!   regression this guards against).
//! - Lazy default rect on first paint (writes back to `state.rect`).
//! - Drag / resize clamp via `widget::panel_chrome::clamp_panel_rect`.
//! - Chrome publish (`set_panel_rect`).
//! - Delegated showcase paint (`widget::showcase::paint_showcase_body`).
//! - `content_h` / `visible_h` publish + scroll clamp — pela porta `scroll_area`
//!   (`close_showcase_body`).

use crate::WidgetGalleryPanel;
use crate::state::WidgetGalleryState;
use ph2d_editor_core::ids;
use ph2d_editor_core::panel::scroll_area::close_showcase_body;
use ph2d_editor_core::panel::{PaintCtx, Panel};
use ph2d_editor_core::widget::panel_chrome::clamp_panel_rect;
use ph2d_editor_core::widget::showcase::paint_showcase_body;
use ph2d_editor_core::zones::Rect;

pub(crate) fn paint(state: &mut WidgetGalleryState, ctx: &mut PaintCtx) {
    smoke_opens_once(ctx);
    smoke_opens_the_side(state);
    smoke_scale_com_zero(state, ctx);
    smoke_holds_the_render(state);
    if !ctx.host.panel_visible(WidgetGalleryPanel::ID) {
        // Stale-rect cleanup: without this, `panel_at` keeps returning
        // GAL_PANEL after the Gallery was closed, and BTreeMap key
        // ordering (950 < 1000) makes that stale rect shadow GS_PANEL
        // for overlapping cursor positions — wheel scroll on the Grid
        // Settings panel then routes to a closed GAL_PANEL and looks
        // like "scroll wheel doesn't work" to the user.
        ctx.host.store_mut().clear_panel_rect(ids::GAL_PANEL);
        return;
    }
    // ⭐ CartoesUI/08: a lista de cartões no encaixe do dock; `PH2D_CARDS=0` = a janela de hoje.
    if crate::cards::cards_on() {
        crate::cards::paint(state, ctx);
        return;
    }
    let base_rect = match state.rect {
        Some(r) => r,
        None => {
            let r = default_rect(ctx.layout.viewport.w, ctx.layout.viewport.h, ctx.slot.w);
            state.rect = Some(r);
            r
        }
    };
    let viewport = ctx.viewport;
    // Read drag/resize deltas and clamp, then write back any clamp
    // corrections + publish the final panel rect to the store. The
    // joint `store_and_hit_index_mut` returns `&WidgetStore`
    // immutable, so we use `store_mut()` for the writes here.
    let store_imm = ctx.host.store();
    let gal_off = store_imm.blender_picker_offset(ids::GAL_PANEL);
    let gal_resize = store_imm.panel_resize_delta(ids::GAL_PANEL);
    let (gallery_rect, gal_clamped_off, gal_clamped_resize) =
        clamp_panel_rect(base_rect, gal_off, gal_resize, viewport);
    {
        let store = ctx.host.store_mut();
        if (gal_clamped_off.0 - gal_off.0).abs() > f32::EPSILON
            || (gal_clamped_off.1 - gal_off.1).abs() > f32::EPSILON
        {
            store.set_blender_picker_offset(ids::GAL_PANEL, gal_clamped_off.0, gal_clamped_off.1);
        }
        if (gal_clamped_resize.0 - gal_resize.0).abs() > f32::EPSILON
            || (gal_clamped_resize.1 - gal_resize.1).abs() > f32::EPSILON
        {
            store.set_panel_resize_delta(
                ids::GAL_PANEL,
                gal_clamped_resize.0,
                gal_clamped_resize.1,
            );
        }
        store.set_panel_rect(ids::GAL_PANEL, gallery_rect);
    }
    let theme = ctx.host.theme();
    let body = {
        let (store, hit_index) = ctx.host.store_and_hit_index_mut();
        paint_showcase_body(
            gallery_rect,
            ctx.scene,
            ctx.text_system,
            theme,
            hit_index,
            store,
        )
    };
    // ⭐ A PORTA da rolagem fecha aqui (o `store` já é mutável): barra com a TRILHA, as duas
    // alturas, o clamp — e só depois o cromo do título é re-registado por cima.
    close_showcase_body(body, ctx);
}

/// `PH2D_GALLERY_SMOKE=1`: a galeria abre-se sozinha no primeiro quadro (o smoke do doc 08 §3).
/// `PH2D_GALLERY_SMOKE_SCROLL=<px>`: já rolada — a foto dos cartões de baixo (o XTest não chega à
/// sessão virtual da `fotografa_cena.sh`, a roda não se simula).
fn smoke_opens_once(ctx: &mut PaintCtx) {
    use std::sync::atomic::{AtomicBool, Ordering};
    static FEITO: AtomicBool = AtomicBool::new(false);
    if std::env::var("PH2D_GALLERY_SMOKE").as_deref() == Ok("1")
        && !FEITO.swap(true, Ordering::Relaxed)
    {
        ctx.host.set_panel_visible(WidgetGalleryPanel::ID, true);
    }
    // A cada quadro: no 1.º a lista ainda não sabe a própria altura e o clamp devolve-a ao topo.
    static ROLA: std::sync::OnceLock<Option<f32>> = std::sync::OnceLock::new();
    let rola = ROLA.get_or_init(|| {
        std::env::var("PH2D_GALLERY_SMOKE_SCROLL")
            .ok()
            .and_then(|v| v.parse().ok())
    });
    if let Some(y) = *rola {
        ctx.host.store_mut().set_panel_scroll(ids::GAL_PANEL, y);
    }
}

/// `PH2D_GALLERY_SMOKE_SIDE=1`: a linha do efeito já aberta — a foto do painel à esquerda (F43).
fn smoke_opens_the_side(state: &mut WidgetGalleryState) {
    use std::sync::atomic::{AtomicBool, Ordering};
    static FEITO: AtomicBool = AtomicBool::new(false);
    if std::env::var("PH2D_GALLERY_SMOKE_SIDE").as_deref() == Ok("1")
        && !FEITO.swap(true, Ordering::Relaxed)
    {
        state.choices.choose(crate::state::Pick::SideOpen, 0);
    }
}

/// `PH2D_GALLERY_SMOKE_SCALE=1`: a foto do dono (04/11), corrente fechada com Scale X `-0.004` e Y `9`:
/// a caixa do X tem de ler `0.00`, sem sinal.
fn smoke_scale_com_zero(state: &mut WidgetGalleryState, ctx: &mut PaintCtx) {
    use std::sync::atomic::{AtomicBool, Ordering};
    static FEITO: AtomicBool = AtomicBool::new(false);
    if std::env::var("PH2D_GALLERY_SMOKE_SCALE").as_deref() == Ok("1")
        && !FEITO.swap(true, Ordering::Relaxed)
    {
        let store = ctx.host.store_mut();
        store.set_number_value(crate::ids::SCALE_X, -0.004); // LITERAL-PX-OK: dado do objeto fictício
        store.set_number_value(crate::ids::SCALE_Y, 9.0); // LITERAL-PX-OK: dado do objeto fictício
        state.choices.set(crate::state::Flag::ScaleLink, true);
    }
}

/// `PH2D_GALLERY_SMOKE_RENDER=<quadro>`: a corrida do cartão Render parada nesse quadro (a foto).
fn smoke_holds_the_render(state: &mut WidgetGalleryState) {
    use std::sync::atomic::{AtomicBool, Ordering};
    static FEITO: AtomicBool = AtomicBool::new(false);
    if let Some(q) = std::env::var("PH2D_GALLERY_SMOKE_RENDER")
        .ok()
        .and_then(|v| v.parse().ok())
        && !FEITO.swap(true, Ordering::Relaxed)
    {
        state.render.prende_em(q);
    }
}

/// Default panel geometry when the gallery is first opened. Width
/// matches the live Inspector (`layout.inspector.w`) so the showcase
/// renders at the same dimensions peripheral agents will see when
/// the actual panel reads it. Height clamps to the viewport with
/// 8 px breathing margins.
pub fn default_rect(viewport_w: f32, viewport_h: f32, inspector_w: f32) -> Rect {
    let w = inspector_w.max(280.0).min(viewport_w - 16.0); // LITERAL-PX-OK: gallery min width 280 + viewport margin 16 (chrome-specific)
    let h = 720.0_f32.min(viewport_h - 16.0).max(420.0); // LITERAL-PX-OK: gallery default height 720 + viewport margin + min 420 (chrome-specific)
    let x = ((viewport_w - w) * 0.5).max(8.0); // LITERAL-PX-OK: viewport edge inset
    let y = ((viewport_h - h) * 0.5).max(8.0); // LITERAL-PX-OK: viewport edge inset
    Rect::new(x, y, w, h)
}
