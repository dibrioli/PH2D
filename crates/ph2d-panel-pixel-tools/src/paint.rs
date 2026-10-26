//! O quadro do painel PIXEL — porta de visibilidade (+ limpeza do rect) · rect do encaixe · moldura
//! · corpo dentro do recorte da rolagem (cores, paleta — as ferramentas moram na coluna da
//! esquerda, [`crate::column`]) · barra.

use crate::state::with_current;
use crate::{PixelToolsPanel, PixelToolsPanelState, ids, seam};
use ph2d_a11y::NodeId;
use ph2d_editor_core::ids as core_ids;
use ph2d_editor_core::interaction::PanelRowFamily;
use ph2d_editor_core::interaction::{HitIndex, WidgetStore};
use ph2d_editor_core::paint::{
    paint_text, paint_text_block, resolve, stroke_rect, stroke_rounded_rect,
};
use ph2d_editor_core::panel::{PaintCtx, Panel};
use ph2d_editor_core::pixel_panel::{PixelBrushShape, PixelInk, PixelPanelInfo, PixelTool};
use ph2d_editor_core::property_row;
use ph2d_editor_core::widget::Seccao;
use ph2d_editor_core::widget::panel_chrome::{
    PANEL_HEAD_PAD, PANEL_HEADER_CLOSE_RESERVE, PANEL_TITLE_BASELINE, paint_panel_close_button,
    paint_panel_surface, paint_panel_title,
};
use ph2d_editor_core::widget::{
    Button, ColorSwatch, PIXEL_SCROLLBAR_ID, SCROLLBAR_W, SwatchState, paint_button,
    paint_color_swatch,
};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::{tr, tr_with};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, ROW_H_PX, Spacing, StrokeToken, Theme, TypeToken};
use ph2d_vector::{ImageQuality, VectorScene};
use std::sync::Arc;

/// O lado das imagens do seletor, em píxeis da imagem: 256 e amostragem suave (smoke do dono 25/10:
/// a 96 em vizinho mais próximo a borda da roda saía em escada).
const LADO: u32 = 256;

#[path = "paint_camadas.rs"]
mod camadas;
#[path = "paint_paleta.rs"]
mod paleta;

