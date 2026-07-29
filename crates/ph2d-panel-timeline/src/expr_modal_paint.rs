//! **The Expression modal — the painter** (plano 10 W1).
//!
//! Two columns and a formula bar:
//!
//! ```text
//! ┌──────────────────────────────────────────────────┐
//! │ Expression — Ball · Position Y               [x] │
//! ├────────────────┬─────────────────────────────────┤
//! │ ⌕ search       │ ◉ Shake            →  12.43     │
//! │ ▾ Life     (6) │     Speed   ▬▬▬▬▬──   2.0       │
//! │ ▾ Wave     (9) │     Amount  ▬▬▬───── 30.0       │
//! │ …              │ [ + ]                           │
//! ├────────────────┴─────────────────────────────────┤
//! │ fx  min(max(value + wiggle(2, 30), -10), 10)     │
//! ├──────────────────────────────────────────────────┤
//! │                       [ Cancel ]      [ Apply ]  │
//! └──────────────────────────────────────────────────┘
//! ```
//!
//! ⚠️ The **third** column (the live preview) is W2. It is absent rather than
//! empty on purpose: a blank box is a promise, and a card that promises a preview
//! and shows nothing reads as broken rather than unfinished.
//!
//! ⚠️ Nothing here scrolls, and the geometry says why. The body is
//! [`BODY_SLOTS`] rows of `ROW_H_PX`; the gallery spends one on the search field
//! and shows either the NINE families or ONE family's recipes (the largest family
//! has ten, so a family always fits); the sheet spends `1 + knobs` per row and
//! **says how many rows it could not show** rather than truncating in silence.

use ph2d_editor_core::interaction::{InteractiveState, TimelineHitKind, WidgetStore};
use ph2d_editor_core::paint::{fill_rounded_rect, paint_text, resolve, stroke_rounded_rect};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::widget::ButtonState;
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{ColorToken, ROW_H_PX, Radius, Spacing, StrokeToken, Theme, TypeToken};

use crate::expr_modal::sync_from_store;
use crate::expr_modal_columns::expr_button;
use crate::ids;
use ph2d_timeline::TimelineViewSnapshot;

/// Gallery column width — fits `"▾ Physics  (4)"` and the longest recipe label.
pub(crate) const GALLERY_W: f32 = 190.0; // LITERAL-PX-OK: expression gallery column width
/// Sheet column width — a knob label column plus a slider track plus a readout.
pub(crate) const SHEET_W: f32 = 320.0; // LITERAL-PX-OK: expression sheet column width
/// Preview column width (plano 10 W2).
const PREVIEW_W: f32 = 190.0; // LITERAL-PX-OK: expression preview column width
/// Rows the puppet frame takes; the curve strip gets the rest of the body.
const PUPPET_SLOTS: f32 = 7.0; // LITERAL-PX-OK: CONTAGEM de linhas do quadro, nao medida
/// Knob label column inside a sheet row.
pub(crate) const KNOB_LABEL_W: f32 = 84.0; // LITERAL-PX-OK: expression knob label column
/// Numeric readout column at the right of a knob row.
pub(crate) const KNOB_READOUT_W: f32 = 52.0; // LITERAL-PX-OK: expression knob readout column
/// Close-X square in the title band.
const CLOSE_W: f32 = 20.0; // LITERAL-PX-OK: expression modal close-X width
/// Small square button (bypass eye / remove) in a row header.
pub(crate) const ROW_BTN_W: f32 = 22.0; // LITERAL-PX-OK: expression row button width
/// Apply / Cancel footer button width.
const FOOT_BTN_W: f32 = 84.0; // LITERAL-PX-OK: expression modal footer button width

/// How many `ROW_H_PX` rows the body is tall.
///
/// ⚠️ Derived, not chosen: the gallery needs one slot for the search field plus
/// the largest family (Shape, ten recipes) plus the row that walks back out —
/// twelve. The sheet then gets the same height for free, and its own capacity
/// (`1 + knobs` per row) falls out of it.
/// Half, for centring the card in the viewport.
const CENTRE: f32 = 0.5; // LITERAL-PX-OK: aritmetica de centralizacao, nao medida

pub(crate) const BODY_SLOTS: usize = 12;

/// The rows the card spends on ITSELF — the title band, the formula bar and the
/// footer — and therefore also the number of gaps between the four bands.
const CHROME_ROWS: f32 = 3.0; // LITERAL-PX-OK: CONTAGEM das tres linhas nomeadas, nao medida

/// Total card width.
pub fn card_w() -> f32 {
    Spacing::Md.px() * 2.0 + GALLERY_W + SHEET_W + PREVIEW_W + Spacing::Sm.px() * 2.0
}

