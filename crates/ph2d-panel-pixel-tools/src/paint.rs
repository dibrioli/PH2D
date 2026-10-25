//! O quadro do painel PIXEL — porta de visibilidade (+ limpeza do rect) · rect do encaixe · moldura
//! · corpo dentro do recorte da rolagem (ferramentas, cores, paleta) · barra.

use crate::state::with_current;
use crate::{PixelToolsPanel, PixelToolsPanelState, ids, seam};
use ph2d_editor_core::ids as core_ids;
use ph2d_editor_core::interaction::{HitIndex, WidgetStore};
use ph2d_editor_core::paint::{paint_text, paint_text_block, resolve};
use ph2d_editor_core::panel::{PaintCtx, Panel};
use ph2d_editor_core::pixel_panel::PixelPanelInfo;
use ph2d_editor_core::widget::panel_chrome::{
    PANEL_HEAD_PAD, PANEL_HEADER_CLOSE_RESERVE, PANEL_TITLE_BASELINE, paint_panel_close_button,
    paint_panel_surface, paint_panel_title,
};
use ph2d_editor_core::widget::{
    Button, ButtonKind, ColorSwatch, PIXEL_SCROLLBAR_ID, SCROLLBAR_W, SwatchState, paint_button,
    paint_color_swatch,
};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, ROW_H_PX, Spacing, Theme, TypeToken};
use ph2d_vector::VectorScene;

pub(crate) fn paint(_st: &mut PixelToolsPanelState, ctx: &mut PaintCtx) {
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
        };
        let y = c.tudo(info, top);
        (y - top + PANEL_HEAD_PAD).max(0.0)
    });
    ph2d_editor_core::panel::scroll_area::close(area, ctx, content_h);
}

struct Corpo<'a> {
    scene: &'a mut VectorScene,
    text_system: &'a mut TextSystem,
    store: &'a WidgetStore,
    hit_index: &'a mut HitIndex,
    theme: Theme,
    x: f32,
    w: f32,
}

/// `#RRGGBB` (com `AA` quando a cor não é opaca) — o hex que o Aseprite mostra.
#[must_use]
pub fn hex(c: [u8; 4]) -> String {
    let [r, g, b, a] = c;
    if a == u8::MAX {
        format!("#{r:02X}{g:02X}{b:02X}")
    } else {
        format!("#{r:02X}{g:02X}{b:02X}{a:02X}")
    }
}

impl Corpo<'_> {
    fn tudo(&mut self, info: &PixelPanelInfo, y: f32) -> f32 {
        if !info.editing {
            return self.bloco(tr("panel.pixel.empty"), ColorToken::Text3, y);
        }
        let gap = ph2d_tokens::control_gap_px();
        let y = self.ferramentas(info, y) + gap;
        let y = self.cores(info, y) + gap;
        self.paleta(info, y)
    }

    /// As quatro ferramentas, uma por linha à largura toda (a barra vertical do Aseprite: o nome
    /// inteiro cabe em qualquer largura de coluna); a activa acentuada.
    fn ferramentas(&mut self, info: &PixelPanelInfo, y: f32) -> f32 {
        let mut y = y;
        for (t, id) in ids::TOOLS {
            let r = Rect::new(self.x, y, self.w, ROW_H_PX);
            let kind = if t == info.tool {
                ButtonKind::Accent
            } else {
                ButtonKind::Default
            };
            let b = Button::new(id, tr(t.label_key()))
                .kind(kind)
                .visual(self.store.button_visual(id));
            paint_button(&b, r, self.scene, self.text_system, self.theme);
            self.hit_index.register(id, r);
            y += ROW_H_PX + ph2d_tokens::control_gap_px();
        }
        y
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

    /// A grelha da paleta: esquerdo = frente, direito = fundo (o gémeo do botão direito).
    fn paleta(&mut self, info: &PixelPanelInfo, y: f32) -> f32 {
        let y = self.linha(tr("panel.pixel.palette"), ColorToken::Text3, y);
        let cell = Spacing::Xl.px();
        let gap = Spacing::Xxs.px();
        let cols = (((self.w + gap) / (cell + gap)).floor().max(1.0)) as usize;
        let mut bottom = y;
        for (i, c) in info.palette.iter().enumerate() {
            #[expect(clippy::cast_precision_loss, reason = "uma paleta, ≤ 256")]
            let (cx, cy) = ((i % cols) as f32, (i / cols) as f32);
            let r = Rect::new(
                self.x + cx * (cell + gap),
                y + cy * (cell + gap),
                cell,
                cell,
            );
            let id = ids::palette_id(i);
            // A cor de frente acende na paleta, como no Aseprite.
            let st = if *c == info.fg {
                SwatchState::Focused
            } else if self.store.hover_live(id) > 0.0 {
                SwatchState::Hovered
            } else {
                SwatchState::Normal
            };
            paint_color_swatch(
                &ColorSwatch::new(id, "", *c).state(st),
                r,
                self.scene,
                self.theme,
            );
            self.hit_index
                .register_with_secondary(id, r, ids::palette_bg_id(i));
            bottom = r.y + r.h;
        }
        bottom
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