pub(crate) fn paint(st: &mut PixelToolsPanelState, ctx: &mut PaintCtx) {
    if !ctx.host.panel_visible(PixelToolsPanel::ID) {
        ctx.host.store_mut().clear_panel_rect(core_ids::PIXEL_PANEL);
        return;
    }
    let rect: Rect = ctx.slot;
    let theme = ctx.host.theme();
    ctx.host
        .store_mut()
        .set_panel_rect(core_ids::PIXEL_PANEL, rect);
    paint_panel_surface(rect, ctx.scene, theme);
    let title_size = paint_panel_title(
        rect,
        PixelToolsPanel::TITLE.tr(),
        PANEL_HEADER_CLOSE_RESERVE,
        ctx.scene,
        ctx.text_system,
        theme,
    );
    paint_panel_close_button(
        rect,
        ids::PIXEL_CLOSE,
        ctx.host.hit_index_mut(),
        ctx.scene,
        theme,
    );
    let n = with_current(|i| if i.editing { i.palette.len() } else { 0 });
    seam::register_palette(ctx.host.store_mut(), n);
    let np = with_current(|i| if i.editing { i.presets.len() } else { 0 });
    seam::register_presets(ctx.host.store_mut(), np);
    seam::arrasta_seletor(st, ctx.host);
    with_current(|i| seam::register_layers(ctx.host.store_mut(), &i.layers));
    seam::seed_rename(st, ctx.host.store_mut());
    let renaming = st.rename.map(|(id, _)| id);
    // Os campos mostram o MODELO (o instantâneo); a edição em curso de um campo com foco fica.
    let (opcoes, cores, opacidade) = with_current(|i| {
        let activa = i
            .active_layer
            .and_then(|a| i.layers.iter().find(|r| r.id == a));
        (
            i.options,
            i.palette.len(),
            activa.map_or(255, |r| r.opacity),
        )
    });
    let store = ctx.host.store_mut();
    #[expect(clippy::cast_precision_loss, reason = "uma paleta, ≤ 256")]
    let tamanho = (ids::PALETTE_SIZE, cores as f64);
    let numeros = ids::NUMBERS
        .iter()
        .map(|(id, _)| (*id, seam::numero(&opcoes, *id)));
    let opacidade = (ids::LAYER_OPACITY, f64::from(opacidade));
    for (id, v) in numeros.chain([tamanho, opacidade]) {
        if store.number_value(id) != Some(v) {
            store.set_number_value(id, v);
        }
    }

    let inner_x = rect.x + PANEL_HEAD_PAD;
    let scrollbar_reserve = SCROLLBAR_W + Spacing::Sm.px();
    let inner_w = (rect.w - PANEL_HEAD_PAD * 2.0 - scrollbar_reserve).max(0.0);
    let body_top = rect.y + PANEL_TITLE_BASELINE + title_size + Spacing::Md.px();
    let body_h = (rect.y + rect.h - body_top - PANEL_HEAD_PAD).max(0.0);
    let area = ph2d_editor_core::panel::scroll_area::open(
        ctx,
        core_ids::PIXEL_PANEL,
        PIXEL_SCROLLBAR_ID,
        Rect::new(rect.x, body_top, rect.w, body_h),
    );
    let top = area.top();
    let mut blend_aberto = None;
    let seletor = &mut st.seletor;
    seletor.rects = [None; 3];
    let content_h = with_current(|info| {
        let (store, hit_index) = ctx.host.store_and_hit_index_mut();
        let mut c = Corpo {
            scene: &mut *ctx.scene,
            text_system: &mut *ctx.text_system,
            store,
            hit_index,
            theme,
            x: inner_x,
            w: inner_w,
            blend_aberto: None,
            seletor,
        };
        let y = c.tudo(info, renaming, top);
        blend_aberto = c.blend_aberto;
        (y - top + PANEL_HEAD_PAD).max(0.0)
    });
    ph2d_editor_core::panel::scroll_area::close(area, ctx, content_h);
    // O menu do modo, por cima do corpo (fora do recorte da rolagem).
    if let Some((chip, cur)) = blend_aberto {
        let dd =
            ph2d_editor_core::widget::Dropdown::new(ids::LAYER_BLEND, "", camadas::blend_options())
                .selected(cur)
                .open(true);
        let viewport = ctx.viewport;
        ph2d_editor_core::panel::paint_dropdown_list(ctx, &dd, chip, viewport, theme, None);
    }
}

struct Corpo<'a> {
    scene: &'a mut VectorScene,
    text_system: &'a mut TextSystem,
    store: &'a WidgetStore,
    hit_index: &'a mut HitIndex,
    theme: Theme,
    x: f32,
    w: f32,
    /// O menu do modo aberto: o chip e o modo actual (pinta-se depois, por cima).
    blend_aberto: Option<(Rect, ph2d_editor_core::pixel_panel::PixelBlend)>,
    seletor: &'a mut crate::state::Seletor,
}

