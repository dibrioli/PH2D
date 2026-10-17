//! ⭐ **Os glifos de um quadro**: o texto das filas (só a faixa horizontal à vista), os números da
//! calha e o contador de trabalho da pintura — um `draw_glyphs` por grupo (fonte, tamanho, tinta).

use crate::frame::CodeFrame;
use crate::palette::Palette;
use crate::style::Ink;
use crate::view::CodeView;
use ph2d_editor_core::paint::{fill_circle, rect_to_vello};
use ph2d_editor_core::ui_scale::ao_pixel;
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{Spacing, StrokeToken};
use ph2d_vector::{Affine, Fill, FontData, Glyph, VectorScene};

thread_local! {
    static WORK: std::cell::Cell<PaintWork> = const { std::cell::Cell::new(PaintWork { row_bytes: 0, glyphs: 0 }) };
}

/// O trabalho da pintura nesta thread desde o início, sem relógio: os bytes de linha preparados
/// (o texto mostrado de cada fila) e os glifos entregues à cena. O gate da linha longa conta-o.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PaintWork {
    pub row_bytes: u64,
    pub glyphs: u64,
}

#[must_use]
pub fn paint_work() -> PaintWork {
    WORK.with(std::cell::Cell::get)
}

pub(crate) fn add_work(row_bytes: usize, glyphs: usize) {
    WORK.with(|w| {
        let mut v = w.get();
        v.row_bytes += row_bytes as u64;
        v.glyphs += glyphs as u64;
        w.set(v);
    });
}

