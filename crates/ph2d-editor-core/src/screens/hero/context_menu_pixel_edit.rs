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
        // Título · canais · Preview · botões.
        PixelDialogKind::Invert => 4,
        // Título · W · H · Tiled · canais · Preview · botões.
        PixelDialogKind::Despeckle => 7,
        // Título · dentro/fora · matriz (2 filas) · fundo · Tiled · canais · Preview · botões.
        PixelDialogKind::Outline => 9,
        // Título · Brightness · Contrast · canais · Preview · botões.
        PixelDialogKind::BrightnessContrast => 6,
        // Título · modelo · Hue · Saturation · Lightness/Value · Alpha · canais · Preview · botões.
        PixelDialogKind::HueSaturation => 9,
        // Título · a caixa da curva · canais · Preview · botões.
        PixelDialogKind::ColorCurve => 4 + CURVE_ROWS,
        // Título · Tiled · canais · Preview · botões (+ as filas da lista: [`KERNEL_COLS`] por fila).
        PixelDialogKind::ConvolutionMatrix => 5,
    }
}

/// A lista de kernels da Convolution Matrix vai em colunas de botões de escolha, numa janela de
/// largura dupla: as 37 numa coluna só (≈ 1 000 px) passariam da altura de uma janela pequena, e em
/// três (13 filas) cabem sem rolagem com o nome mais longo inteiro. O Aseprite tem também «Apply» e
/// «Reload Stock» (relê o ficheiro de kernels) — não há ficheiro aqui.
const KERNEL_COLS: usize = 3;

/// A largura da janela de `kind`: a da Convolution Matrix é dupla (as colunas de kernels).
fn width_of(kind: PixelDialogKind) -> f32 {
    if kind == PixelDialogKind::ConvolutionMatrix {
        DIALOG_W + DIALOG_W
    } else {
        DIALOG_W
    }
}

/// A caixa da Color Curve ocupa estas filas.
const CURVE_ROWS: usize = 7;

