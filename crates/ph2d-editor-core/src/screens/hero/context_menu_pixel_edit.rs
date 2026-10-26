//! **Os quatro diálogos do Pixel P6** (Select ▸ Color Range, Select ▸ Modify, Sprite ▸ Canvas
//! Size, Sprite ▸ Sprite Size do Aseprite) — irmãos do *Color Mode* na forma: caixa centrada de
//! `DIALOG_W`, título, as filas do `kind`, Cancel | OK. Os campos são os numéricos da casa
//! (`RowCtx::labeled_number_field_unit`); as escolhas saem das tabelas `ids::PIXEL_EDIT_*`, as
//! mesmas que o despacho e o registo leem. A interpolação vai uma por fila: «Nearest-neighbor» não
//! cabe num terço da caixa.

use super::super::context_menu_overlay::{ROW_H, pad_y};
use super::{DIALOG_W, paint_choice_button};
use crate::ids;
use crate::interaction::{HitIndex, WidgetStore};
use crate::paint::{fill_rounded_rect, paint_text_elided, resolve};
use crate::panel::rows::RowCtx;
use crate::pixel_panel::{PixelDialogKind, PixelModify};
use crate::widget::{Button, Unit, paint_button};
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_i18n::tr;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Radius, Spacing, Theme, TypeToken};
use ph2d_vector::VectorScene;

/// As filas de cada diálogo (título e Cancel | OK incluídos) — uma CONTAGEM.
const fn rows_of(kind: PixelDialogKind) -> usize {
    match kind {
        // Título · nota da cor · Tolerance · botões.
        PixelDialogKind::ColorRange => 4,
        // Título · n · rótulo do pincel · fila do pincel · botões.
        PixelDialogKind::Modify(_) => 5,
        // Título · W · H · 4 bordas · rótulo da âncora · 3 filas da grelha · botões.
        PixelDialogKind::CanvasSize => 12,
        // Título · Lock Ratio · W · H · W% · H% · rótulo · 3 métodos · botões.
        PixelDialogKind::SpriteSize => 11,
    }
}

pub(in crate::screens::hero) fn paint_pixel_edit_dialog(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    viewport: Rect,
) {
    let d = store.pixel_edit_dialog();
    #[expect(clippy::cast_precision_loss, reason = "uma contagem de filas, < 16")]
    let rows = rows_of(d.kind) as f32;
    let gap = Spacing::Xs.px();
    let total_h = pad_y() * 2.0 + ROW_H * rows + gap * (rows - 1.0);
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
    let mut c = Caixa {
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x: rect.x + Spacing::Md.px(),
        w: rect.w - Spacing::Md.px() * 2.0,
    };
    let mut y = rect.y + pad_y();
    let title = match d.kind {
        PixelDialogKind::ColorRange => "pixel.edit_dialog.color_range.title",
        PixelDialogKind::Modify(PixelModify::Expand) => "pixel.edit_dialog.modify.expand",
        PixelDialogKind::Modify(PixelModify::Contract) => "pixel.edit_dialog.modify.contract",
        PixelDialogKind::Modify(PixelModify::Border) => "pixel.edit_dialog.modify.border",
        PixelDialogKind::CanvasSize => "pixel.edit_dialog.canvas.title",
        PixelDialogKind::SpriteSize => "pixel.edit_dialog.sprite.title",
    };
    y = c.label(title, y, ColorToken::Text1);
    match d.kind {
        PixelDialogKind::ColorRange => {
            y = c.label("pixel.edit_dialog.color_range.note", y, ColorToken::Text3);
            y = c.field(
                "pixel.edit_dialog.tolerance",
                ids::PIXEL_EDIT_TOLERANCE,
                None,
                y,
            );
        }
        PixelDialogKind::Modify(m) => {
            let by = match m {
                PixelModify::Expand => "pixel.edit_dialog.modify.expand_by",
                PixelModify::Contract => "pixel.edit_dialog.modify.contract_by",
                PixelModify::Border => "pixel.edit_dialog.modify.border_width",
            };
            y = c.field(by, ids::PIXEL_EDIT_MODIFY_N, Some(Unit::Px), y);
            y = c.label("pixel.edit_dialog.brush", y, ColorToken::Text3);
            let opts: Vec<_> = ids::PIXEL_EDIT_BRUSHES
                .iter()
                .map(|(b, id, k)| (*id, tr(k), *b == d.brush))
                .collect();
            y = c.choices(&opts, y);
        }
        PixelDialogKind::CanvasSize => {
            for (key, id) in [
                ("pixel.edit_dialog.width", ids::PIXEL_EDIT_CANVAS_W),
                ("pixel.edit_dialog.height", ids::PIXEL_EDIT_CANVAS_H),
                ("pixel.edit_dialog.left", ids::PIXEL_EDIT_LEFT),
                ("pixel.edit_dialog.top", ids::PIXEL_EDIT_TOP),
                ("pixel.edit_dialog.right", ids::PIXEL_EDIT_RIGHT),
                ("pixel.edit_dialog.bottom", ids::PIXEL_EDIT_BOTTOM),
            ] {
                y = c.field(key, id, Some(Unit::Px), y);
            }
            y = c.label("pixel.edit_dialog.anchor", y, ColorToken::Text3);
            for row in ids::PIXEL_EDIT_ANCHORS.chunks(3) {
                let opts: Vec<_> = row
                    .iter()
                    .map(|(a, id, k)| (*id, tr(k), *a == d.anchor))
                    .collect();
                y = c.choices(&opts, y);
            }
        }
        PixelDialogKind::SpriteSize => {
            y = c.choices(
                &[(
                    ids::PIXEL_EDIT_LOCK,
                    tr("pixel.edit_dialog.lock_ratio"),
                    d.lock_ratio,
                )],
                y,
            );
            for (key, id, unit) in [
                (
                    "pixel.edit_dialog.width",
                    ids::PIXEL_EDIT_SPRITE_W,
                    Unit::Px,
                ),
                (
                    "pixel.edit_dialog.height",
                    ids::PIXEL_EDIT_SPRITE_H,
                    Unit::Px,
                ),
                (
                    "pixel.edit_dialog.width_scale",
                    ids::PIXEL_EDIT_SPRITE_WPCT,
                    Unit::Percent,
                ),
                (
                    "pixel.edit_dialog.height_scale",
                    ids::PIXEL_EDIT_SPRITE_HPCT,
                    Unit::Percent,
                ),
            ] {
                y = c.field(key, id, Some(unit), y);
            }
            y = c.label("pixel.edit_dialog.interpolation", y, ColorToken::Text3);
            for (m, id, k) in ids::PIXEL_EDIT_METHODS {
                y = c.choices(&[(id, tr(k), m == d.method)], y);
            }
        }
    }
    // Cancel | OK — o OK à direita e acentuado (Enter), o Cancel à esquerda (Esc).
    let half = ((c.w - gap) * 0.5).max(1.0);
    for (id, key, x, accent) in [
        (
            ids::PIXEL_EDIT_CANCEL,
            "pixel.new_sprite.cancel",
            c.x,
            false,
        ),
        (
            ids::PIXEL_EDIT_OK,
            "pixel.new_sprite.ok",
            c.x + half + gap,
            true,
        ),
    ] {
        let b_rect = Rect::new(x, y, half, ROW_H);
        c.hit_index.register(id, b_rect);
        let b = Button::new(id, tr(key)).visual(store.button_visual(id));
        let b = if accent { b.accent() } else { b };
        paint_button(&b, b_rect, c.scene, c.text_system, theme);
    }
}

