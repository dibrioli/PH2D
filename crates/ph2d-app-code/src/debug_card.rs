//! O cartão do depurador na aba ([`crate::debug`]): a altura e a pintura — cabeçalho numa linha
//! (título · fila · botões), variáveis em grelha compacta na mono, a pilha.

use crate::commands::{CMD_CONTINUE, CMD_NEXT_LINE};
use crate::debug::{Ask, DebugCard};
use crate::doc::CodeDoc;
use ph2d_editor_core::paint::{fill_rounded_rect, paint_text, paint_text_title, resolve};
use ph2d_editor_core::widget::{Button, paint_button};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Radius, Spacing, Theme, TypeToken};
use ph2d_vector::VectorScene;

impl CodeDoc {
    /// A altura do cartão de baixo do código em `w` de largura (0 = nenhum).
    pub(crate) fn debug_card_h(&self, w: f32) -> f32 {
        self.debug.card.as_ref().map_or(0.0, |c| {
            let rows = 1
                + usize::from(c.error.is_some())
                + self.var_grid(c, w).rows
                + usize::from(c.stack.is_some());
            rows as f32 * ph2d_tokens::row_pitch_px() + Spacing::Sm.px() * 2.0
        })
    }

    /// A grelha das variáveis: colunas iguais na largura natural (o nome e o valor mais compridos,
    /// na mono do código), tantas quantas cabem em `w`, com o vão `Spacing::Xl` — compacta, à
    /// esquerda.
    fn var_grid(&self, c: &DebugCard, w: f32) -> VarGrid {
        let adv = self.view.metrics().advance;
        let chars = |f: fn(&(String, String)) -> &String| {
            c.vars
                .iter()
                .map(|v| f(v).chars().count())
                .max()
                .unwrap_or(0) as f32
                * adv
        };
        let inner = (w - Spacing::Sm.px() * 4.0).max(0.0);
        let name_w = chars(|v| &v.0);
        let col_w = (name_w + Spacing::Sm.px() + chars(|v| &v.1)).min(inner);
        let gap = Spacing::Xl.px();
        let cols = (((inner + gap) / (col_w + gap)).floor() as usize).clamp(1, c.vars.len().max(1));
        VarGrid {
            cols,
            rows: c.vars.len().div_ceil(cols),
            name_w,
            col_w,
            gap,
        }
    }