pub(in crate::screens::hero) fn paint_pixel_edit_dialog(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    viewport: Rect,
) {
    let d = store.pixel_edit_dialog();
    let kernel_rows = if d.kind == PixelDialogKind::ConvolutionMatrix {
        d.filter.kernels.len().div_ceil(KERNEL_COLS)
    } else {
        0
    };
    #[expect(clippy::cast_precision_loss, reason = "uma contagem de filas, < 64")]
    let rows = (rows_of(d.kind) + kernel_rows) as f32;
    let gap = Spacing::Xs.px();
    let total_h = pad_y() * 2.0 + ROW_H * rows + ph2d_tokens::control_gap_px() * (rows - 1.0);
    let dialog_w = width_of(d.kind);
    // Os de filtro encostam-se onde tapam menos o sprite e nunca a dica do fundo (as filas dela).
    let alvo = d.kind.is_filter().then_some(d.avoid).flatten();
    #[expect(clippy::cast_precision_loss, reason = "px de ecrã")]
    let (view, avoid) = alvo.map_or((viewport, None), |((x, y, w, h), s)| {
        (Rect::new(x as f32, y as f32, w as f32, h as f32), Some(s))
    });
    let hint_band = ROW_H + ROW_H + Spacing::Md.px();
    let rect = crate::pixel_panel::place_dialog(
        view,
        (dialog_w, total_h),
        avoid,
        Spacing::Md.px(),
        hint_band,
    );
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
        PixelDialogKind::Invert => "pixel.filter_dialog.invert.title",
        PixelDialogKind::Despeckle => "pixel.filter_dialog.despeckle.title",
        PixelDialogKind::Outline => "pixel.filter_dialog.outline.title",
        PixelDialogKind::BrightnessContrast => "pixel.filter_dialog.brightness_contrast.title",
        PixelDialogKind::HueSaturation => "pixel.filter_dialog.hue_saturation.title",
        PixelDialogKind::ColorCurve => "pixel.filter_dialog.curve.title",
        PixelDialogKind::ConvolutionMatrix => "pixel.filter_dialog.convolution.title",
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
        k if k.is_filter() => {
            y = adjust_rows(&mut c, d, y);
            y = filter_rows(&mut c, d, y);
        }
        _ => {}
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

/// As filas de um diálogo de filtro (P8), depois do título.
fn filter_rows(c: &mut Caixa<'_>, d: crate::pixel_panel::PixelEditDialog, mut y: f32) -> f32 {
    let f = d.filter;
    if d.kind == PixelDialogKind::Outline {
        let opts: Vec<_> = ids::PIXEL_FILTER_PLACES
            .iter()
            .map(|(p, id, k)| (*id, tr(k), *p == f.place))
            .collect();
        y = c.choices(&opts, y);
        // A matriz em duas filas de duas: «Horizontal» não cabe num quarto da caixa.
        for par in ids::PIXEL_FILTER_MATRICES.chunks(2) {
            let opts: Vec<_> = par
                .iter()
                .map(|(m, id, k)| (*id, tr(k), *m == f.matrix))
                .collect();
            y = c.choices(&opts, y);
        }
        let opts: Vec<_> = ids::PIXEL_FILTER_BACKGROUNDS
            .iter()
            .map(|(b, id, k)| (*id, tr(k), *b == f.background))
            .collect();
        y = c.choices(&opts, y);
    }
    if d.kind == PixelDialogKind::Despeckle {
        y = c.field(
            "pixel.edit_dialog.width",
            ids::PIXEL_FILTER_MEDIAN_W,
            Some(Unit::Px),
            y,
        );
        y = c.field(
            "pixel.edit_dialog.height",
            ids::PIXEL_FILTER_MEDIAN_H,
            Some(Unit::Px),
            y,
        );
    }
    if d.kind == PixelDialogKind::ConvolutionMatrix {
        let sel = usize::from(f.kernel);
        for (fila, nomes) in f.kernels.chunks(KERNEL_COLS).enumerate() {
            let opts: Vec<_> = nomes
                .iter()
                .enumerate()
                .filter_map(|(j, nome)| {
                    let i = fila * KERNEL_COLS + j;
                    ids::pixel_filter_kernel_id(i).map(|id| (id, *nome, i == sel))
                })
                .collect();
            y = c.choices(&opts, y);
        }
    }
    if matches!(
        d.kind,
        PixelDialogKind::Despeckle | PixelDialogKind::Outline | PixelDialogKind::ConvolutionMatrix
    ) {
        let tiled = (
            ids::PIXEL_FILTER_TILED,
            tr("pixel.filter_dialog.tiled"),
            f.tiled,
        );
        y = c.choices(&[tiled], y);
    }
    let opts: Vec<_> = ids::PIXEL_FILTER_CHANNELS
        .iter()
        .map(|(k, id, key)| (*id, tr(key), f.channels[*k]))
        .collect();
    y = c.choices(&opts, y);
    let preview = (
        ids::PIXEL_FILTER_PREVIEW,
        tr("pixel.filter_dialog.preview"),
        f.preview,
    );
    // O alvo (o botão do Trial que alterna «Selected» → «All») vai na fila do Preview.
    let target = if f.all {
        "pixel.filter_dialog.target.all"
    } else {
        "pixel.filter_dialog.target.selected"
    };
    let target = (ids::PIXEL_FILTER_TARGET, tr(target), f.all);
    c.choices(&[preview, target], y)
}

/// As filas próprias de Adjustments (P8): os campos de B/C e H/S, o modelo do H/S e a curva.
fn adjust_rows(c: &mut Caixa<'_>, d: crate::pixel_panel::PixelEditDialog, mut y: f32) -> f32 {
    let f = d.filter;
    let campos: &[(&str, NodeId)] = match d.kind {
        PixelDialogKind::BrightnessContrast => &[
            (
                "pixel.filter_dialog.brightness",
                ids::PIXEL_FILTER_BRIGHTNESS,
            ),
            ("pixel.filter_dialog.contrast", ids::PIXEL_FILTER_CONTRAST),
        ],
        PixelDialogKind::HueSaturation => {
            let opts: Vec<_> = ids::PIXEL_FILTER_HUE_MODES
                .iter()
                .map(|(m, id, k)| (*id, tr(k), *m == f.hue_mode))
                .collect();
            y = c.choices(&opts, y);
            let luz = if f.hue_mode.is_hsv() {
                "pixel.filter_dialog.value"
            } else {
                "pixel.filter_dialog.lightness"
            };
            &[
                ("pixel.filter_dialog.hue", ids::PIXEL_FILTER_HUE),
                (
                    "pixel.filter_dialog.saturation",
                    ids::PIXEL_FILTER_SATURATION,
                ),
                (luz, ids::PIXEL_FILTER_LIGHTNESS),
                ("pixel.filter_dialog.alpha", ids::PIXEL_FILTER_ALPHA),
            ]
        }
        PixelDialogKind::ColorCurve => {
            #[expect(clippy::cast_precision_loss, reason = "uma contagem de filas, < 16")]
            let n = CURVE_ROWS as f32;
            let h = ROW_H * n + ph2d_tokens::control_gap_px() * (n - 1.0);
            let b = Rect::new(c.x, y, c.w, h);
            c.hit_index.register(ids::PIXEL_FILTER_CURVE, b);
            crate::widget::paint_curve_editor(b, f.curve.points(), f.curve_grab, c.scene, c.theme);
            return y + ph2d_tokens::row_pitch_px() * n;
        }
        _ => &[],
    };
    for (key, id) in campos {
        y = c.field(key, *id, None, y);
    }
    y
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
        y + ph2d_tokens::row_pitch_px()
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
            row_gap: ph2d_tokens::control_gap_px(),
            font: TypeToken::Sm.px(),
            open_fold: None,
        };
        r.labeled_number_field_unit(tr(key), id, 1.0, unit, y)
    }

    /// Uma fila de escolhas de largura igual: `(id, rótulo, escolhida)`.
    fn choices(&mut self, opts: &[(NodeId, &str, bool)], y: f32) -> f32 {
        let gap = Spacing::Xs.px();
        #[expect(clippy::cast_precision_loss, reason = "≤ 4 escolhas")]
        let n = opts.len() as f32;
        let bw = ((self.w - gap * (n - 1.0)) / n).max(1.0);
        for (i, (id, lbl, on)) in opts.iter().enumerate() {
            #[expect(clippy::cast_precision_loss, reason = "≤ 4 escolhas")]
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
        y + ph2d_tokens::row_pitch_px()
    }
}