struct Caixa<'a> {
    scene: &'a mut VectorScene,
    text_system: &'a mut TextSystem,
    theme: Theme,
    hit_index: &'a mut HitIndex,
    store: &'a WidgetStore,
    x: f32,
    w: f32,
}

impl Caixa<'_> {
    /// Um rótulo numa fila só (elidido: o censo das elisões vê se coube).
    fn label(&mut self, key: &str, y: f32, token: ColorToken) -> f32 {
        let font = TypeToken::Sm.px();
        paint_text_elided(
            self.text_system,
            self.scene,
            tr(key),
            self.x,
            y + (ROW_H - font) * 0.5,
            font,
            self.w,
            resolve(token, self.theme),
        );
        y + ROW_H + Spacing::Xs.px()
    }

    /// Um campo numérico rotulado, o da casa.
    fn field(&mut self, key: &str, id: NodeId, unit: Option<Unit>, y: f32) -> f32 {
        let mut r = RowCtx {
            scene: &mut *self.scene,
            text_system: &mut *self.text_system,
            store: self.store,
            hit_index: &mut *self.hit_index,
            theme: self.theme,
            inner_x: self.x,
            inner_w: self.w,
            row_h: ROW_H,
            row_gap: Spacing::Xs.px(),
            font: TypeToken::Sm.px(),
            open_fold: None,
        };
        r.labeled_number_field_unit(tr(key), id, 1.0, unit, y)
    }

    /// Uma fila de escolhas de largura igual: `(id, rótulo, escolhida)`.
    fn choices(&mut self, opts: &[(NodeId, &str, bool)], y: f32) -> f32 {
        let gap = Spacing::Xs.px();
        #[expect(clippy::cast_precision_loss, reason = "≤ 3 escolhas")]
        let n = opts.len() as f32;
        let bw = ((self.w - gap * (n - 1.0)) / n).max(1.0);
        for (i, (id, lbl, on)) in opts.iter().enumerate() {
            #[expect(clippy::cast_precision_loss, reason = "≤ 3 escolhas")]
            let r = Rect::new(self.x + i as f32 * (bw + gap), y, bw, ROW_H);
            paint_choice_button(
                self.scene,
                self.text_system,
                self.theme,
                self.hit_index,
                self.store,
                *id,
                lbl,
                r,
                *on,
            );
        }
        y + ROW_H + gap
    }
}
