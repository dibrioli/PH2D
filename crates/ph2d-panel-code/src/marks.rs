//! ⭐ **As marcas da Bancada na margem** (plano 13 Onda B, item 7): ◆ na linha de cada
//! `ph2d.property`, ⚡ em cada sinal que o `on_signal` ouve, e, no fim da linha da propriedade, um
//! ponto na cor de cada objecto cujo valor difere do do script. O rato sobre a marca (ou os pontos)
//! abre o balão «em quem» ([`crate::bancada`]); o ponto de parada da mesma linha ganha à marca.

use crate::frame::CodeFrame;
use crate::geom;
use crate::palette::Palette;
use crate::view::CodeView;
use ph2d_editor_core::icons::IconId;
use ph2d_editor_core::paint::{fill_circle, paint_icon, resolve};
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{ColorToken, Spacing, StrokeToken};
use ph2d_vector::{Affine, BezPath, Brush, VectorScene};

impl CodeView {
    /// As marcas da Bancada pintadas no último quadro: a linha e a caixa (a marca e os pontos).
    #[must_use]
    pub fn bancada_marks(&self) -> &[(usize, Rect)] {
        &self.bancada_marks
    }

    /// O raio de um ponto de objecto no fim da linha.
    fn dot_of_holder(&self) -> f32 {
        Spacing::Xs.px().min(self.m.line_h * 0.5)
    }

    /// Os pontos dos objectos que mudaram cada propriedade, depois do fim da linha (dentro do
    /// recorte do texto); a caixa deles entra no alvo da marca.
    pub(crate) fn paint_bancada_dots(
        &mut self,
        frame: &CodeFrame<'_>,
        scene: &mut VectorScene,
        pal: &Palette,
    ) {
        self.bancada_marks.clear();
        let Some(who) = frame.who else { return };
        let r = self.dot_of_holder();
        let step = r * 2.0 + Spacing::Xxs.px();
        for (c, col) in who.columns.iter().enumerate() {
            let Some(row) = self.row_of(col.line) else {
                continue;
            };
            let folded = self.folds.iter().any(|f| f.folded && f.start == col.line);
            let Some(layout) = self.cache.peek(row.key).filter(|_| !row.cut && !folded) else {
                continue;
            };
            let x0 = self.row_x0(row) + geom::width(layout) + Spacing::Lg.px();
            let cy = row.y + self.m.line_h * 0.5;
            let mut x = x0;
            for (h, holder) in who.holders.iter().enumerate() {
                if who.cell(h, c).is_some_and(|cell| cell.changed) {
                    let color = resolve(ColorToken::layer(holder.color), pal.theme);
                    fill_circle(scene, x + r, cy, r, color);
                    x += step;
                }
            }
            if x > x0 {
                let dots = Rect::new(x0, row.y, x - x0, self.m.line_h);
                self.bancada_marks.push((col.line, dots));
            }
        }
    }

    /// ◆ e ⚡ na coluna das marcas (o ponto de parada da linha ganha), e o alvo de cada uma.
    pub(crate) fn paint_bancada_gutter(
        &mut self,
        frame: &CodeFrame<'_>,
        scene: &mut VectorScene,
        pal: &Palette,
    ) {
        let Some(who) = frame.who else { return };
        let tr = self.text_rect();
        let a = self.area;
        let ink = resolve(ColorToken::Accent, pal.theme);
        let lines = who
            .columns
            .iter()
            .map(|c| (c.line, false))
            .chain(who.hears.iter().map(|e| (e.1, true)));
        let mut painted = Vec::new();
        for (line, hears) in lines {
            let Some(row) = self.row_of(line) else {
                continue;
            };
            let cell = Rect::new(a.x, row.y, self.m.mark_w, self.m.line_h);
            if row.y < tr.y || row.y >= tr.y + tr.h {
                continue;
            }
            painted.push((line, cell));
            if frame.debug.breakpoints.iter().any(|b| b.line == line) {
                continue;
            }
            let h = f64::from(Spacing::Xs.px().min(self.m.line_h * 0.5));
            let (cx, cy) = (
                f64::from(cell.x + cell.w * 0.5),
                f64::from(cell.y + cell.h * 0.5),
            );
            if hears {
                let s = (self.m.line_h * 0.5 + Spacing::Xxs.px()).min(cell.w);
                let icon = Rect::new(
                    cell.x + (cell.w - s) * 0.5,
                    cell.y + (cell.h - s) * 0.5,
                    s,
                    s,
                );
                paint_icon(scene, IconId::Bolt, icon, ink, StrokeToken::Default.px());
            } else {
                let mut p = BezPath::new();
                p.move_to((cx, cy - h));
                p.line_to((cx + h, cy));
                p.line_to((cx, cy + h));
                p.line_to((cx - h, cy));
                p.close_path();
                scene.fill_path(&p, &Brush::Solid(ink), Affine::IDENTITY);
            }
        }
        // A marca e os pontos da mesma linha abrem o MESMO balão, mas são DOIS alvos: o texto entre
        // eles é do cursor (uma caixa só roubava o clique no texto da linha da propriedade).
        self.bancada_marks.splice(0..0, painted);
    }
}
