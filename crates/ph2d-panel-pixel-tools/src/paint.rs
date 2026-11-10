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
use ph2d_editor_core::widget::card_ink::tinta;
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
#[path = "paint_cartoes.rs"]
mod cartoes;
#[path = "paint_cartoes_linhas.rs"]
mod cartoes_linhas;
#[path = "paint_fonte.rs"]
pub(crate) mod fonte;
#[path = "paint_formas.rs"]
mod formas;
#[path = "paint_hex.rs"]
mod hex_cor;
pub use hex_cor::hex;
#[path = "paint_paleta.rs"]
mod paleta;
#[path = "paint_paleta_menu.rs"]
pub(crate) mod paleta_menu;
#[path = "paint_pincel.rs"]
pub(crate) mod pincel;
#[path = "paint_pintura.rs"]
mod pintura;
#[path = "paint_rampas.rs"]
mod rampas;
#[path = "paint_selecao.rs"]
mod selecao;
#[path = "paint_sprite_props.rs"]
mod sprite_props;
#[path = "paint_paleta_voo.rs"]
mod voo;
pub(crate) use voo::Queda;

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
    let n = with_current(|i| if i.editing { i.palette.len() } else { 0 });
    seam::register_palette(ctx.host.store_mut(), n);
    let np = with_current(|i| if i.editing { i.presets.len() } else { 0 });
    seam::register_presets(ctx.host.store_mut(), np);
    crate::seam_seletor::arrasta_seletor(st, ctx.host);
    crate::seam_pincel::arrasta_limiar(st, ctx.host);
    let trancados: Vec<bool> = with_current(|i| i.brushes.iter().map(|b| b.locked).collect());
    crate::seam_pincel::register_brushes(ctx.host.store_mut(), &trancados);
    with_current(|i| crate::column::por_quadro(ctx.host.store_mut(), i));
    seam::seed_rename(st, ctx.host.store_mut());
    crate::seam_user::quadro(ctx.host);
    with_current(|i| crate::seam_rampas::quadro(ctx.host.store_mut(), i));
    let renaming = st.rename.map(|(id, _)| id);
    // Os campos mostram o MODELO (o instantâneo); a edição em curso de um campo com foco fica.
    let moldura = with_current(|i| i.transform);
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
        .chain(&crate::ids_pintura::NUMBERS)
        .map(|(id, _)| (*id, crate::seam_opcoes::numero(&opcoes, *id)))
        .chain(crate::seam_formas::numeros(&opcoes))
        .chain(crate::seam_pincel::numeros(&opcoes));
    let opacidade = (ids::LAYER_OPACITY, f64::from(opacidade));
    let campos = moldura.into_iter().flat_map(|t| {
        ids::XFORM_FIELDS
            .iter()
            .map(move |(f, id, _)| (*id, f64::from(t.get(*f))))
    });
    for (id, v) in numeros.chain([tamanho, opacidade]).chain(campos) {
        if store.number_value(id) != Some(v) {
            store.set_number_value(id, v);
        }
    }
    if ph2d_editor_core::panel::cards::ferramenta::cards_on() {
        cartoes::pinta(st, ctx, renaming);
        return;
    }
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
    let mut menu_aberto = None;
    let mut fonte_aberta = None;
    let seletor = &mut st.seletor;
    seletor.rects = [None; 3];
    let paleta_vista = &mut st.paleta;
    *paleta_vista = paleta_vista.do_quadro_novo();
    let pincel = &mut st.pincel;
    pincel.trilhas = [None; 2];
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
            cartao: None,
            blend_aberto: None,
            menu_aberto: None,
            fonte_aberta: None,
            seletor,
            paleta_vista,
            pincel,
        };
        let y = c.tudo(info, renaming, top);
        blend_aberto = c.blend_aberto;
        menu_aberto = c.menu_aberto;
        fonte_aberta = c.fonte_aberta;
        (y - top + PANEL_HEAD_PAD).max(0.0)
    });
    ph2d_editor_core::panel::scroll_area::close(area, ctx, content_h);
    // O bloco de amostras em voo, por cima do painel (fora do recorte da rolagem).
    if let Some(v) = &st.paleta.voo {
        voo::voa(ctx.scene, theme, v);
    }
    // O menu do modo, por cima do corpo (fora do recorte da rolagem).
    if let Some((chip, cur)) = blend_aberto {
        let dd =
            ph2d_editor_core::widget::Dropdown::new(ids::LAYER_BLEND, "", camadas::blend_options())
                .selected(cur)
                .open(true);
        let viewport = ctx.viewport;
        ph2d_editor_core::panel::paint_dropdown_list(ctx, &dd, chip, viewport, theme, None);
    }
    if let Some(chip) = fonte_aberta {
        fonte::popover(ctx, chip, theme);
    }
    // O menu da barra da paleta aberto, por cima de tudo.
    if let Some((chip, menu)) = menu_aberto {
        let dd = with_current(|info| paleta_menu::dropdown(menu, info));
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
    /// Dentro de um cartão: a tinta dele e o topo do corpo (a 1.ª linha não leva vão).
    cartao: Option<(ph2d_editor_core::widget::card_ink::CardInk, f32)>,
    /// O menu do modo aberto: o chip e o modo actual (pinta-se depois, por cima).
    blend_aberto: Option<(Rect, ph2d_editor_core::pixel_panel::PixelBlend)>,
    /// O menu da barra da paleta aberto: o ícone e qual (pinta-se depois, por cima).
    menu_aberto: Option<(Rect, paleta_menu::MenuPaleta)>,
    /// O dropdown de fontes do Text aberto: o chip (a lista pinta-se depois, por cima).
    fonte_aberta: Option<Rect>,
    seletor: &'a mut crate::state::Seletor,
    paleta_vista: &'a mut crate::state::PaletaVista,
    pincel: &'a mut crate::state::PincelVista,
}

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

