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
use crate::paint::{paint_text_elided, resolve};
use crate::panel::cards::ferramenta::campo;
use crate::panel::cards::{CardCtx, Widget};
use crate::panel::rows::RowCtx;
use crate::pixel_panel::{PixelDialogKind, PixelModify};
use crate::widget::card_field::CardFieldSpec;
use crate::widget::card_ink::CardInk;
use crate::widget::{Button, Dropdown, DropdownOption, PaintedPopover, Unit, paint_button};
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_card_stack::metrics as m;
use ph2d_i18n::tr;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Spacing, Theme, TypeToken};
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
        // Título · Kernel (dropdown) · Tiled · canais · Preview · botões.
        PixelDialogKind::ConvolutionMatrix => 6,
    }
}

thread_local! {
    /// A lista dos kernels pintada neste quadro: o pintor só tem o store `&`; o
    /// [`publish_kernel_list`] publica-a no fim do quadro (rolagem, roda, clique de fora).
    static KERNEL_LIST: std::cell::Cell<Option<PaintedPopover>> =
        const { std::cell::Cell::new(None) };
}

/// Publica a lista dos kernels pintada neste quadro (se a houve) — chamado depois do overlay.
pub(in crate::screens::hero) fn publish_kernel_list(store: &mut WidgetStore) {
    if let Some(p) = KERNEL_LIST.with(std::cell::Cell::take) {
        store.publish_dropdown_popover(p);
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
    #[expect(clippy::cast_precision_loss, reason = "uma contagem de filas, < 64")]
    let rows = rows_of(d.kind) as f32;
    let total_h = pad_y() * 2.0 + ROW_H * rows + ph2d_tokens::control_gap_px() * (rows - 1.0);
    let dialog_w = DIALOG_W;
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
    let superficie = crate::widget::lista_flutuante::Superficie::de(theme);
    let _tinta = superficie.veste();
    let ink = superficie.tinta();
    // Em cartões a altura sai do corpo, num ensaio descartável: rótulos em cima, campos aos pares.
    let (pad_x, pad_topo) = ink.map_or((Spacing::Md.px(), pad_y()), |_| {
        (m::CARD_PAD_X, m::CARD_PAD_TOP)
    });
    let rect = if ink.is_some() {
        let (mut ensaio, mut alvos) = (VectorScene::new(), HitIndex::default());
        let mut c = Caixa {
            scene: &mut ensaio,
            text_system: &mut *text_system,
            theme,
            hit_index: &mut alvos,
            store,
            x: pad_x,
            w: dialog_w - pad_x * 2.0,
            kernel_chip: None,
            ink,
        };
        let h = corpo(&mut c, d, pad_topo) + m::CARD_PAD_BOTTOM;
        crate::pixel_panel::place_dialog(view, (dialog_w, h), avoid, Spacing::Md.px(), hint_band)
    } else {
        rect
    };
    superficie.moldura_de_caixa(scene, rect);
    let mut c = Caixa {
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x: rect.x + pad_x,
        w: rect.w - pad_x * 2.0,
        kernel_chip: None,
        ink,
    };
    corpo(&mut c, d, rect.y + pad_topo);
    // A lista aberta dos kernels por último: por cima dos botões, e os hits dela ganham.
    if let Some(chip) = c
        .kernel_chip
        .filter(|_| store.pixel_filter_kernel_list_open())
    {
        let dd = kernel_dropdown(d.filter, true);
        // A lista flutua sobre o canvas e a Timeline: fica dentro da janela com a moldura à vista.
        let region = dd.region_in_whole_rows(chip, viewport, Spacing::Md.px());
        let p = crate::panel::paint_dropdown_popover_scrolled_with_hits(
            &dd,
            chip,
            region,
            store,
            c.scene,
            c.text_system,
            theme,
            c.hit_index,
        );
        KERNEL_LIST.with(|k| k.set(Some(p)));
    }
}

/// Título, as filas do diálogo e Cancel | OK. Devolve o fim do conteúdo.
fn corpo(c: &mut Caixa<'_>, d: crate::pixel_panel::PixelEditDialog, mut y: f32) -> f32 {
    let gap = Spacing::Xs.px();
    let store = c.store;
    let theme = c.theme;
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
            let px = Some(Unit::Px);
            y = c.fields(
                &[
                    ("pixel.edit_dialog.width", ids::PIXEL_EDIT_CANVAS_W, px),
                    ("pixel.edit_dialog.height", ids::PIXEL_EDIT_CANVAS_H, px),
                    ("pixel.edit_dialog.left", ids::PIXEL_EDIT_LEFT, px),
                    ("pixel.edit_dialog.top", ids::PIXEL_EDIT_TOP, px),
                    ("pixel.edit_dialog.right", ids::PIXEL_EDIT_RIGHT, px),
                    ("pixel.edit_dialog.bottom", ids::PIXEL_EDIT_BOTTOM, px),
                ],
                y,
            );
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
            let (px, pct) = (Some(Unit::Px), Some(Unit::Percent));
            y = c.fields(
                &[
                    ("pixel.edit_dialog.width", ids::PIXEL_EDIT_SPRITE_W, px),
                    ("pixel.edit_dialog.height", ids::PIXEL_EDIT_SPRITE_H, px),
                    (
                        "pixel.edit_dialog.width_scale",
                        ids::PIXEL_EDIT_SPRITE_WPCT,
                        pct,
                    ),
                    (
                        "pixel.edit_dialog.height_scale",
                        ids::PIXEL_EDIT_SPRITE_HPCT,
                        pct,
                    ),
                ],
                y,
            );
            y = c.label("pixel.edit_dialog.interpolation", y, ColorToken::Text3);
            for (m, id, k) in ids::PIXEL_EDIT_METHODS {
                y = c.choices(&[(id, tr(k), m == d.method)], y);
            }
        }
        k if k.is_filter() => {
            y = adjust_rows(c, d, y);
            y = filter_rows(c, d, y);
        }
        _ => {}
    }
    if c.ink.is_some() {
        y += m::FIELD_ROW_GAP;
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
        let b_rect = Rect::new(x, y, half, c.h());
        c.hit_index.register(id, b_rect);
        let b = Button::new(id, tr(key)).visual(store.button_visual(id));
        let b = if accent { b.accent() } else { b };
        paint_button(&b, b_rect, c.scene, c.text_system, theme);
    }
    y + c.h()
}