/// Total card height: title · body · formula · footer, plus the paddings.
pub fn card_h() -> f32 {
    let gap = Spacing::Xs.px();
    Spacing::Sm.px() * 2.0 + ROW_H_PX * (BODY_SLOTS as f32 + CHROME_ROWS) + gap * CHROME_ROWS
}

pub(crate) fn button_state(store: &WidgetStore, id: ph2d_a11y::NodeId) -> ButtonState {
    match store.get(id) {
        Some(InteractiveState::Button { state }) => *state,
        _ => ButtonState::Normal,
    }
}

/// Paint the open modal (no-op when none is open). Called last in the panel paint
/// so it overlays the sheet, where the inline formula field used to.
pub(crate) fn paint(
    state: &mut crate::state::TimelinePanelState,
    ctx: &mut PaintCtx,
    theme: Theme,
    snap: &TimelineViewSnapshot,
) {
    let Some(m) = state.expr_modal.as_mut() else {
        return;
    };
    // The track may have vanished (deleted / undo). Abandon rather than author a
    // formula onto whatever slid into its place — the same guard `expr_edit` has.
    let Some(track) = snap.tracks.iter().find(|t| t.target.get() == m.target) else {
        state.expr_modal = None;
        return;
    };
    // First frame: seed the title and FREEZE the clock. The menu that opens the
    // card has no snapshot, so this is where those two come from.
    if !m.opened {
        // ⚠️ SEED from the formula the track already has, as a single `Custom
        // Formula` row. Without this the card opens EMPTY over an existing
        // expression and Apply erases it — the artist's own text, gone, by
        // pressing the button that says commit. As a row it is preserved, still
        // editable, and rows can be stacked around it.
        if let Some(src) = track.expr.as_ref().filter(|s| !s.trim().is_empty())
            && let Some(mut row) = ph2d_expr_recipes::Row::new("custom")
        {
            row.set("formula", ph2d_expr_recipes::KnobValue::Text(src.clone()));
            m.stack.push(row);
        }
        // ⚠️ The SAME label the track row shows (`tracks::prop_label`), so the card
        // and the row it was opened from never name the property differently.
        m.prop = track.prop;
        m.title = format!(
            "{}  #{}",
            crate::tracks::prop_label(track.prop),
            track.entity % 10_000
        );
        m.time = snap.time_seconds;
        m.opened = true;
    }
    // ONE read-back per frame, before anything is painted or projected, so the
    // formula bar and every readout describe the same instant.
    sync_from_store(m, ctx.host.store());
    let reseed = core::mem::take(&mut m.reseed);
    // ⚠️ ONE evaluation of the window per frame, shared by the strip and the
    // puppet — a puppet that sampled on its own would drift from the curve drawn
    // beside it.
    let samples = crate::expr_modal_preview::sample_window(&m.stack);
    m.preview_frame = m.preview_frame.wrapping_add(1);
    // ── Place the card, and KEEP it reachable. ──
    //
    // ⚠️ Both halves came out of the first smoke, which found the card pinned
    // half off the bottom of the screen with no way to move it:
    //
    // * it CENTRES on first paint rather than opening at the click, because the
    //   menu that opens it does not know how big the window is — and the click
    //   that opens it is, by construction, down in the timeline;
    // * and the top-left is CLAMPED every frame, so the whole card stays inside
    //   the viewport however it was dragged or however the window was resized.
    let (cw, ch) = (card_w(), card_h());
    let vp = ctx.viewport;
    let (px, py) = m
        .pos
        .unwrap_or((vp.x + (vp.w - cw) * CENTRE, vp.y + (vp.h - ch) * CENTRE));
    let max_x = (vp.x + vp.w - cw).max(vp.x);
    let max_y = (vp.y + vp.h - ch).max(vp.y);
    let rect = Rect::new(
        px.clamp(vp.x, max_x), // CLAMP-OK: bounds ordenados (max_x >= vp.x) e nao-NaN
        py.clamp(vp.y, max_y), // CLAMP-OK: bounds ordenados (max_y >= vp.y) e nao-NaN
        cw,
        ch,
    );
    m.pos = Some((rect.x, rect.y));
    let m = &*m;

    let radius = Radius::Md.px();
    fill_rounded_rect(ctx.scene, rect, radius, resolve(ColorToken::BgElev, theme));
    stroke_rounded_rect(
        ctx.scene,
        rect,
        radius,
        StrokeToken::Thin.px(),
        resolve(ColorToken::Border, theme),
    );

    let pad = Spacing::Md.px();
    let gap = Spacing::Xs.px();
    let font = TypeToken::Sm.px();
    let inner_x = rect.x + pad;
    let mut cy = rect.y + Spacing::Sm.px();

    // ── Title band = the drag handle, then the close X. ──
    //
    // ⚠️ The band is a GESTURE surface, not a button: it streams the panel's own
    // `TimelineGesture`s (the machinery the label splitter and the strip drags
    // already use), so the card moves with the pointer instead of merely
    // consuming a click. W1 shipped without it — a hit rect with no drag machine
    // behind it would have been worse than none — and the first smoke found the
    // card unreachable, which is what promoted this from polish to the fix.
    //
    // The band stops short of the X so the two never share a pixel: a Down on the
    // X closes, a Down on the band drags.
    let close_x = rect.x + rect.w - CLOSE_W - Spacing::Sm.px();
    let band = Rect::new(
        rect.x,
        rect.y,
        close_x - rect.x,
        ROW_H_PX + Spacing::Sm.px(),
    );
    ctx.host.store_mut().register(
        ids::EXPR_MODAL_HANDLE,
        InteractiveState::TimelineSurface {
            parent: ids::TIMELINE_PANEL,
            kind: TimelineHitKind::ExprModalHandle,
            canvas: band,
        },
    );
    ctx.host
        .hit_index_mut()
        .register(ids::EXPR_MODAL_HANDLE, band);
    paint_text(
        ctx.text_system,
        ctx.scene,
        &format!(
            "{} — {}",
            ph2d_i18n::tr("panel.timeline.expression"),
            m.title
        ),
        inner_x,
        cy + (ROW_H_PX - font) * 0.5,
        font,
        rect.w - pad * 2.0 - CLOSE_W,
        resolve(ColorToken::Text1, theme),
    );
    let close_rect = Rect::new(close_x, cy, CLOSE_W, ROW_H_PX);
    expr_button(ctx, theme, ids::EXPR_MODAL_CLOSE, "X", close_rect);
    cy += ROW_H_PX + gap;

    let body_y = cy;
    crate::expr_modal_columns::paint_gallery(m, ctx, theme, inner_x, body_y);
    let sheet_x = inner_x + GALLERY_W + Spacing::Sm.px();
    crate::expr_modal_columns::paint_sheet(m, ctx, theme, sheet_x, body_y, reseed);

    // ── The preview column (W2): the puppet above, the curve strip below. ──
    let pv_x = sheet_x + SHEET_W + Spacing::Sm.px();
    let puppet_h = ROW_H_PX * PUPPET_SLOTS;
    let strip_h = ROW_H_PX * (BODY_SLOTS as f32 - PUPPET_SLOTS) - gap;
    crate::expr_modal_preview::paint(
        ctx.scene,
        theme,
        Rect::new(pv_x, body_y, PREVIEW_W, puppet_h),
        Rect::new(pv_x, body_y + puppet_h + gap, PREVIEW_W, strip_h),
        &samples,
        crate::expr_modal_preview::puppet_for(m.prop),
        m.preview_frame,
    );
    cy += ROW_H_PX * BODY_SLOTS as f32 + gap;

    // ── The formula bar: the PROJECTION of the sheet, never a stored copy. ──
    let bar = Rect::new(inner_x, cy, rect.w - pad * 2.0, ROW_H_PX);
    stroke_rounded_rect(
        ctx.scene,
        bar,
        Radius::Xs.px(),
        StrokeToken::Thin.px(),
        resolve(ColorToken::Border, theme),
    );
    paint_text(
        ctx.text_system,
        ctx.scene,
        &format!("fx  {}", m.stack.to_formula()),
        bar.x + Spacing::Sm.px(),
        bar.y + (ROW_H_PX - font) * 0.5,
        font,
        bar.w - Spacing::Sm.px() * 2.0,
        resolve(ColorToken::Text2, theme),
    );
    cy += ROW_H_PX + gap;

    // ── Footer: Cancel · Apply. ──
    let apply_x = rect.x + rect.w - pad - FOOT_BTN_W;
    let cancel_x = apply_x - FOOT_BTN_W - Spacing::Sm.px();
    for (id, key, x) in [
        (
            ids::EXPR_MODAL_CANCEL,
            "panel.timeline.expr_cancel",
            cancel_x,
        ),
        (ids::EXPR_MODAL_APPLY, "panel.timeline.expr_apply", apply_x),
    ] {
        let r = Rect::new(x, cy, FOOT_BTN_W, ROW_H_PX);
        expr_button(ctx, theme, id, ph2d_i18n::tr(key), r);
    }
}
