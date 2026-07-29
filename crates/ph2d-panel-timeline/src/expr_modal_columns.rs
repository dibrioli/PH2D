//! **The Expression modal's two body columns** — the gallery and the sheet.
//!
//! Split from `expr_modal_paint` by responsibility (the panel's 600-LOC cap made
//! the cut, the line of it is the meaning): that file owns what a CARD is — where
//! it sits, its title band, its formula bar, its footer — and this one owns what
//! goes INSIDE it. The card's geometry constants stay there and are shared, so
//! there is one answer to how wide a column is.

use ph2d_editor_core::interaction::InteractiveState;
use ph2d_editor_core::paint::{paint_text, resolve};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::widget::{
    Button, ButtonState, Slider, SliderOrientation, SliderState, TextInput, TextInputState,
    paint_button, paint_slider, paint_text_input_with_buffer,
};
use ph2d_editor_core::zones::Rect;
use ph2d_expr_recipes::{CATALOG, Family, KnobKind, SearchHit, search};
use ph2d_tokens::{ColorToken, ROW_H_PX, Spacing, Theme, TypeToken};

use crate::expr_modal::{ExprModal, GalleryPage, knob_track, row_result};
use crate::expr_modal_paint::{
    BODY_SLOTS, GALLERY_W, KNOB_LABEL_W, KNOB_READOUT_W, ROW_BTN_W, SHEET_W, button_state,
};
use crate::ids;

/// The left column: the search field, then either the families or one family's
/// recipes (plus any refusal cards a query surfaced).
pub(crate) fn paint_gallery(m: &ExprModal, ctx: &mut PaintCtx, theme: Theme, x: f32, y: f32) {
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
pub(crate) fn expr_button(
    ctx: &mut PaintCtx,
    theme: Theme,
    id: ph2d_a11y::NodeId,
    label: &str,
    rect: Rect,
) {
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
pub(crate) fn paint_sheet(
    m: &ExprModal,
    ctx: &mut PaintCtx,
    theme: Theme,
    x: f32,
    y: f32,
    reseed: bool,
) {
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