/// O dropdown dos kernels da Convolution Matrix: uma opção por nome publicado, na ordem do núcleo.
/// O Aseprite tem também «Apply» e «Reload Stock» (relê o ficheiro de kernels) — não há ficheiro.
fn kernel_dropdown(f: crate::pixel_panel::PixelFilterDialog, open: bool) -> Dropdown<usize> {
    let opts = f
        .kernels
        .iter()
        .enumerate()
        .filter_map(|(i, nome)| {
            ids::pixel_filter_kernel_id(i).map(|id| DropdownOption::new(id, i, *nome))
        })
        .collect();
    Dropdown::new(ids::PIXEL_FILTER_KERNEL, "", opts)
        .selected(usize::from(f.kernel))
        .open(open)
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
        let px = Some(Unit::Px);
        y = c.fields(
            &[
                ("pixel.edit_dialog.width", ids::PIXEL_FILTER_MEDIAN_W, px),
                ("pixel.edit_dialog.height", ids::PIXEL_FILTER_MEDIAN_H, px),
            ],
            y,
        );
    }
    if d.kind == PixelDialogKind::ConvolutionMatrix {
        y = c.kernel_row(f, y);
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
    let linhas: Vec<_> = campos.iter().map(|(k, id)| (*k, *id, None)).collect();
    c.fields(&linhas, y)
}

struct Caixa<'a> {
    scene: &'a mut VectorScene,
    text_system: &'a mut TextSystem,
    theme: Theme,
    hit_index: &'a mut HitIndex,
    store: &'a WidgetStore,
    x: f32,
    w: f32,
    /// O chip do dropdown dos kernels, quando pintado (a lista aberta pinta-se no fim).
    kernel_chip: Option<Rect>,
    /// A tinta do cartão: as filas pintam-se em linhas de cartão. `None` no modo de antes.
    ink: Option<CardInk>,
}

