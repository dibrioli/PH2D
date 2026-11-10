//! ⭐ **O depurador na vista** (plano 13 Onda D): o ponto vermelho de cada ponto de parada na coluna
//! das marcas, a seta e a faixa da linha parada, e os valores ao vivo depois do fim de cada linha.

use crate::frame::CodeFrame;
use crate::geom;
use crate::palette::Palette;
use crate::view::CodeView;
use ph2d_editor_core::paint::{fill_circle, frame_radius, paint_text, rect_to_vello};
use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::{Radius, Spacing, StrokeToken, TypeToken};
use ph2d_vector::{Affine, BezPath, Brush, VectorScene};

/// Um ponto de parada (linha 0-based). `armed = false`: não há código nessa linha (ponto oco —
/// o motor arma-o na próxima linha com código).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Breakpoint {
    pub line: usize,
    pub armed: bool,
}

/// 2,5 amostras por segundo — o ritmo do mostrador de um multímetro, abaixo dos 3 Hz da WCAG 2.3.1.
/// Antes era a cada quadro (60 Hz). Sem nada mostrado, a 1.ª amostra é já (sem 400 ms de atraso).
#[must_use]
pub fn ghosts_due(sampled: Option<u128>, now_ns: u128) -> bool {
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let period = (f64::from(ph2d_tokens::Duration::Slow.ms()) * 1e6) as u128;
    sampled.is_none_or(|t| now_ns.saturating_sub(t) >= period)
}

/// As linhas que um toque da roda (`dy` px, linhas de `line_h`) anda numa lista: uma no mínimo (o
/// trackpad manda fracções), para cima com `dy` positivo.
#[must_use]
pub fn wheel_rows(dy: f32, line_h: f32) -> isize {
    let lines = -dy / line_h.max(1.0);
    if lines.abs() < 1.0 {
        lines.signum() as isize
    } else {
        lines.round() as isize
    }
}

/// O pior problema da linha 0-based `line` (`problems` vem ordenado por linha).
pub(crate) fn worst_problem(
    problems: &[crate::frame::Problem],
    line: usize,
) -> Option<crate::frame::Severity> {
    let from = problems.partition_point(|p| p.line < line);
    problems[from..]
        .iter()
        .take_while(|p| p.line == line)
        .map(|p| p.severity)
        .max()
}