/// `#RRGGBB` (com `AA` quando a cor não é opaca) — o hex que o Aseprite mostra.
#[must_use]
/// Os botões de uma tabela de acções como (id, rótulo).
fn acoes<const N: usize>(
    t: [(
        NodeId,
        ph2d_editor_core::pixel_panel::PixelRequest,
        &'static str,
    ); N],
) -> [(NodeId, &'static str); N] {
    t.map(|(id, _, k)| (id, k))
}

pub fn hex(c: [u8; 4]) -> String {
    let [r, g, b, a] = c;
    if a == u8::MAX {
        format!("#{r:02X}{g:02X}{b:02X}")
    } else {
        format!("#{r:02X}{g:02X}{b:02X}{a:02X}")
    }
}

impl Corpo<'_> {
    fn tudo(&mut self, info: &PixelPanelInfo, renaming: Option<u32>, y: f32) -> f32 {
        if !info.editing {
            return self.bloco(tr("panel.pixel.empty"), ColorToken::Text3, y);
        }
        let gap = ph2d_tokens::control_gap_px();
        let y = self.opcoes(info, y) + gap;
        let y = self.cores(info, y) + gap;
        let y = self.camadas(info, renaming, y) + gap;
        let y = self.paleta(info, y) + gap;
        let y = self.seletor(info, y) + gap;
        let y = self.acoes_da_paleta(info, y) + gap;
        self.sprite(y)
    }

    /// ⭐ **As opções da ferramenta na mão** (a barra de contexto do Aseprite): o pincel, o canto do
    /// retângulo, o spray, o balde, e para tudo o que pinta a simetria e o ladrilho.
    fn opcoes(&mut self, info: &PixelPanelInfo, y: f32) -> f32 {
        let (o, t) = (info.options, info.tool);
        let nomes = [
            tr("panel.pixel.shape"),
            tr("panel.pixel.size"),
            tr("panel.pixel.angle"),
            tr("panel.pixel.corner"),
            tr("panel.pixel.tolerance"),
            tr("panel.pixel.connectivity"),
            tr("panel.pixel.sample"),
            tr("panel.pixel.spray_width"),
            tr("panel.pixel.spray_speed"),
        ];
        let sec = Seccao::medida(self.text_system, 1, &nomes);
        let mut y = y;
        if t.uses_brush() {
            y = self.linha(tr("panel.pixel.brush"), ColorToken::Text3, y);
            let formas: Vec<(&str, bool, NodeId)> = ids::SHAPES
                .iter()
                .map(|&(s, id)| (tr(s.label_key()), s == o.shape, id))
                .collect();
            y = self.escolha(tr("panel.pixel.shape"), &formas, sec, y);
            y = self.campo(tr("panel.pixel.size"), ids::SIZE, sec, y);
            if o.shape != PixelBrushShape::Circle {
                y = self.campo(tr("panel.pixel.angle"), ids::ANGLE, sec, y);
            }
            if matches!(
                t,
                PixelTool::Pencil | PixelTool::Eraser | PixelTool::Contour
            ) {
                y = self.caixas(
                    &[(
                        ids::PIXEL_PERFECT,
                        tr("panel.pixel.pixel_perfect"),
                        o.pixel_perfect,
                    )],
                    sec,
                    y,
                );
            }
        }
        if matches!(t, PixelTool::Rectangle | PixelTool::FilledRectangle) {
            y = self.campo(tr("panel.pixel.corner"), ids::CORNER, sec, y);
        }
        if t == PixelTool::Spray {
            y = self.campo(tr("panel.pixel.spray_width"), ids::SPRAY_WIDTH, sec, y);
            y = self.campo(tr("panel.pixel.spray_speed"), ids::SPRAY_SPEED, sec, y);
        }
        if t == PixelTool::Bucket {
            y = self.linha(tr("panel.pixel.bucket"), ColorToken::Text3, y);
            y = self.campo(tr("panel.pixel.tolerance"), ids::TOLERANCE, sec, y);
            y = self.caixas(
                &[(ids::CONTIGUOUS, tr("panel.pixel.contiguous"), o.contiguous)],
                sec,
                y,
            );
            let conn = [
                (tr("panel.pixel.connect4"), !o.eight, ids::CONNECT_4),
                (tr("panel.pixel.connect8"), o.eight, ids::CONNECT_8),
            ];
            y = self.escolha(tr("panel.pixel.connectivity"), &conn, sec, y);
            let amostra = [
                (
                    tr("panel.pixel.sample.layer"),
                    !o.sample_visible,
                    ids::SAMPLE_LAYER,
                ),
                (
                    tr("panel.pixel.sample.visible"),
                    o.sample_visible,
                    ids::SAMPLE_VISIBLE,
                ),
            ];
            y = self.escolha(tr("panel.pixel.sample"), &amostra, sec, y);
        }
        // A borracha pinta sempre com a Simple: a tinta é das outras.
        if t.paints() && t != PixelTool::Eraser {
            let tintas: Vec<(&str, bool, NodeId)> = ids::INKS
                .iter()
                .map(|&(k, id)| (tr(k.label_key()), k == o.ink, id))
                .collect();
            y = self.escolha(tr("panel.pixel.ink"), &tintas, sec, y);
            if o.ink == PixelInk::Shading && info.selected.len() < 2 {
                y = self.bloco(tr("panel.pixel.ink.shading_hint"), ColorToken::Text3, y);
            }
        }
        if t.paints() {
            y = self.linha(tr("panel.pixel.symmetry"), ColorToken::Text3, y);
            y = self.caixas(
                &[
                    (ids::SYM_H, tr("panel.pixel.sym_h"), o.sym_h),
                    (ids::SYM_V, tr("panel.pixel.sym_v"), o.sym_v),
                    (ids::SYM_D1, tr("panel.pixel.sym_d1"), o.sym_d1),
                    (ids::SYM_D2, tr("panel.pixel.sym_d2"), o.sym_d2),
                ],
                sec,
                y,
            );
            y = self.linha(tr("panel.pixel.tiled"), ColorToken::Text3, y);
            y = self.caixas(
                &[
                    (ids::TILE_X, tr("panel.pixel.tile_x"), o.tile_x),
                    (ids::TILE_Y, tr("panel.pixel.tile_y"), o.tile_y),
                ],
                sec,
                y,
            );
        }
        y
    }

    fn campo(&mut self, nome: &str, id: NodeId, sec: Seccao, y: f32) -> f32 {
        let (store, hit) = (self.store, &mut *self.hit_index);
        property_row::paint_field_row(
            self.scene,
            self.text_system,
            self.theme,
            hit,
            store,
            self.x,
            self.w,
            y,
            nome,
            id,
            1.0,
            None,
            sec,
        )
        .0
    }

    fn escolha(&mut self, nome: &str, seg: &[(&str, bool, NodeId)], sec: Seccao, y: f32) -> f32 {
        let (store, hit) = (self.store, &mut *self.hit_index);
        property_row::paint_choice_row(
            self.scene,
            self.text_system,
            self.theme,
            hit,
            store,
            self.x,
            self.w,
            y,
            nome,
            seg,
            sec,
        )
    }

    fn caixas(&mut self, cx: &[(NodeId, &str, bool)], sec: Seccao, y: f32) -> f32 {
        let (store, hit) = (self.store, &mut *self.hit_index);
        property_row::paint_check_rows(
            self.scene,
            self.text_system,
            self.theme,
            hit,
            store,
            self.x,
            self.w,
            y,
            cx,
            sec,
        )
    }

    /// Frente e fundo (amostra + hex, uma linha cada) e o Trocar.
    fn cores(&mut self, info: &PixelPanelInfo, y: f32) -> f32 {
        let mut y = y;
        for (sid, c) in [(ids::PIXEL_FG, info.fg), (ids::PIXEL_BG, info.bg)] {
            let side = ROW_H_PX;
            let r = Rect::new(self.x, y, side, side);
            // A amostra da casa pinta o xadrez quando alfa < 255.
            paint_color_swatch(&ColorSwatch::new(sid, "", c), r, self.scene, self.theme);
            let tx = self.x + side + Spacing::Sm.px();
            paint_text(
                self.text_system,
                self.scene,
                &hex(c),
                tx,
                y + (ROW_H_PX - TypeToken::Sm.px()) * 0.5,
                TypeToken::Sm.px(),
                (self.x + self.w - tx).max(1.0),
                resolve(ColorToken::Text2, self.theme),
            );
            y += side + ph2d_tokens::control_gap_px();
        }
        let r = Rect::new(self.x, y, self.w, ROW_H_PX);
        let b = Button::new(ids::PIXEL_SWAP, tr("panel.pixel.swap"))
            .visual(self.store.button_visual(ids::PIXEL_SWAP));
        paint_button(&b, r, self.scene, self.text_system, self.theme);
        self.hit_index.register(ids::PIXEL_SWAP, r);
        y + ROW_H_PX
    }

    /// O sprite: o *Color Mode…* e o *Replace Color* (frente → fundo, com tolerância).
    fn sprite(&mut self, y: f32) -> f32 {
        let sec = Seccao::medida(self.text_system, 1, &[tr("panel.pixel.replace_tolerance")]);
        let mut y = self.linha(tr("panel.pixel.sprite"), ColorToken::Text3, y);
        let r = Rect::new(self.x, y, self.w, ROW_H_PX);
        let b = Button::new(ids::COLOR_MODE, tr("panel.pixel.color_mode"))
            .visual(self.store.button_visual(ids::COLOR_MODE));
        paint_button(&b, r, self.scene, self.text_system, self.theme);
        self.hit_index.register(ids::COLOR_MODE, r);
        y += ROW_H_PX + ph2d_tokens::control_gap_px();
        y = self.linha(tr("panel.pixel.replace"), ColorToken::Text3, y);
        y = self.bloco(tr("panel.pixel.replace_hint"), ColorToken::Text3, y);
        y = self.campo(
            tr("panel.pixel.replace_tolerance"),
            ids::REPLACE_TOLERANCE,
            sec,
            y,
        );
        let [.., trocar] = ids::ACTIONS;
        self.botoes(&acoes([trocar]), y)
    }

    /// Uma fila de botões de largura igual — em par só quando os dois rótulos cabem INTEIROS na
    /// metade; senão um por linha (nenhum rótulo cortado no degrau estreito do dock).
    pub(super) fn botoes(&mut self, fila: &[(NodeId, &str)], y: f32) -> f32 {
        let gap = Spacing::Xs.px();
        #[expect(clippy::cast_precision_loss, reason = "≤ 2 botões")]
        let n = fila.len() as f32;
        let bw = ((self.w - gap * (n - 1.0)) / n).max(1.0);
        let fonte = Button::label_font_px();
        let cabe = |ts: &mut TextSystem, key: &str| {
            let pad = Button::new(NodeId(0), "").padding();
            ts.prefix_width(tr(key), fonte) + 2.0 * pad <= bw
        };
        if fila.len() > 1 && !fila.iter().all(|(_, k)| cabe(self.text_system, k)) {
            return fila
                .iter()
                .fold(y, |y, b| self.botoes(std::slice::from_ref(b), y));
        }
        for (i, (id, key)) in fila.iter().enumerate() {
            #[expect(clippy::cast_precision_loss, reason = "≤ 2 botões")]
            let r = Rect::new(self.x + i as f32 * (bw + gap), y, bw, ROW_H_PX);
            let b = Button::new(*id, tr(key)).visual(self.store.button_visual(*id));
            paint_button(&b, r, self.scene, self.text_system, self.theme);
            self.hit_index.register(*id, r);
        }
        y + ROW_H_PX + ph2d_tokens::control_gap_px()
    }

    fn linha(&mut self, texto: &str, cor: ColorToken, y: f32) -> f32 {
        paint_text(
            self.text_system,
            self.scene,
            texto,
            self.x,
            y + (ROW_H_PX - TypeToken::Sm.px()) * 0.5,
            TypeToken::Sm.px(),
            self.w.max(1.0),
            resolve(cor, self.theme),
        );
        y + ROW_H_PX
    }

    /// Uma frase que QUEBRA (nunca cortada).
    fn bloco(&mut self, texto: &str, cor: ColorToken, y: f32) -> f32 {
        let topo = y + (ROW_H_PX - TypeToken::Sm.px()) * 0.5;
        let alta = paint_text_block(
            self.text_system,
            self.scene,
            texto,
            self.x,
            topo,
            TypeToken::Sm.px(),
            self.w.max(1.0),
            resolve(cor, self.theme),
        );
        (topo + alta).max(y + ROW_H_PX)
    }
}
