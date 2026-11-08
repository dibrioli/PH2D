//! ⭐ **O depurador na vista** (plano 13 Onda D): o ponto vermelho de cada ponto de parada na coluna
//! das marcas, a seta e a faixa da linha parada, e os valores ao vivo depois do fim de cada linha.

use crate::frame::CodeFrame;
use crate::geom;
use crate::palette::Palette;
use crate::view::CodeView;
use ph2d_editor_core::paint::{fill_circle, paint_text, rect_to_vello};
use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::{Spacing, TypeToken};
use ph2d_vector::{Affine, BezPath, Brush, VectorScene};

/// Um ponto de parada (linha 0-based). `armed = false`: não há código nessa linha (ponto oco —
/// o motor arma-o na próxima linha com código).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Breakpoint {
    pub line: usize,
    pub armed: bool,
}

/// O que uma linha fez no último quadro, já escrito (`x = 412, step = 1  ×3`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ghost {
    pub line: usize,
    pub text: String,
}

/// O estado do depurador que a pintura mostra.
#[derive(Clone, Copy, Debug, Default)]
pub struct DebugMarks<'a> {
    /// Por ordem de linha.
    pub breakpoints: &'a [Breakpoint],
    /// A linha onde o jogo está parado (0-based).
    pub stopped: Option<usize>,
    pub ghosts: &'a [Ghost],
}

impl CodeView {
    /// A faixa da linha parada, por baixo do texto.
    pub(crate) fn paint_stopped_band(
        &self,
        frame: &CodeFrame<'_>,
        scene: &mut VectorScene,
        pal: &Palette,
    ) {
        let Some(row) = frame.debug.stopped.and_then(|l| self.row_of(l)) else {
            return;
        };
        let tr = self.text_rect();
        scene.fill_rect(
            rect_to_vello(Rect {
                x: tr.x,
                y: row.y,
                w: tr.w,
                h: self.m.line_h,
            }),
            pal.match_current,
        );
    }

    /// Os valores ao vivo, apagados, depois do fim de cada linha (nunca por cima do código: uma
    /// linha dobrada ou cortada leva o «…» e não os mostra).
    pub(crate) fn paint_ghosts(
        &self,
        frame: &CodeFrame<'_>,
        scene: &mut VectorScene,
        ts: &mut TextSystem,
        pal: &Palette,
    ) {
        let tr = self.text_rect();
        let font = ph2d_text::displayed_font_px(TypeToken::Sm.px());
        for g in frame.debug.ghosts {
            let Some(row) = self.row_of(g.line) else {
                continue;
            };
            let folded = self.folds.iter().any(|f| f.folded && f.start == g.line);
            if row.cut || folded {
                continue;
            }
            let Some(layout) = self.cache.peek(row.key) else {
                continue;
            };
            let x = self.text_x0() + geom::width(layout) + Spacing::Lg.px();
            let room = tr.x + tr.w - x - Spacing::Sm.px();
            if room <= 0.0 {
                continue;
            }
            paint_text(
                ts,
                scene,
                &g.text,
                x,
                row.y + (self.m.line_h - font) * 0.5,
                font,
                room,
                pal.number,
            );
        }
    }

    /// Na coluna das marcas: o ponto de cada ponto de parada (oco se não armou) e a seta da linha
    /// parada por cima.
    pub(crate) fn paint_debug_gutter(
        &self,
        frame: &CodeFrame<'_>,
        scene: &mut VectorScene,
        pal: &Palette,
    ) {
        let a = self.area;
        let tr = self.text_rect();
        scene.push_clip(&rect_to_vello(Rect {
            x: a.x,
            y: tr.y,
            w: self.m.mark_w.min(a.w),
            h: tr.h,
        }));
        let cx = a.x + self.m.mark_w * 0.5;
        let r = (self.m.mark_w * 0.5 - Spacing::Xxs.px()).min(self.m.line_h * 0.5);
        let hole = r * 0.5;
        for b in frame.debug.breakpoints {
            let Some(row) = self.row_of(b.line) else {
                continue;
            };
            let cy = row.y + self.m.line_h * 0.5;
            fill_circle(scene, cx, cy, r, pal.error);
            if !b.armed {
                fill_circle(scene, cx, cy, hole, pal.gutter);
            }
        }
        if let Some(row) = frame.debug.stopped.and_then(|l| self.row_of(l)) {
            let cy = f64::from(row.y + self.m.line_h * 0.5);
            let (x0, x1) = (f64::from(a.x + Spacing::Xxs.px()), f64::from(cx + r));
            let h = f64::from(r);
            let mut p = BezPath::new();
            p.move_to((x0, cy - h));
            p.line_to((x1, cy));
            p.line_to((x0, cy + h));
            p.close_path();
            scene.fill_path(&p, &Brush::Solid(pal.warn), Affine::IDENTITY);
        }
        scene.pop_layer();
    }
}
