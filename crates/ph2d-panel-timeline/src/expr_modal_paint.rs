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

use ph2d_editor_core::interaction::{InteractiveState, WidgetStore};
use ph2d_editor_core::paint::{fill_rounded_rect, paint_text, resolve, stroke_rounded_rect};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::widget::{
    Button, ButtonState, Slider, SliderOrientation, SliderState, TextInput, TextInputState,
    paint_button, paint_slider, paint_text_input_with_buffer,
};
use ph2d_editor_core::zones::Rect;
use ph2d_expr_recipes::{CATALOG, Family, KnobKind, SearchHit, search};
use ph2d_tokens::{ColorToken, ROW_H_PX, Radius, Spacing, StrokeToken, Theme, TypeToken};

use crate::expr_modal::{ExprModal, GalleryPage, knob_track, row_result, sync_from_store};
use crate::ids;
use ph2d_timeline::TimelineViewSnapshot;

/// Gallery column width — fits `"▾ Physics  (4)"` and the longest recipe label.
const GALLERY_W: f32 = 190.0; // LITERAL-PX-OK: expression gallery column width
/// Sheet column width — a knob label column plus a slider track plus a readout.
const SHEET_W: f32 = 320.0; // LITERAL-PX-OK: expression sheet column width
/// Knob label column inside a sheet row.
const KNOB_LABEL_W: f32 = 84.0; // LITERAL-PX-OK: expression knob label column
/// Numeric readout column at the right of a knob row.
const KNOB_READOUT_W: f32 = 52.0; // LITERAL-PX-OK: expression knob readout column
/// Close-X square in the title band.
const CLOSE_W: f32 = 20.0; // LITERAL-PX-OK: expression modal close-X width
/// Small square button (bypass eye / remove) in a row header.
const ROW_BTN_W: f32 = 22.0; // LITERAL-PX-OK: expression row button width
/// Apply / Cancel footer button width.
const FOOT_BTN_W: f32 = 84.0; // LITERAL-PX-OK: expression modal footer button width

/// How many `ROW_H_PX` rows the body is tall.
///
/// ⚠️ Derived, not chosen: the gallery needs one slot for the search field plus
/// the largest family (Shape, ten recipes) plus the row that walks back out —
/// twelve. The sheet then gets the same height for free, and its own capacity
/// (`1 + knobs` per row) falls out of it.
pub(crate) const BODY_SLOTS: usize = 12;

/// The rows the card spends on ITSELF — the title band, the formula bar and the
/// footer — and therefore also the number of gaps between the four bands.
const CHROME_ROWS: f32 = 3.0; // LITERAL-PX-OK: CONTAGEM das tres linhas nomeadas, nao medida

/// Total card width.
pub(crate) fn card_w() -> f32 {
    Spacing::Md.px() * 2.0 + GALLERY_W + Spacing::Sm.px() + SHEET_W
}

/// Total card height: title · body · formula · footer, plus the paddings.
pub(crate) fn card_h() -> f32 {
    let gap = Spacing::Xs.px();
    Spacing::Sm.px() * 2.0 + ROW_H_PX * (BODY_SLOTS as f32 + CHROME_ROWS) + gap * CHROME_ROWS
}

