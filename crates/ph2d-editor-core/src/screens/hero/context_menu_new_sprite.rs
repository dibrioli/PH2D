//! **O diálogo New Sprite** (Add ▸ Pixel Sprite, ADR-0192; o *File ▸ New* do Aseprite) — filho do
//! [`super`] pelo tecto de LOC e irmão do *New Image* na forma: caixa centrada, filas de escolha,
//! CTA acentuado. Width e Height são os campos numéricos da casa; os modos e os fundos saem das
//! tabelas `ids::NEW_SPRITE_MODES`/`NEW_SPRITE_BGS`, as mesmas que o despacho e o registo leem.

use super::super::context_menu_overlay::{ROW_H, pad_y};
use super::{DIALOG_W, paint_choice_button};
use crate::ids;
use crate::interaction::{HitIndex, WidgetStore};
use crate::paint::{fill_rounded_rect, paint_text, resolve};
use crate::panel::rows::RowCtx;
use crate::widget::{Button, paint_button};
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_i18n::tr;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Radius, Spacing, Theme, TypeToken};
use ph2d_vector::VectorScene;

/// Título · Width · Height · rótulo + fila do modo · rótulo + fila do fundo · Cancel | OK.
const ROWS: f32 = 8.0; // LITERAL-PX-OK: uma CONTAGEM de filas, não uma medida

pub(in crate::screens::hero) fn paint_new_sprite_dialog(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    viewport: Rect,
) {
    let gap = Spacing::Xs.px();
    let total_h = pad_y() * 2.0 + ROW_H * ROWS + gap * (ROWS - 1.0);
    let rect_x = (viewport.x + (viewport.w - DIALOG_W) * 0.5).max(viewport.x);
    let rect_y = (viewport.y + (viewport.h - total_h) * 0.5).max(viewport.y);
    let rect = Rect::new(rect_x, rect_y, DIALOG_W, total_h);
    let radius = crate::paint::frame_radius(theme, Radius::Md.px());
    fill_rounded_rect(scene, rect, radius, resolve(ColorToken::BgElev, theme));
    crate::paint::stroke_frame(
        scene,
        rect,
        radius,
        theme,
        ph2d_tokens::visuals::Feel::Rest,
        1.0,
        resolve(ColorToken::Border, theme),
    );
    let inner_x = rect.x + Spacing::Md.px();
    let inner_w = rect.w - Spacing::Md.px() * 2.0;
    let font = TypeToken::Sm.px();
    let label = |scene: &mut VectorScene, ts: &mut TextSystem, key: &str, y: f32, token| {
        paint_text(
            ts,
            scene,
            tr(key),
            inner_x,
            y + (ROW_H - font) * 0.5,
            font,
            inner_w,
            resolve(token, theme),
        );
    };
    let mut y = rect.y + pad_y();
    label(
        scene,
        text_system,
        "pixel.new_sprite.title",
        y,
        ColorToken::Text1,
    );
    y += ROW_H + gap;

    let mut r = RowCtx {
        scene: &mut *scene,
        text_system: &mut *text_system,
        store,
        hit_index: &mut *hit_index,
        theme,
        inner_x,
        inner_w,
        row_h: ROW_H,
        row_gap: gap,
        font,
        open_fold: None,
    };
    y = r.labeled_number_field_unit(
        tr("pixel.new_sprite.width"),
        ids::NEW_SPRITE_WIDTH,
        1.0,
        Some(crate::widget::Unit::Px),
        y,
    );
    y = r.labeled_number_field_unit(
        tr("pixel.new_sprite.height"),
        ids::NEW_SPRITE_HEIGHT,
        1.0,
        Some(crate::widget::Unit::Px),
        y,
    );

    let choice = store.new_sprite_choice();
    label(
        scene,
        text_system,
        "pixel.new_sprite.color_mode",
        y,
        ColorToken::Text3,
    );
    y += ROW_H + gap;
    let modes: Vec<(NodeId, &str, bool)> = ids::NEW_SPRITE_MODES
        .iter()
        .map(|(m, id, k)| (*id, tr(k), *m == choice.mode))
        .collect();
    y = choice_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        &modes,
        inner_x,
        inner_w,
        y,
    );
    label(
        scene,
        text_system,
        "pixel.new_sprite.background",
        y,
        ColorToken::Text3,
    );
    y += ROW_H + gap;
    let bgs: Vec<(NodeId, &str, bool)> = ids::NEW_SPRITE_BGS
        .iter()
        .map(|(b, id, k)| (*id, tr(k), *b == choice.background))
        .collect();
    y = choice_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        &bgs,
        inner_x,
        inner_w,
        y,
    );

    // Cancel | OK — o OK à direita e acentuado (Enter), o Cancel à esquerda (Esc).
    let half = ((inner_w - gap) * 0.5).max(1.0);
    for (id, key, x, accent) in [
        (
            ids::NEW_SPRITE_CANCEL,
            "pixel.new_sprite.cancel",
            inner_x,
            false,
        ),
        (
            ids::NEW_SPRITE_OK,
            "pixel.new_sprite.ok",
            inner_x + half + gap,
            true,
        ),
    ] {
        let b_rect = Rect::new(x, y, half, ROW_H);
        hit_index.register(id, b_rect);
        let b = Button::new(id, tr(key)).visual(store.button_visual(id));
        let b = if accent { b.accent() } else { b };
        paint_button(&b, b_rect, scene, text_system, theme);
    }
}

/// Uma fila de escolhas de largura igual (acentuada a escolhida).
#[allow(clippy::too_many_arguments)]
fn choice_row(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    opts: &[(NodeId, &str, bool)],
    x: f32,
    w: f32,
    y: f32,
) -> f32 {
    let gap = Spacing::Xs.px();
    #[expect(clippy::cast_precision_loss, reason = "três escolhas")]
    let n = opts.len() as f32;
    let bw = ((w - gap * (n - 1.0)) / n).max(1.0);
    for (i, (id, lbl, on)) in opts.iter().enumerate() {
        #[expect(clippy::cast_precision_loss, reason = "três escolhas")]
        let bx = x + i as f32 * (bw + gap);
        let r = Rect::new(bx, y, bw, ROW_H);
        paint_choice_button(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            *id,
            lbl,
            r,
            *on,
        );
    }
    y + ROW_H + gap
}
