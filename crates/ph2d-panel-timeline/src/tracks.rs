//! Track list (W2.E4) — the left label column of the dope-sheet: the "+Track"
//! property buttons (header row, aligned with the ruler strip) and one row per
//! binding (property label + a short object tag).
//!
//! "+Track" is six per-property buttons (X/Y/R/Sx/Sy/Op). Clicking one raises a
//! `PanelEvent::Click(<prop id>)`; the shell binds the *currently selected*
//! sprite's matching property (it owns the selection). Each row's time area
//! paints its key **diamonds** (E5; selected keys in the accent colour), culled
//! to the visible span; click-select + drag land in E5b.

use ph2d_editor_core::paint::{fill_rounded_rect, paint_text, resolve};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::widget::{Button, ButtonState, paint_button};
use ph2d_editor_core::zones::Rect;
use ph2d_timeline::{PropKind, TimelineViewSnapshot};
use ph2d_tokens::{ColorToken, ROW_H_PX, Radius, Spacing, Theme, TypeToken};
use ph2d_vector::{Affine, BezPath, Brush, Fill};

use crate::ids;

/// Width of the left label column (property + object tag).
pub(crate) const LABEL_COL_W: f32 = 132.0; // LITERAL-PX-OK: track-label column width
const ADD_LABEL_W: f32 = 30.0; // LITERAL-PX-OK: "+Trk" caption column
const PROP_BTN_W: f32 = 22.0; // LITERAL-PX-OK: square per-property "+Track" button
const DIAMOND_H: f32 = 4.5; // LITERAL-PX-OK: keyframe diamond half-size

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

/// Paint one row per binding in `region`: the label (left `label_w` column) and
/// the key diamonds in the time area (mapped by `time_x`/`view_start`/`px_per_s`,
/// culled to the visible span). Selected keys paint in the accent colour.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_rows(
    ctx: &mut PaintCtx,
    theme: Theme,
    region: Rect,
    label_w: f32,
    time_x: f32,
    view_start: f64,
    px_per_s: f64,
    snap: &TimelineViewSnapshot,
) {
    let right = region.x + region.w;
    let mut y = region.y;
    for (i, track) in snap.tracks.iter().enumerate() {
        if y + ROW_H_PX > region.y + region.h {
            break;
        }
        // Zebra so lanes read as discrete rows.
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
        // Key diamonds.
        let cy = y + ROW_H_PX * 0.5;
        for k in &track.keys {
            let kx = time_x + ((k.t_seconds - view_start) * px_per_s) as f32;
            if kx < time_x - DIAMOND_H || kx > right + DIAMOND_H {
                continue;
            }
            let tok = if k.selected {
                ColorToken::Accent
            } else {
                ColorToken::Text1
            };
            paint_diamond(ctx, kx, cy, DIAMOND_H, resolve(tok, theme));
        }
        y += ROW_H_PX;
    }
}

/// Fill a keyframe diamond centred at `(cx, cy)` with half-size `h`.
fn paint_diamond(ctx: &mut PaintCtx, cx: f32, cy: f32, h: f32, color: ph2d_vector::Color) {
    let (cx, cy, h) = (cx as f64, cy as f64, h as f64);
    let mut p = BezPath::new();
    p.move_to((cx, cy - h));
    p.line_to((cx + h, cy));
    p.line_to((cx, cy + h));
    p.line_to((cx - h, cy));
    p.close_path();
    ctx.scene.inner_mut().fill(
        Fill::NonZero,
        Affine::IDENTITY,
        &Brush::Solid(color),
        None,
        &p,
    );
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