impl Corpo<'_> {
    fn tudo(&mut self, info: &PixelPanelInfo, renaming: Option<u32>, y: f32) -> f32 {
        if !info.editing {
            return self.bloco(tr("panel.pixel.empty"), ColorToken::Text3, y);
        }
        let gap = ph2d_tokens::control_gap_px();
        let y = self.opcoes(info, y) + gap;
        let y = self.moldura(info, y) + gap;
        let y = self.cores(info, y) + gap;
        let y = self.camadas(info, renaming, y) + gap;
        let y = self.paleta(info, y) + gap;
        let y = self.rampas(info, y) + gap;
        let y = self.acoes_da_paleta(info, y) + gap;
        let y = self.seletor(info, y) + gap;
        let y = self.menus_selecao(info, y) + gap;
        self.sprite(info, y)
    }

    /// ⭐ **As opções da ferramenta na mão** (a barra de contexto do Aseprite): o pincel, o canto do
    /// retângulo, o spray, o balde, e para tudo o que pinta a simetria e o ladrilho.
    fn opcoes(&mut self, info: &PixelPanelInfo, y: f32) -> f32 {
        let (o, t) = (info.options, info.tool);
        let nomes = [
            tr("panel.pixel.tip"),
            tr("panel.pixel.size"),
            tr("panel.pixel.angle"),
            tr("panel.pixel.tolerance"),
            tr("panel.pixel.connectivity"),
            tr("panel.pixel.sample"),
            tr("panel.pixel.spray_width"),
            tr("panel.pixel.spray_speed"),
            tr(selecao::NOMES[0]),
            tr(selecao::NOMES[2]),
            tr("panel.pixel.pattern"),
            tr("panel.pixel.dynamics.stabilizer_len"),
            tr("panel.pixel.dynamics.size"),
            tr("panel.pixel.dynamics.angle"),
            tr("panel.pixel.dynamics.gradient"),
            tr("panel.pixel.dynamics.min_size"),
            tr("panel.pixel.dynamics.min_angle"),
            tr("panel.pixel.dynamics.dither"),
            tr("panel.pixel.gradient.shape"),
            tr("panel.pixel.gradient.dither"),
            tr("panel.pixel.text.font"),
            tr("panel.pixel.text.size"),
        ];
        let nomes: Vec<&str> = nomes.into_iter().chain(formas::nomes()).collect();
        let sec = Seccao::medida(self.text_system, 1, &nomes);
        let mut y = self.opcoes_selecao(info, sec, y);
        if t.uses_brush() {
            y = self.linha(tr("panel.pixel.brush"), ColorToken::Text3, y);
            let formas: Vec<(&str, bool, NodeId)> = ids::SHAPES
                .iter()
                .map(|&(s, id)| {
                    (
                        tr(s.label_key()),
                        s == o.shape && info.pattern.is_none(),
                        id,
                    )
                })
                .collect();
            y = self.escolha(tr("panel.pixel.tip"), &formas, sec, y);
            y = self.padrao(info, sec, y);
            if info.pattern.is_none() {
                y = self.campo(tr("panel.pixel.size"), ids::SIZE, sec, y);
            }
            if info.pattern.is_none() && o.shape != PixelBrushShape::Circle {
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
            y = self.guardados_da_mao(info, sec, y);
        }
        self.opcoes_de_baixo(info, sec, y, true)
    }

    /// Os pincéis guardados: a mão (dinâmica e corda) é do traço livre, só o Pencil e o Eraser.
    fn guardados_da_mao(&mut self, info: &PixelPanelInfo, sec: Seccao, y: f32) -> f32 {
        let livre = matches!(info.tool, PixelTool::Pencil | PixelTool::Eraser);
        self.guardados(info, livre, sec, y)
    }

    /// As opções abaixo dos pincéis guardados: formas, spray, balde, tinta e (`com_simetria`) a simetria.
    fn opcoes_de_baixo(
        &mut self,
        info: &PixelPanelInfo,
        sec: Seccao,
        y: f32,
        com_simetria: bool,
    ) -> f32 {
        let (o, t) = (info.options, info.tool);
        let mut y = self.opcoes_forma(info, sec, y);
        if t == PixelTool::Spray {
            y = self.dois_campos(
                (tr("panel.pixel.spray_width"), ids::SPRAY_WIDTH),
                (tr("panel.pixel.spray_speed"), ids::SPRAY_SPEED),
                sec,
                y,
            );
        }
        if t == PixelTool::Gradient {
            y = self.opcoes_gradiente(info, sec, y);
        }
        if t == PixelTool::Text {
            y = self.opcoes_texto(info, sec, y);
        }
        // Só o balde: o Gradient pinta a selecção (ou o cel) inteira, sem contiguidade.
        if t == PixelTool::Bucket {
            y = self.titulo(tr("panel.pixel.bucket"), y);
            let conn = [
                (tr("panel.pixel.connect4"), !o.eight, ids::CONNECT_4),
                (tr("panel.pixel.connect8"), o.eight, ids::CONNECT_8),
            ];
            y = self.campo_e_escolha(
                (tr("panel.pixel.tolerance"), ids::TOLERANCE),
                (tr("panel.pixel.connectivity"), &conn),
                sec,
                y,
            );
            y = self.caixas(
                &[(ids::CONTIGUOUS, tr("panel.pixel.contiguous"), o.contiguous)],
                sec,
                y,
            );
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
        // A borracha pinta sempre com a Simple: a tinta é das outras (o Gradient, o Blur e o
        // Jumble escrevem a cor pronta, sem tinta).
        if t.paints() && !crate::ids_pintura::sem_tinta(t) {
            let tintas: Vec<(&str, bool, NodeId)> = ids::INKS
                .iter()
                .map(|&(k, id)| (tr(k.label_key()), k == o.ink, id))
                .collect();
            y = self.escolha(tr("panel.pixel.ink"), &tintas, sec, y);
            if o.ink == PixelInk::Shading && info.selected.len() < 2 {
                y = self.bloco(tr("pixel.ink.shading.tooltip"), ColorToken::Text3, y);
            }
            y = self.opcoes_dither(info, sec, y);
        }
        y = self.opcoes_push(info, sec, y);
        if com_simetria && t.paints() {
            y = self.simetria(info, sec, y);
        }
        y
    }

    /// A simetria e o ladrilho: valem para tudo o que pinta.
    fn simetria(&mut self, info: &PixelPanelInfo, sec: Seccao, y: f32) -> f32 {
        let o = info.options;
        let mut y = self.titulo(tr("panel.pixel.symmetry"), y);
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
        self.caixas(
            &[
                (ids::TILE_X, tr("panel.pixel.tile_x"), o.tile_x),
                (ids::TILE_Y, tr("panel.pixel.tile_y"), o.tile_y),
            ],
            sec,
            y,
        )
    }

    fn campo(&mut self, nome: &str, id: NodeId, sec: Seccao, y: f32) -> f32 {
        if self.cartao.is_some() {
            return self.campos_no_cartao(&[(nome, id)], y);
        }
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
        if self.cartao.is_some() {
            return self.escolha_no_cartao(nome, seg, y);
        }
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
        if self.cartao.is_some() {
            return self.caixas_no_cartao(cx, sec, y);
        }
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
        if self.cartao.is_some() {
            return self.cores_no_cartao(info, y);
        }
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
                tinta(ColorToken::Text2, self.theme),
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
    fn sprite(&mut self, info: &PixelPanelInfo, y: f32) -> f32 {
        let sec = Seccao::medida(self.text_system, 1, &[tr("panel.pixel.replace_tolerance")]);
        let y = self.titulo(tr("panel.pixel.sprite"), y);
        let y = self.propriedades_do_sprite(info, y);
        let mut y = self.botoes(&[(ids::COLOR_MODE, "panel.pixel.color_mode")], y);
        y = self.menu_sprite(info, y);
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

    fn linha(&mut self, texto: &str, cor: ColorToken, y: f32) -> f32 {
        if self.cartao.is_some() {
            return self.legenda_no_cartao(texto, cor, y);
        }
        paint_text(
            self.text_system,
            self.scene,
            texto,
            self.x,
            y + (ROW_H_PX - TypeToken::Sm.px()) * 0.5,
            TypeToken::Sm.px(),
            self.w.max(1.0),
            tinta(cor, self.theme),
        );
        y + ROW_H_PX
    }

    /// Uma frase que QUEBRA (nunca cortada).
    fn bloco(&mut self, texto: &str, cor: ColorToken, y: f32) -> f32 {
        if let Some(mut c) = self.cartao_ctx() {
            return c.hint(texto, y);
        }
        let topo = y + (ROW_H_PX - TypeToken::Sm.px()) * 0.5;
        let alta = paint_text_block(
            self.text_system,
            self.scene,
            texto,
            self.x,
            topo,
            TypeToken::Sm.px(),
            self.w.max(1.0),
            tinta(cor, self.theme),
        );
        (topo + alta).max(y + ROW_H_PX)
    }
}