    /// Pinta o cartão em `r` (dentro do seu molde, [`crate::cards::open`]).
    pub(crate) fn paint_debug_card(
        &mut self,
        r: Rect,
        scene: &mut VectorScene,
        ts: &mut TextSystem,
        theme: &Theme,
    ) {
        self.debug.painted.clear();
        self.debug.card_rect = None;
        let Some(c) = self.debug.card.clone() else {
            return;
        };
        self.debug.card_rect = Some(r);
        let pad = Spacing::Sm.px();
        let rh = ph2d_tokens::row_pitch_px();
        let font = TypeToken::Sm.px();
        let ink = |t: ColorToken| resolve(t, *theme);
        let (x, w) = (r.x + pad * 2.0, (r.w - pad * 4.0).max(0.0));
        let right = x + w;
        let mut y = r.y + pad;
        let ty = |y: f32| y + (rh - font) * 0.5;
        if !c.stopped {
            // Os valores ao vivo: só o rótulo, no tom apagado dos valores.
            paint_text(
                ts,
                scene,
                &c.title,
                x,
                ty(y),
                font,
                w,
                ink(ColorToken::Text3),
            );
            return;
        }
        // O cabeçalho: título · chip da fila … [Continue] F8  [Next line] F10, à direita.
        let bh = rh - ph2d_tokens::control_gap_px();
        let mut buttons = vec![(
            Ask::Continue,
            CMD_CONTINUE,
            tr("code.debug.continue"),
            tr("code.debug.continue_key"),
        )];
        if c.can_step {
            buttons.push((
                Ask::NextLine,
                CMD_NEXT_LINE,
                tr("code.debug.next_line"),
                tr("code.debug.next_line_key"),
            ));
        }
        let gap = Spacing::Md.px();
        let sized: Vec<_> = buttons
            .into_iter()
            .map(|(ask, id, label, key)| {
                let bw = ts.layout(label, font, f32::INFINITY).width() + pad * 4.0;
                let kw = ts.layout(key, font, f32::INFINITY).width() + Spacing::Xs.px();
                (ask, id, label, key, bw, kw)
            })
            .collect();
        let total: f32 = sized.iter().map(|b| b.4 + pad + b.5 + gap).sum::<f32>() - gap;
        let mut bx = (right - total).max(x);
        let buttons_x = bx;
        let by = y + (rh - bh) * 0.5;
        for (i, (ask, id, label, key, bw, kw)) in sized.into_iter().enumerate() {
            let br = Rect::new(bx, by, bw, bh);
            let b = Button::new(id, label);
            let b = if i == 0 { b.accent() } else { b };
            paint_button(&b, br, scene, ts, *theme);
            self.debug.painted.push((ask, br));
            bx += bw + pad;
            paint_text(ts, scene, key, bx, ty(y), font, kw, ink(ColorToken::Text3));
            bx += kw + gap;
        }
        let room = (buttons_x - gap - x).max(0.0);
        let chip_w = c.waiting.as_ref().map_or(0.0, |s| {
            ts.layout(s, font, f32::INFINITY).width() + pad * 2.0
        });
        let title_room = (room - chip_w - gap).max(0.0);
        let title_w =
            (ts.layout(&c.title, font, f32::INFINITY).width() + Spacing::Xs.px()).min(title_room);
        let title_ink = if c.error.is_some() {
            ColorToken::Danger
        } else {
            ColorToken::Text1
        };
        paint_text_title(
            ts,
            scene,
            &c.title,
            x,
            ty(y),
            font,
            title_room,
            ink(title_ink),
        );
        if let Some(wait) = &c.waiting {
            let cr = Rect::new(x + title_w + gap, by, chip_w.min(room), bh);
            fill_rounded_rect(
                scene,
                cr,
                Radius::Full.px().min(bh * 0.5),
                ink(ColorToken::Bg3),
            );
            paint_text(
                ts,
                scene,
                wait,
                cr.x + pad,
                ty(y),
                font,
                cr.w,
                ink(ColorToken::Text2),
            );
        }
        y += rh;
        if let Some(e) = &c.error {
            paint_text(ts, scene, e, x, ty(y), font, w, ink(ColorToken::Danger));
            y += rh;
        }
        // As variáveis: nome apagado, valor na mono do código.
        let g = self.var_grid(&c, r.w);
        for (i, (name, value)) in c.vars.iter().enumerate() {
            let cx = x + (i % g.cols) as f32 * (g.col_w + g.gap);
            let cy = y + (i / g.cols) as f32 * rh;
            let cell_right = (cx + g.col_w).min(right);
            let name_ink = if value.is_empty() {
                ph2d_panel_code::Ink::Comment
            } else {
                ph2d_panel_code::Ink::Hint
            };
            self.view.paint_code_value(
                ts,
                scene,
                *theme,
                name,
                name_ink,
                (cx, cy, rh),
                (cx + g.name_w).min(cell_right),
            );
            if !value.is_empty() {
                let vx = cx + g.name_w + pad;
                self.view.paint_code_value(
                    ts,
                    scene,
                    *theme,
                    value,
                    ph2d_panel_code::Ink::Plain,
                    (vx, cy, rh),
                    cell_right,
                );
            }
        }
        y += g.rows as f32 * rh;
        if let Some(stack) = &c.stack {
            let label = tr("code.debug.stack");
            let lw = ts.layout(label, font, f32::INFINITY).width();
            paint_text(ts, scene, label, x, ty(y), font, w, ink(ColorToken::Text3));
            let vx = x + lw.max(g.name_w) + pad;
            paint_text(
                ts,
                scene,
                stack,
                vx,
                ty(y),
                font,
                (right - vx).max(0.0),
                ink(ColorToken::Text2),
            );
        }
    }
}

/// A grelha das variáveis do cartão parado.
struct VarGrid {
    cols: usize,
    rows: usize,
    /// A coluna dos nomes dentro de cada célula.
    name_w: f32,
    col_w: f32,
    gap: f32,
}
