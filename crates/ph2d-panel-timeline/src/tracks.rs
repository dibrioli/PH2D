//! Track list (W2.E4) — the left label column of the dope-sheet: the "+Track"
//! property buttons (header row, aligned with the ruler strip) and one row per
//! binding (property label + a short object tag).
//!
//! "+Track" is six per-property buttons (X/Y/R/Sx/Sy/Op). Clicking one raises a
//! `PanelEvent::Click(<prop id>)`; the shell binds the *currently selected*
//! sprite's matching property (it owns the selection). The key lanes to the
//! right (diamonds, drag, select) land in E5; here the row's time area is empty
//! and the playhead line (painted by the ruler) crosses it.

use ph2d_editor_core::paint::{fill_rounded_rect, paint_text, resolve};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::widget::{Button, ButtonState, paint_button};
use ph2d_editor_core::zones::Rect;
use ph2d_timeline::{PropKind, TimelineViewSnapshot};
use ph2d_tokens::{ColorToken, ROW_H_PX, Radius, Spacing, Theme, TypeToken};

use crate::ids;

/// Width of the left label column (property + object tag).
pub(crate) const LABEL_COL_W: f32 = 132.0; // LITERAL-PX-OK: track-label column width
const ADD_LABEL_W: f32 = 30.0; // LITERAL-PX-OK: "+Trk" caption column
const PROP_BTN_W: f32 = 22.0; // LITERAL-PX-OK: square per-property "+Track" button

/// Paint the "+Track:" caption + the six per-property buttons in `header`
/// (the label-column slice aligned with the ruler strip).
pub(crate) fn paint_add_track(ctx: &mut PaintCtx, theme: Theme, header: Rect) {
    let font = TypeToken::Sm.px();
    paint_text(
        ctx.text_system,
        ctx.scene,
        "+Trk",
        header.x,
        header.y + (header.h - font) * 0.5,
        font,
        ADD_LABEL_W,
        resolve(ColorToken::Text2, theme),
    );
    let mut x = header.x + ADD_LABEL_W + Spacing::Xxs.px();
    for (id, label) in ids::ADDPROP_BUTTONS {
        let rect = Rect::new(x, header.y, PROP_BTN_W, header.h);
        let state = ctx
            .host
            .store()
            .button_state(id)
            .unwrap_or(ButtonState::Normal);
        let btn = Button::new(id, label).state(state);
        paint_button(&btn, rect, ctx.scene, ctx.text_system, theme);
        ctx.host.hit_index_mut().register(id, rect);
        x += PROP_BTN_W + Spacing::Xxs.px();
    }
}

/// Paint one row per binding in `region` (labels in the left `label_w` column;
/// the time area to its right hosts the lanes in E5). Returns the next `y`.
pub(crate) fn paint_rows(
    ctx: &mut PaintCtx,
    theme: Theme,
    region: Rect,
    label_w: f32,
    snap: &TimelineViewSnapshot,
) {
    let mut y = region.y;
    for (i, track) in snap.tracks.iter().enumerate() {
        if y + ROW_H_PX > region.y + region.h {
            break;
        }
        // Zebra + a divider under each row so lanes read as discrete.
        if i % 2 == 1 {
            fill_row(
                ctx,
                theme,
                Rect::new(region.x, y, region.w, ROW_H_PX),
                ColorToken::Bg2,
            );
        }
        let font = TypeToken::Sm.px();
        let text = format!("{}  #{}", prop_label(track.prop), track.entity % 10_000);
        let color = if track.missing {
            ColorToken::Danger
        } else {
            ColorToken::Text1
        };
        paint_text(
            ctx.text_system,
            ctx.scene,
            &text,
            region.x + Spacing::Xs.px(),
            y + (ROW_H_PX - font) * 0.5,
            font,
            label_w - Spacing::Xs.px() * 2.0,
            resolve(color, theme),
        );
        y += ROW_H_PX;
    }
}

/// The display label for a property (the panel's presentation of `PropKind`).
fn prop_label(p: PropKind) -> &'static str {
    match p {
        PropKind::TranslationX => "Translate X",
        PropKind::TranslationY => "Translate Y",
        PropKind::Rotation => "Rotation",
        PropKind::ScaleX => "Scale X",
        PropKind::ScaleY => "Scale Y",
        PropKind::Opacity => "Opacity",
    }
}

/// A subtle zebra fill for alternate rows.
fn fill_row(ctx: &mut PaintCtx, theme: Theme, rect: Rect, tok: ColorToken) {
    fill_rounded_rect(ctx.scene, rect, Radius::Xs.px(), resolve(tok, theme));
}
