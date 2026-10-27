//! **O diálogo Color Mode** do Pixel Art (Sprite ▸ Color Mode ▸ More Options do Aseprite 1.3.18) —
//! irmão do *New Sprite* na forma: o alvo (abre no modo actual, ESCOLHIDO e não apagado — o smoke
//! do dono 25/10: apagado confundia), para Indexed os oito dithers (os dois longos numa fila, depois
//! `Ordered` e `Old` como rótulo de fila com os três tamanhos — nenhum nome se corta), para Grayscale
//! os três métodos, Cancel | OK. As filas saem das tabelas `ids::COLOR_MODE_*`, as mesmas que o
//! despacho e o registo leem.

use super::super::context_menu_overlay::{ROW_H, pad_y};
use super::{DIALOG_W, paint_choice_button};
use crate::ids;
use crate::interaction::{HitIndex, WidgetStore};
use crate::paint::{fill_rounded_rect, paint_text, paint_text_elided, resolve};
use crate::pixel_panel::{NewSpriteMode, PixelDither};
use crate::widget::{Button, paint_button};
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_i18n::tr;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Radius, Spacing, Theme, TypeToken};
use ph2d_vector::VectorScene;

/// A grelha das filas rotuladas: o rótulo ocupa uma coluna, os três tamanhos as outras.
const COLUNAS: usize = 4;

pub(in crate::screens::hero) fn paint_color_mode_dialog(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    viewport: Rect,
) {
    let choice = store.color_mode_choice();
    let current = store.color_mode_current();
    let extra = match choice.target {
        NewSpriteMode::Indexed => 2 + ids::COLOR_MODE_DITHER_ROWS.len(),
        NewSpriteMode::Gray => 2,
        NewSpriteMode::Rgba => 0,
    };
    // Título · rótulo do alvo · fila do alvo · (as do alvo) · Cancel | OK.
    #[expect(clippy::cast_precision_loss, reason = "uma contagem de filas, < 16")]
    let rows = (4 + extra) as f32;
    let gap = Spacing::Xs.px();
    let total_h = pad_y() * 2.0 + ROW_H * rows + ph2d_tokens::control_gap_px() * (rows - 1.0);
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
        y + ph2d_tokens::row_pitch_px()
    };
    let mut f = Fila {
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x: inner_x,
        w: inner_w,
    };
    let mut y = rect.y + pad_y();
    y = label(
        f.scene,
        f.text_system,
        "pixel.color_mode.title",
        y,
        ColorToken::Text1,
    );
    y = label(
        f.scene,
        f.text_system,
        "pixel.color_mode.target",
        y,
        ColorToken::Text3,
    );
    let alvos: Vec<_> = ids::COLOR_MODE_TARGETS
        .iter()
        .map(|(m, id, k)| (*id, tr(k), *m == choice.target))
        .collect();
    debug_assert!(
        alvos.iter().any(|a| a.2),
        "{current:?}: um alvo está sempre escolhido"
    );
    y = f.fila(&alvos, y);
    match choice.target {
        NewSpriteMode::Indexed => {
            y = label(
                f.scene,
                f.text_system,
                "pixel.color_mode.dithering",
                y,
                ColorToken::Text3,
            );
            let opt = |d: PixelDither| {
                let (_, id, k) = ids::COLOR_MODE_DITHERS
                    .iter()
                    .find(|(x, ..)| *x == d)
                    .expect("todo dither está na tabela");
                (*id, tr(k), d == choice.dither)
            };
            let largos: Vec<_> = ids::COLOR_MODE_DITHER_WIDE.map(opt).to_vec();
            y = f.fila(&largos, y);
            for (rotulo, ds) in ids::COLOR_MODE_DITHER_ROWS {
                y = f.fila_rotulada(tr(rotulo), &ds.map(opt), y);
            }
        }
        NewSpriteMode::Gray => {
            y = label(
                f.scene,
                f.text_system,
                "pixel.color_mode.method",
                y,
                ColorToken::Text3,
            );
            let metodos: Vec<_> = ids::COLOR_MODE_GRAYS
                .iter()
                .map(|(g, id, k)| (*id, tr(k), *g == choice.gray))
                .collect();
            y = f.fila(&metodos, y);
        }
        NewSpriteMode::Rgba => {}
    }
    // Cancel | OK — o OK à direita e acentuado (Enter), o Cancel à esquerda (Esc).
    let half = ((inner_w - gap) * 0.5).max(1.0);
    for (id, key, x, accent) in [
        (
            ids::COLOR_MODE_CANCEL,
            "pixel.new_sprite.cancel",
            inner_x,
            false,
        ),
        (
            ids::COLOR_MODE_OK,
            "pixel.new_sprite.ok",
            inner_x + half + gap,
            true,
        ),
    ] {
        let b_rect = Rect::new(x, y, half, ROW_H);
        f.hit_index.register(id, b_rect);
        let b = Button::new(id, tr(key)).visual(store.button_visual(id));
        let b = if accent { b.accent() } else { b };
        paint_button(&b, b_rect, f.scene, f.text_system, theme);
    }
}

struct Fila<'a> {
    scene: &'a mut VectorScene,
    text_system: &'a mut TextSystem,
    theme: Theme,
    hit_index: &'a mut HitIndex,
    store: &'a WidgetStore,
    x: f32,
    w: f32,
}

impl Fila<'_> {
    /// Uma fila de escolhas de largura igual: `(id, rótulo, escolhida)`.
    fn fila(&mut self, opts: &[(NodeId, &str, bool)], y: f32) -> f32 {
        self.botoes(opts, self.x, self.w, y)
    }

    /// Uma fila com o RÓTULO dela na 1.ª coluna da grelha (como o de uma propriedade) e as
    /// escolhas nas outras.
    fn fila_rotulada(&mut self, rotulo: &str, opts: &[(NodeId, &str, bool)], y: f32) -> f32 {
        let gap = Spacing::Xs.px();
        #[expect(clippy::cast_precision_loss, reason = "uma contagem de colunas, < 16")]
        let col = ((self.w - gap * (COLUNAS - 1) as f32) / COLUNAS as f32).max(1.0);
        let font = TypeToken::Sm.px();
        paint_text_elided(
            self.text_system,
            self.scene,
            rotulo,
            self.x,
            y + (ROW_H - font) * 0.5,
            font,
            col,
            resolve(ColorToken::Text2, self.theme),
        );
        self.botoes(opts, self.x + col + gap, self.w - col - gap, y)
    }

    fn botoes(&mut self, opts: &[(NodeId, &str, bool)], x: f32, w: f32, y: f32) -> f32 {
        let gap = Spacing::Xs.px();
        #[expect(clippy::cast_precision_loss, reason = "≤ 4 escolhas")]
        let n = opts.len() as f32;
        let bw = ((w - gap * (n - 1.0)) / n).max(1.0);
        for (i, (id, lbl, on)) in opts.iter().enumerate() {
            #[expect(clippy::cast_precision_loss, reason = "≤ 4 escolhas")]
            let r = Rect::new(x + i as f32 * (bw + gap), y, bw, ROW_H);
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