/// O que uma linha fez no último quadro, já escrito (`x = 412, step = 1  ×3`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ghost {
    pub line: usize,
    pub text: String,
    /// O maior número de caracteres que esta linha já mostrou: a moldura não encolhe enquanto corre.
    pub min_chars: usize,
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
    /// A faixa da linha parada, por baixo do texto: tinta âmbar com o fio cheio à esquerda.
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
        let band = Rect {
            x: tr.x,
            y: row.y,
            w: tr.w,
            h: self.m.line_h,
        };
        scene.fill_rect(rect_to_vello(band), pal.stopped_band);
        scene.fill_rect(
            rect_to_vello(Rect {
                w: StrokeToken::Thick.px(),
                ..band
            }),
            pal.warn,
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
            // A moldura à volta do valor, da largura do texto mais comprido já visto (os dígitos que
            // faltam contam como «0»): o valor muda e ela não mexe.
            let short = g.min_chars.saturating_sub(g.text.chars().count());
            let widest = format!("{}{}", g.text, "0".repeat(short));
            let shaped = ts.layout(&widest, font, f32::INFINITY);
            let w = shaped.width().min(room);
            // Na linha de base do código ao lado (centrar a caixa da letra descia-o ~2 px; foto 09/11).
            let base = shaped.lines().next().map_or(font, |l| l.metrics().baseline);
            let ty = row.y + self.m.text_dy + self.m.baseline - base;
            let (pad, inset) = (Spacing::Sm.px(), Spacing::Xxs.px());
            let frame = Rect {
                x: x - pad,
                y: row.y + inset,
                w: w + pad * 2.0,
                h: (self.m.line_h - inset * 2.0).max(0.0),
            };
            let radius = frame_radius(pal.theme, Radius::Sm.px());
            ph2d_editor_core::paint::stroke_frame(
                scene,
                frame,
                radius,
                pal.theme,
                ph2d_tokens::visuals::Feel::Rest,
                StrokeToken::Thin.px(),
                pal.border,
            );
            paint_text(ts, scene, &g.text, x, ty, font, room, pal.number);
        }
    }

    /// Na coluna das marcas: o ponto vermelho de cada ponto de parada (anel se não armou) e a seta
    /// âmbar da linha parada — por cima do ponto, menor e contornada, para os dois se lerem.
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
        let r = self.dot_r();
        let ring = StrokeToken::Thick.px();
        let mut dot_on = false;
        for b in frame.debug.breakpoints {
            let Some(row) = self.row_of(b.line) else {
                continue;
            };
            let cy = row.y + self.m.line_h * 0.5;
            fill_circle(scene, cx, cy, r, pal.error);
            if !b.armed {
                fill_circle(scene, cx, cy, r - ring, pal.gutter);
            }
            // O problema da mesma linha: um selo no canto do ponto, contornado do fundo.
            if let Some(sev) = worst_problem(frame.problems, b.line) {
                let d = Spacing::Sm.px() * 0.5;
                let (bx, by) = (cx + r - d, cy + r - d);
                fill_circle(scene, bx, by, d + ring, pal.gutter);
                fill_circle(scene, bx, by, d, pal.severity(sev));
            }
            if frame.debug.stopped == Some(b.line) {
                dot_on = true;
            }
        }
        if let Some(row) = frame.debug.stopped.and_then(|l| self.row_of(l)) {
            let cy = f64::from(row.y + self.m.line_h * 0.5);
            // Sobre o ponto, a seta encolhe para dentro dele e leva o contorno do fundo.
            let h = f64::from(if dot_on { r * 0.5 } else { r });
            let x0 = f64::from(cx) - h;
            let x1 = f64::from(cx) + h;
            let mut p = BezPath::new();
            p.move_to((x0, cy - h));
            p.line_to((x1, cy));
            p.line_to((x0, cy + h));
            p.close_path();
            if dot_on {
                scene.stroke_path(
                    &ph2d_vector::Stroke::new(f64::from(ring)),
                    Affine::IDENTITY,
                    &Brush::Solid(pal.gutter),
                    &p,
                );
            }
            scene.fill_path(&p, &Brush::Solid(pal.warn), Affine::IDENTITY);
        }
        scene.pop_layer();
    }

    /// Um valor na mono do código, centrado numa linha de `row_h` que começa em `y` (o cartão do
    /// depurador da aba); devolve onde acabou.
    #[allow(clippy::too_many_arguments)]
    pub fn paint_code_value(
        &mut self,
        ts: &mut TextSystem,
        scene: &mut VectorScene,
        theme: ph2d_tokens::Theme,
        text: &str,
        ink: crate::style::Ink,
        (x, y, row_h): (f32, f32, f32),
        right: f32,
    ) -> f32 {
        let pal = Palette::new(theme);
        let row_y = y + (row_h - self.m.line_h) * 0.5;
        self.paint_mono(ts, scene, &pal, text, ink, x, row_y, right)
    }

    /// O raio do ponto de parada: a meia linha menos a folga (≈ 60 % da linha de diâmetro).
    fn dot_r(&self) -> f32 {
        (self.m.line_h * 0.5 - Spacing::Xs.px()).max(Spacing::Xxs.px())
    }
}

#[cfg(test)]
mod tests {
    /// Um segundo a 60 quadros: os valores ao vivo mudam 3 vezes (0, 400 e 800 ms), não 60.
    #[test]
    fn os_valores_ao_vivo_amostram_duas_vezes_e_meia_por_segundo() {
        let quadro_ns: u128 = 1_000_000_000 / 60;
        let mut sampled = None;
        let mut vezes = 0;
        for i in 0..60 {
            let now = 5_000_000_000 + i * quadro_ns;
            if super::ghosts_due(sampled, now) {
                sampled = Some(now);
                vezes += 1;
            }
        }
        assert_eq!(vezes, 3);
    }
}