fn button_state(store: &WidgetStore, id: ph2d_a11y::NodeId) -> ButtonState {
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
    let m = &*m;

    let rect = Rect::new(m.x, m.y, card_w(), card_h());
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

    // ── Title band + close X. ──
    //
    // ⚠️ NO drag handle in W1. The card sits where the menu was clicked, like the
    // field it replaces. A hit rect for a gesture that does not exist is a rect
    // that swallows clicks and moves nothing — the wiring-parity gate caught the
    // one registered here before any drag machine existed to answer it.
    let close_x = rect.x + rect.w - CLOSE_W - Spacing::Sm.px();
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
    paint_gallery(m, ctx, theme, inner_x, body_y);
    paint_sheet(
        m,
        ctx,
        theme,
        inner_x + GALLERY_W + Spacing::Sm.px(),
        body_y,
        reseed,
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

/// The left column: the search field, then either the families or one family's
/// recipes (plus any refusal cards a query surfaced).
fn paint_gallery(m: &ExprModal, ctx: &mut PaintCtx, theme: Theme, x: f32, y: f32) {
    let font = TypeToken::Sm.px();
    let mut cy = y;

    // Search field — a TextInput seeded ONCE (re-seeding every frame would stomp
    // the artist's typing, the lesson `expr_edit` already paid for).
    let field = Rect::new(x, cy, GALLERY_W, ROW_H_PX);
    ctx.host.store_mut().register_if_absent(
        ids::EXPR_MODAL_SEARCH,
        InteractiveState::TextInput {
            state: TextInputState::Normal,
            text: String::new(),
            caret: 0,
            selection_anchor: None,
        },
    );
    let (ti_state, text, caret, anchor) = match ctx.host.store().get(ids::EXPR_MODAL_SEARCH) {
        Some(InteractiveState::TextInput {
            state,
            text,
            caret,
            selection_anchor,
        }) => (*state, text.clone(), *caret, *selection_anchor),
        _ => (TextInputState::Normal, String::new(), 0, None),
    };
    let input = TextInput::new(
        ids::EXPR_MODAL_SEARCH,
        ph2d_i18n::tr("panel.timeline.expr_search"),
    )
    .state(ti_state);
    paint_text_input_with_buffer(
        &input,
        Some(text.as_str()),
        Some(caret),
        anchor,
        field,
        ctx.scene,
        ctx.text_system,
        theme,
    );
    ctx.host
        .hit_index_mut()
        .register(ids::EXPR_MODAL_SEARCH, field);
    cy += ROW_H_PX;

    let slots = BODY_SLOTS - 1;
    let query = text.trim().to_string();

    if !query.is_empty() {
        // A query flattens the gallery: recipes first, then the refusal cards
        // that route to where a refused idea actually lives.
        let hits = search(&query);
        let shown = hits.len().min(slots);
        for h in hits.iter().take(shown) {
            match h {
                SearchHit::Recipe(r) => {
                    let id = ids::expr_gallery_id(r.id);
                    expr_button(
                        ctx,
                        theme,
                        id,
                        r.label,
                        Rect::new(x, cy, GALLERY_W, ROW_H_PX),
                    );
                }
                SearchHit::Refusal(rf) => {
                    let id = ids::expr_refusal_id(rf.key);
                    let label = format!("{} -> {}", rf.title, rf.to.label());
                    expr_button(
                        ctx,
                        theme,
                        id,
                        &label,
                        Rect::new(x, cy, GALLERY_W, ROW_H_PX),
                    );
                }
            }
            cy += ROW_H_PX;
        }
        if hits.len() > shown {
            // ⚠️ Named, never silent: a list that quietly stops reads as "there is
            // nothing else", which is the one thing it must not say.
            paint_text(
                ctx.text_system,
                ctx.scene,
                &format!("+{} more", hits.len() - shown),
                x + Spacing::Sm.px(),
                cy + (ROW_H_PX - font) * 0.5,
                font,
                GALLERY_W,
                resolve(ColorToken::Text2, theme),
            );
        }
        return;
    }

    match m.page {
        GalleryPage::Families => {
            for f in Family::ALL {
                let n = CATALOG.iter().filter(|r| r.family == f).count();
                let id = ids::expr_gallery_id(f.label());
                let label = format!("{}  ({n})", f.label());
                expr_button(
                    ctx,
                    theme,
                    id,
                    &label,
                    Rect::new(x, cy, GALLERY_W, ROW_H_PX),
                );
                cy += ROW_H_PX;
            }
        }
        GalleryPage::Family(f) => {
            let id = ids::expr_gallery_id("..");
            expr_button(
                ctx,
                theme,
                id,
                "< All",
                Rect::new(x, cy, GALLERY_W, ROW_H_PX),
            );
            cy += ROW_H_PX;
            for r in CATALOG.iter().filter(|r| r.family == f) {
                let id = ids::expr_gallery_id(r.id);
                expr_button(
                    ctx,
                    theme,
                    id,
                    r.label,
                    Rect::new(x, cy, GALLERY_W, ROW_H_PX),
                );
                cy += ROW_H_PX;
            }
        }
    }
}

/// Paint a button AND make it live under the mouse.
///
/// ⚠️ The `register_if_absent` is not decoration: a hit rect alone gets the
/// pointer to the right id, and the store entry is what makes the id FOCUSABLE —
/// without it the button is painted, hit-registered and dead under the mouse,
/// while a synthetic `WidgetEvent::Click` in a gate sails straight through. That
/// pair (green gate, dead button) is the exact failure the physics panel paid for
/// with 36 collision-matrix cells.
fn expr_button(ctx: &mut PaintCtx, theme: Theme, id: ph2d_a11y::NodeId, label: &str, rect: Rect) {
    ctx.host.hit_index_mut().register(id, rect);
    ctx.host.store_mut().register_if_absent(
        id,
        InteractiveState::Button {
            state: ButtonState::Normal,
        },
    );
    let b = Button::new(id, label).state(button_state(ctx.host.store(), id));
    paint_button(&b, rect, ctx.scene, ctx.text_system, theme);
}

/// The centre column: the stack, one row per recipe, each with its knobs and the
/// number it produces RIGHT NOW.
fn paint_sheet(m: &ExprModal, ctx: &mut PaintCtx, theme: Theme, x: f32, y: f32, reseed: bool) {
    let font = TypeToken::Sm.px();
    let mut cy = y;
    let mut used = 0usize;

    for (ri, row) in m.stack.rows.iter().enumerate() {
        let Some(rec) = ph2d_expr_recipes::by_id(row.recipe) else {
            continue;
        };
        let need = 1 + rec.knobs.len();
        if used + need > BODY_SLOTS {
            paint_text(
                ctx.text_system,
                ctx.scene,
                &format!("+{} more rows", m.stack.rows.len() - ri),
                x + Spacing::Sm.px(),
                cy + (ROW_H_PX - font) * 0.5,
                font,
                SHEET_W,
                resolve(ColorToken::Text2, theme),
            );
            return;
        }
        used += need;

        // Row header: bypass eye · label · result · remove.
        let eye = Rect::new(x, cy, ROW_BTN_W, ROW_H_PX);
        let eye_id = ids::expr_bypass_id(ri);
        expr_button(ctx, theme, eye_id, if row.bypass { "o" } else { "O" }, eye);

        let rm = Rect::new(x + SHEET_W - ROW_BTN_W, cy, ROW_BTN_W, ROW_H_PX);
        let rm_id = ids::expr_remove_id(ri);
        expr_button(ctx, theme, rm_id, "X", rm);

        // ⚠️ The result readout is the payload of the spreadsheet metaphor: in a
        // spreadsheet you never wonder what a formula IS, you see what it GIVES.
        let result = row_result(&m.stack, ri, m.time);
        paint_text(
            ctx.text_system,
            ctx.scene,
            rec.label,
            eye.x + ROW_BTN_W + Spacing::Xs.px(),
            cy + (ROW_H_PX - font) * 0.5,
            font,
            SHEET_W - ROW_BTN_W * 2.0 - KNOB_READOUT_W,
            resolve(
                if row.bypass {
                    ColorToken::Text2
                } else {
                    ColorToken::Text1
                },
                theme,
            ),
        );
        paint_text(
            ctx.text_system,
            ctx.scene,
            &result,
            x + SHEET_W - ROW_BTN_W - KNOB_READOUT_W,
            cy + (ROW_H_PX - font) * 0.5,
            font,
            KNOB_READOUT_W,
            resolve(ColorToken::Text2, theme),
        );
        cy += ROW_H_PX;

        // Knob rows.
        for (ki, k) in rec.knobs.iter().enumerate() {
            let lx = x + Spacing::Md.px();
            paint_text(
                ctx.text_system,
                ctx.scene,
                k.label,
                lx,
                cy + (ROW_H_PX - font) * 0.5,
                font,
                KNOB_LABEL_W,
                resolve(ColorToken::Text2, theme),
            );
            let ctrl_x = lx + KNOB_LABEL_W + Spacing::Xs.px();
            let ctrl_w = (x + SHEET_W - ctrl_x - KNOB_READOUT_W - Spacing::Xs.px()).max(1.0);
            let id = ids::expr_knob_id(ri, ki);
            match k.kind {
                KnobKind::Number | KnobKind::Literal => {
                    let track = knob_track(k, &row.knobs[ki]);
                    let seed = InteractiveState::Slider {
                        state: SliderState::Normal,
                        value: track,
                        orientation: SliderOrientation::Horizontal,
                    };
                    if reseed {
                        ctx.host.store_mut().register(id, seed);
                    } else {
                        ctx.host.store_mut().register_if_absent(id, seed);
                    }
                    let (st, v) = ctx
                        .host
                        .store()
                        .slider(id)
                        .unwrap_or((SliderState::Normal, track));
                    let r = Rect::new(ctrl_x, cy, ctrl_w, ROW_H_PX);
                    ctx.host.hit_index_mut().register(id, r);
                    let mut s = Slider::new(id, k.label).accent(true).state(st);
                    s.set_value(v);
                    paint_slider(&s, r, ctx.scene, theme);
                    paint_text(
                        ctx.text_system,
                        ctx.scene,
                        &ph2d_expr_recipes::fmt_num(row.knobs[ki].as_num()),
                        x + SHEET_W - KNOB_READOUT_W,
                        cy + (ROW_H_PX - font) * 0.5,
                        font,
                        KNOB_READOUT_W,
                        resolve(ColorToken::Text2, theme),
                    );
                }
                KnobKind::Link | KnobKind::Text => {
                    let seed = row.knobs[ki].as_text().to_string();
                    let caret = seed.len();
                    let init = InteractiveState::TextInput {
                        state: TextInputState::Normal,
                        text: seed,
                        caret,
                        selection_anchor: None,
                    };
                    if reseed {
                        ctx.host.store_mut().register(id, init);
                    } else {
                        ctx.host.store_mut().register_if_absent(id, init);
                    }
                    let (st, t, c, a) = match ctx.host.store().get(id) {
                        Some(InteractiveState::TextInput {
                            state,
                            text,
                            caret,
                            selection_anchor,
                        }) => (*state, text.clone(), *caret, *selection_anchor),
                        _ => (TextInputState::Normal, String::new(), 0, None),
                    };
                    let r = Rect::new(
                        ctrl_x,
                        cy,
                        ctrl_w + KNOB_READOUT_W - Spacing::Xs.px(),
                        ROW_H_PX,
                    );
                    ctx.host.hit_index_mut().register(id, r);
                    let ti = TextInput::new(id, k.label).state(st);
                    paint_text_input_with_buffer(
                        &ti,
                        Some(t.as_str()),
                        Some(c),
                        a,
                        r,
                        ctx.scene,
                        ctx.text_system,
                        theme,
                    );
                }
            }
            cy += ROW_H_PX;
        }
    }

    if m.stack.rows.is_empty() {
        paint_text(
            ctx.text_system,
            ctx.scene,
            ph2d_i18n::tr("panel.timeline.expr_empty"),
            x + Spacing::Sm.px(),
            cy + (ROW_H_PX - font) * 0.5,
            font,
            SHEET_W,
            resolve(ColorToken::Text2, theme),
        );
    }
}