impl Caixa<'_> {
    /// A altura de uma fila.
    fn h(&self) -> f32 {
        if self.ink.is_some() {
            m::FIELD_H
        } else {
            ROW_H
        }
    }

    /// Da fila à seguinte.
    fn pitch(&self) -> f32 {
        if self.ink.is_some() {
            m::FIELD_H
        } else {
            ph2d_tokens::row_pitch_px()
        }
    }

    /// Campos numéricos: em cartões, aos pares e com a unidade no rótulo.
    fn fields(&mut self, lista: &[(&str, NodeId, Option<Unit>)], mut y: f32) -> f32 {
        let Some(ink) = self.ink else {
            for (key, id, unit) in lista {
                y = self.field(key, *id, *unit, y);
            }
            return y;
        };
        for par in lista.chunks(2) {
            let nomes: Vec<String> = par
                .iter()
                .map(|(key, _, unit)| match unit {
                    Some(u) => format!("{} ({})", tr(key), u.suffix()),
                    None => tr(key).to_owned(),
                })
                .collect();
            let cells: Vec<_> = par
                .iter()
                .zip(&nomes)
                .map(|((_, id, _), nome)| {
                    let spec = CardFieldSpec {
                        step: 1.0,
                        scrub: 0.25, // LITERAL-PX-OK: unidades por px de arrasto, como no New Sprite
                        default: None,
                    };
                    self.store.mark_card_field_at_paint(*id, spec);
                    campo(*id, nome)
                })
                .collect();
            let mut c = CardCtx::flutuante(
                &mut *self.scene,
                &mut *self.text_system,
                self.store,
                &mut *self.hit_index,
                ink,
                self.x,
                self.w,
            );
            y = c.row(&cells, y);
        }
        y
    }

    /// Convolution Matrix: «Kernel» e o chip do dropdown no resto da fila. Não na coluna do valor:
    /// ela tem 138 px e o nome mais largo pede 184 (`no_kernel_name_nor_label_…_is_cut`); nenhuma
    /// outra fila deste diálogo é rotulada, não há coluna a alinhar.
    fn kernel_row(&mut self, f: crate::pixel_panel::PixelFilterDialog, y: f32) -> f32 {
        let font = TypeToken::Sm.px();
        let label = tr("pixel.filter_dialog.kernel");
        let label_w = crate::paint::label_column_width(self.text_system, font, [label]);
        crate::widget::paint_property_label(
            self.text_system,
            self.scene,
            label,
            self.x,
            y + (self.h() - font) * 0.5,
            font,
            label_w,
            crate::widget::card_ink::tinta(ColorToken::Text2, self.theme),
        );
        let gap = Spacing::Md.px();
        let chip = Rect::new(
            self.x + label_w + gap,
            y,
            (self.w - label_w - gap).max(1.0),
            self.h(),
        );
        let dd = kernel_dropdown(f, self.store.pixel_filter_kernel_list_open())
            .visual(self.store.dropdown_visual(ids::PIXEL_FILTER_KERNEL));
        crate::widget::paint_dropdown_chip(&dd, chip, self.scene, self.text_system, self.theme);
        self.hit_index.register(ids::PIXEL_FILTER_KERNEL, chip);
        self.kernel_chip = Some(chip);
        y + self.pitch()
    }

    /// Um rótulo numa fila só (elidido: o censo das elisões vê se coube).
    fn label(&mut self, key: &str, y: f32, token: ColorToken) -> f32 {
        if let Some(ink) = self.ink {
            // O título no corpo do valor; o rótulo no do cartão, colado à linha de baixo.
            let (font, h, cor) = if token == ColorToken::Text1 {
                (m::VALUE_PX, m::FIELD_H, ColorToken::KanbanCardFg)
            } else {
                (m::LABEL_PX, m::LABEL_PX, ColorToken::CardLabel)
            };
            let y = if token == ColorToken::Text1 {
                y
            } else {
                y + m::LABEL_TOP
            };
            paint_text_elided(
                self.text_system,
                self.scene,
                tr(key),
                self.x,
                y + crate::paint::folga_do_texto(h, font),
                font,
                self.w,
                ink.color(cor),
            );
            return y + h;
        }
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
        if let Some(ink) = self.ink {
            let segs = super::super::caixa_cartao::segs(opts);
            let mut c = CardCtx::flutuante(
                &mut *self.scene,
                &mut *self.text_system,
                self.store,
                &mut *self.hit_index,
                ink,
                self.x,
                self.w,
            );
            return c.full(None, Widget::Choice(&segs), y + m::FIELD_ROW_GAP);
        }
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