impl CodeView {
    /// O texto: a lei do `paint_text` do app — dica ligada, linha de base no píxel do ecrã. Os glifos
    /// juntam-se por (fonte, tamanho, tinta) num `draw_glyphs` por grupo e por QUADRO, e lêem-se
    /// pelos grupos de cada corrida: o `line.items()`/`positioned_glyphs` do parley é quadrático na
    /// linha e custava 11,5 ms a 100 linhas coloridas (medido, `tests/it/cronometro.rs`).
    /// ⛔ Só os glifos da faixa horizontal à vista (mais uma margem) vão para a cena: uma linha
    /// comprida pinta o que se vê, não o que tem.
    pub(crate) fn paint_glyphs(&self, scene: &mut VectorScene, pal: &Palette) {
        let x = self.text_x0();
        let tr = self.text_rect();
        let margin = self.m.advance * 4.0; // LITERAL-PX-OK: quatro colunas da mono, não um píxel
        let (left, right) = (tr.x - margin, tr.x + tr.w + margin);
        let mut batches: Vec<Batch> = Vec::new();
        for row in &self.rows {
            let Some(layout) = self.cache.peek(row.key) else {
                continue;
            };
            let top = ao_pixel(row.y + self.m.text_dy);
            'row: for line in layout.lines() {
                let base = line.metrics().baseline;
                let mut off = line.metrics().offset;
                for run in line.runs() {
                    let mut last: Option<(Ink, usize)> = None;
                    for c in run.visual_clusters() {
                        if x + off > right {
                            break 'row;
                        }
                        if x + off + c.advance() < left {
                            off += c.advance();
                            continue;
                        }
                        let ink = c.first_style().brush;
                        let i = match last {
                            Some((k, i)) if k == ink => i,
                            _ => batch(
                                &mut batches,
                                run.font(),
                                run.font_size(),
                                run.normalized_coords(),
                                ink,
                            ),
                        };
                        last = Some((ink, i));
                        for g in c.glyphs() {
                            batches[i].glyphs.push(Glyph {
                                id: g.id,
                                x: x + off + g.x,
                                y: top + ao_pixel(base + g.y),
                            });
                            off += g.advance;
                        }
                    }
                }
            }
        }
        add_work(0, batches.iter().map(|b| b.glyphs.len()).sum());
        for b in batches {
            draw(scene, &b.font, b.size, &b.coords, pal.ink(b.ink), b.glyphs);
        }
    }

    /// A calha: números (glifos dos dígitos já moldados), a marca de problema e o marcador de dobra.
    /// Um texto de uma linha na mono a partir de `x`, cortado antes de `right` (as janelinhas);
    /// devolve onde acabou.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn paint_mono(
        &mut self,
        ts: &mut ph2d_text::TextSystem,
        scene: &mut VectorScene,
        pal: &Palette,
        text: &str,
        ink: Ink,
        x: f32,
        row_y: f32,
        right: f32,
    ) -> f32 {
        let k = self
            .cache
            .touch(ts, text, &[(0..text.len(), ink)], self.m.font_px);
        let Some(layout) = self.cache.peek(k) else {
            return x;
        };
        let top = ao_pixel(row_y + self.m.text_dy);
        let mut batches: Vec<Batch> = Vec::new();
        let mut off = 0.0;
        'line: for line in layout.lines() {
            let base = line.metrics().baseline;
            for run in line.runs() {
                for c in run.visual_clusters() {
                    if x + off + c.advance() > right {
                        break 'line;
                    }
                    let i = batch(
                        &mut batches,
                        run.font(),
                        run.font_size(),
                        run.normalized_coords(),
                        ink,
                    );
                    for g in c.glyphs() {
                        batches[i].glyphs.push(Glyph {
                            id: g.id,
                            x: x + off + g.x,
                            y: top + ao_pixel(base + g.y),
                        });
                        off += g.advance;
                    }
                }
            }
        }
        for b in batches {
            draw(scene, &b.font, b.size, &b.coords, pal.ink(b.ink), b.glyphs);
        }
        x + off
    }

    pub(crate) fn paint_gutter(
        &self,
        frame: &CodeFrame<'_>,
        scene: &mut VectorScene,
        pal: &Palette,
    ) {
        let a = self.area;
        let tr = self.text_rect();
        let head_line = frame
            .buffer
            .text()
            .line_of_byte(frame.buffer.selection().primary().head);
        scene.push_clip(&rect_to_vello(Rect {
            x: a.x,
            y: tr.y,
            w: self.m.gutter_w.min(a.w),
            h: tr.h,
        }));
        let edge = Rect {
            x: a.x + self.m.gutter_w - StrokeToken::Thin.px(),
            y: tr.y,
            w: StrokeToken::Thin.px(),
            h: tr.h,
        };
        scene.fill_rect(rect_to_vello(edge), pal.border);
        let right = a.x + self.m.mark_w + self.m.number_w;
        let dot = Spacing::Sm.px();
        let (mut plain, mut current) = (Vec::new(), Vec::new());
        for row in &self.rows {
            let out = if row.line == head_line {
                &mut current
            } else {
                &mut plain
            };
            self.number_glyphs(out, row.line + 1, right, row.y);
            let worst = frame
                .problems
                .iter()
                .filter(|p| p.line == row.line)
                .map(|p| p.severity)
                .max();
            if let Some(sev) = worst {
                let (cx, cy) = (a.x + self.m.mark_w * 0.5, row.y + self.m.line_h * 0.5);
                fill_circle(scene, cx, cy, dot * 0.5, pal.severity(sev));
            }
            if let Some(f) = self
                .folds
                .iter()
                .find(|f| f.start == row.line && f.end > f.start)
            {
                let cx = a.x + self.m.gutter_w - self.m.fold_w * 0.5;
                crate::palette::fold_marker(
                    scene,
                    cx,
                    row.y + self.m.line_h * 0.5,
                    f.folded,
                    pal.fold,
                );
            }
        }
        if let Some(d) = &self.digits {
            draw(scene, &d.font, d.size, &d.coords, pal.number, plain);
            draw(
                scene,
                &d.font,
                d.size,
                &d.coords,
                pal.number_current,
                current,
            );
        }
        scene.pop_layer();
    }

    fn number_glyphs(&self, out: &mut Vec<Glyph>, n: usize, right: f32, y: f32) {
        let Some(d) = &self.digits else { return };
        let s = n.to_string();
        let count = s.len();
        let base = ao_pixel(y + self.m.text_dy) + ao_pixel(self.m.baseline);
        out.extend(s.bytes().enumerate().map(|(i, b)| Glyph {
            id: d.glyphs[usize::from(b - b'0')],
            x: right - (count - i) as f32 * self.m.advance,
            y: base,
        }));
    }
}

/// Um grupo de glifos que se desenha numa chamada só.
struct Batch {
    font: FontData,
    size: f32,
    coords: Vec<i16>,
    ink: Ink,
    glyphs: Vec<Glyph>,
}

fn batch(v: &mut Vec<Batch>, font: &FontData, size: f32, coords: &[i16], ink: Ink) -> usize {
    let same = |b: &Batch| {
        b.ink == ink
            && b.size == size
            && b.font.index == font.index
            && b.font.data.id() == font.data.id()
            && b.coords == coords
    };
    v.iter().position(same).unwrap_or_else(|| {
        v.push(Batch {
            font: font.clone(),
            size,
            coords: coords.to_vec(),
            ink,
            glyphs: Vec::new(),
        });
        v.len() - 1
    })
}

/// Os glifos em coordenadas de ecrã, com dica.
fn draw(
    scene: &mut VectorScene,
    font: &FontData,
    size: f32,
    coords: &[i16],
    color: ph2d_vector::Color,
    glyphs: Vec<Glyph>,
) {
    if glyphs.is_empty() {
        return;
    }
    scene
        .inner_mut()
        .draw_glyphs(font)
        .font_size(size)
        .hint(true)
        .normalized_coords(coords)
        .brush(color)
        .transform(Affine::IDENTITY)
        .draw(Fill::NonZero, glyphs.into_iter());
}
