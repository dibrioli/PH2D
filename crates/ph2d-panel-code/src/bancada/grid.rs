//! ⭐ **«Who has what»** (plano 13 Onda B, item 8) — a grelha objectos × propriedades no pé do
//! código, dobrável, no idioma da grelha «Who bumps into whom» da aba Collision Layers: os nomes nas
//! duas bordas, a linha e a coluna sob o rato acesas (`AccentSoft`), a coluna da propriedade sob o
//! cursor do texto acesa de leve, e a célula ALTERADA acesa na cor do objecto (D2). O clique edita
//! pelo MESMO comando do cartão Script do Inspector (a família vira o [`super::Intent`] nele).

use super::{Aim, BancadaView, Who};
use crate::chrome::{self, Ui};
use ph2d_editor_core::icons::IconId;
use ph2d_editor_core::paint::{
    fill_circle, fill_rounded_rect, frame_radius, paint_text, paint_text_title, rect_to_vello,
    resolve, stroke_rounded_rect,
};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::{tr, tr_with};
use ph2d_text::TextSystem;
use ph2d_tokens::{
    ColorToken, PANEL_MIN_W_PX, ROW_H_PX, ROW_SELECTED_ALPHA, Radius, Spacing, StrokeToken, Theme,
    TypeToken,
};
use ph2d_vector::{Color, VectorScene};

/// A parte da coluna do código que a grelha pode tomar: o código manda, a grelha rola.
const BENCH_SHARE: f32 = 1.0 / 3.0; // LITERAL-PX-OK: fração da coluna (um terço), não píxeis

/// As medidas da grelha num quadro: a coluna dos nomes e as das propriedades.
struct Cols {
    x: f32,
    name_w: f32,
    widths: Vec<f32>,
}

impl Cols {
    fn of(ts: &mut TextSystem, who: &Who, x: f32, w: f32) -> Self {
        let pad = Spacing::Md.px();
        let dot = chrome::font();
        let name_w = who
            .holders
            .iter()
            .map(|h| chrome::label_w(ts, &h.name))
            .fold(0.0f32, f32::max)
            + dot
            + pad * 2.0;
        let name_w = ph2d_editor_core::math::safe_clamp(name_w, ROW_H_PX * 2.0, w * 0.5);
        let n = who.columns.len().max(1);
        let share = ((w - name_w) / n as f32).max(0.0);
        let widths = who
            .columns
            .iter()
            .enumerate()
            .map(|(c, col)| {
                let head = chrome::label_w(ts, &col.name);
                let cells = (0..who.holders.len())
                    .filter_map(|h| who.cell(h, c))
                    .map(|cell| chrome::label_w(ts, &cell.text))
                    .fold(head, f32::max);
                (cells + pad * 2.0).max(ROW_H_PX * 2.0).min(share)
            })
            .collect();
        Self { x, name_w, widths }
    }

    fn col_x(&self, c: usize) -> f32 {
        self.x + self.name_w + self.widths[..c].iter().sum::<f32>()
    }
}

impl BancadaView {
    /// A altura do cartão da grelha (e do mapa ao lado) para `who`, com `avail` de altura na coluna
    /// do documento: dobrada, só o título; aberta, o cabeçalho e uma linha por objecto, até um terço.
    #[must_use]
    pub fn bench_height(&self, who: &Who, avail: f32) -> f32 {
        let s = Spacing::Sm.px();
        let title = s * 2.0 + ROW_H_PX;
        if self.folded {
            return title;
        }
        let rows = if who.holders.is_empty() || who.columns.is_empty() {
            1
        } else {
            1 + who.holders.len()
        };
        let body = (ROW_H_PX * rows as f32).min((avail * BENCH_SHARE - title).max(ROW_H_PX * 2.0));
        title + body + s
    }

    /// Pinta a grelha (e o mapa ao lado, ligado e com objectos) em `area`, cada um no seu cartão.
    /// `caret` = a coluna da propriedade sob o cursor do texto.
    #[allow(clippy::too_many_arguments)]
    pub fn paint_bench(
        &mut self,
        who: &Who,
        area: Rect,
        caret: Option<usize>,
        scene: &mut VectorScene,
        ts: &mut TextSystem,
        theme: &Theme,
    ) {
        self.grid_hits.clear();
        self.map_hits.clear();
        let g = crate::cards::gap();
        let map = self.map_on && !self.folded && !who.holders.is_empty();
        let map_w = if map {
            PANEL_MIN_W_PX.min(area.w * 0.5)
        } else {
            0.0
        };
        let grid_r = Rect::new(
            area.x,
            area.y,
            (area.w - if map { map_w + g } else { 0.0 }).max(0.0),
            area.h,
        );
        crate::cards::open(scene, grid_r, theme);
        self.paint_grid(who, grid_r, caret, scene, ts, theme);
        crate::cards::close(scene);
        if map {
            let m = Rect::new(area.x + area.w - map_w, area.y, map_w, area.h);
            crate::cards::open(scene, m, theme);
            self.paint_map(who, m, scene, ts, theme);
            crate::cards::close(scene);
        }
    }

    #[allow(clippy::too_many_lines)]
    fn paint_grid(
        &mut self,
        who: &Who,
        area: Rect,
        caret: Option<usize>,
        scene: &mut VectorScene,
        ts: &mut TextSystem,
        theme: &Theme,
    ) {
        let ui = Ui::new(*theme);
        scene.fill_rect(rect_to_vello(area), ui.bg);
        let s = Spacing::Sm.px();
        let x = area.x + s;
        let w = (area.w - s * 2.0).max(0.0);
        // O título, a contagem e a dobra (a linha inteira dobra, como o cabeçalho de um cartão).
        let title_r = Rect::new(x, area.y + s, w, ROW_H_PX);
        self.fold_rect = title_r;
        let font = TypeToken::Base.px();
        let title = tr("code.bancada.grid.title");
        let fy = title_r.y + (ROW_H_PX - font) * 0.5;
        paint_text_title(ts, scene, title, x, fy, font, w, ui.text1);
        let count = tr_with(
            "code.bancada.grid.count",
            &[
                ("objects", &who.holders.len()),
                ("props", &who.columns.len()),
            ],
        );
        let tw = ts.prefix_width(title, font) + Spacing::Md.px();
        chrome::label_in(
            scene,
            ts,
            &count,
            Rect::new(x + tw, title_r.y, (w - tw - ROW_H_PX).max(0.0), ROW_H_PX),
            ui.text3,
        );
        let chev = Rect::new(x + w - ROW_H_PX, title_r.y, ROW_H_PX, ROW_H_PX);
        let icon = if self.folded {
            IconId::ChevronDown
        } else {
            IconId::ChevronUp
        };
        chrome::icon_button(scene, icon, chev, &ui);
        if self.folded {
            self.grid.rows = crate::chrome::NO_RECT;
            return;
        }
        let top = title_r.y + ROW_H_PX;
        let body = Rect::new(area.x, top, area.w, (area.y + area.h - s - top).max(0.0));
        if who.holders.is_empty() || who.columns.is_empty() {
            let key = if who.holders.is_empty() {
                "code.bancada.grid.no_objects"
            } else {
                "code.bancada.grid.no_props"
            };
            chrome::label_in(scene, ts, tr(key), Rect::new(x, top, w, ROW_H_PX), ui.text3);
            self.grid.rows = crate::chrome::NO_RECT;
            return;
        }
        let cols = Cols::of(ts, who, x, w);
        let radius = frame_radius(*theme, Radius::Sm.px());
        let lit_row = match self.aim {
            Aim::Cell(h, _) | Aim::Holder(h) | Aim::Select(h) => Some(h),
            _ => self.peeking,
        }
        .or(self.editing.as_ref().map(|e| e.holder));
        let lit_col = match self.aim {
            Aim::Cell(_, c) | Aim::Column(c) => Some(c),
            _ => self.editing.as_ref().map(|e| e.column),
        };
        // As linhas rolam por baixo do cabeçalho (que fica).
        self.grid.rows = Rect::new(body.x, top + ROW_H_PX, body.w, (body.h - ROW_H_PX).max(0.0));
        self.grid.content_h = ROW_H_PX * who.holders.len() as f32;
        self.grid.clamp();
        // As faixas das colunas por baixo de tudo: a acesa e a do cursor do texto.
        for (c, cw) in cols.widths.iter().enumerate() {
            let band = if lit_col == Some(c) {
                Some(resolve(ColorToken::AccentSoft, *theme))
            } else if caret == Some(c) {
                Some(ui.hover)
            } else {
                None
            };
            if let Some(color) = band {
                let r = Rect::new(
                    cols.col_x(c),
                    top,
                    *cw,
                    (self.grid.rows.y + self.grid.content_h.min(self.grid.rows.h)) - top,
                );
                fill_rounded_rect(scene, r, radius, color);
            }
        }
        // O cabeçalho: os nomes das propriedades (a acesa em negrito); o clique salta para a linha.
        let gf = chrome::font();
        for (c, col) in who.columns.iter().enumerate() {
            let r = Rect::new(cols.col_x(c), top, cols.widths[c], ROW_H_PX);
            let lit = lit_col == Some(c);
            let paint = if lit { paint_text_title } else { paint_text };
            let tx = r.x + Spacing::Md.px();
            paint(
                ts,
                scene,
                &col.name,
                tx,
                r.y + (r.h - gf) * 0.5,
                gf,
                (r.w - Spacing::Md.px() * 2.0).max(0.0),
                if lit { ui.text1 } else { ui.text2 },
            );
            self.grid_hits.push((Aim::Column(c), r));
        }
        scene.push_clip(&rect_to_vello(self.grid.rows));
        for (h, holder) in who.holders.iter().enumerate() {
            let y = self.grid.rows.y + ROW_H_PX * h as f32 - self.grid.y;
            if y + ROW_H_PX < self.grid.rows.y || y > self.grid.rows.y + self.grid.rows.h {
                continue;
            }
            let color = resolve(ColorToken::layer(holder.color), *theme);
            let lit = lit_row == Some(h);
            let name_r = Rect::new(area.x + s * 0.5, y, cols.name_w + s * 0.5, ROW_H_PX);
            if lit {
                fill_rounded_rect(
                    scene,
                    name_r,
                    radius,
                    resolve(ColorToken::AccentSoft, *theme),
                );
            }
            let dot = super::chip_r();
            fill_circle(scene, x + dot, y + ROW_H_PX * 0.5, dot, color);
            let nx = x + dot * 2.0 + Spacing::Sm.px();
            let paint = if lit { paint_text_title } else { paint_text };
            paint(
                ts,
                scene,
                &holder.name,
                nx,
                y + (ROW_H_PX - gf) * 0.5,
                gf,
                (cols.x + cols.name_w - nx).max(0.0),
                if lit { ui.text1 } else { ui.text2 },
            );
            self.grid_hits
                .push((Aim::Holder(h), clip(name_r, self.grid.rows)));
            for (c, cw) in cols.widths.iter().enumerate() {
                let Some(cell) = who.cell(h, c) else { continue };
                let r = Rect::new(cols.col_x(c), y, *cw, ROW_H_PX);
                let inner = Rect::new(
                    r.x + Spacing::Xxs.px(),
                    r.y + Spacing::Xxs.px(),
                    (r.w - Spacing::Xxs.px() * 2.0).max(0.0),
                    (r.h - Spacing::Xxs.px() * 2.0).max(0.0),
                );
                self.grid_hits
                    .push((Aim::Cell(h, c), clip(r, self.grid.rows)));
                if let Some(e) = self
                    .editing
                    .as_ref()
                    .filter(|e| e.holder == h && e.column == c)
                {
                    chrome::text_field(scene, ts, &e.buf, "", inner, true, &ui);
                    continue;
                }
                if cell.changed {
                    fill_rounded_rect(scene, inner, radius, color.with_alpha(ROW_SELECTED_ALPHA));
                }
                let tx = r.x + Spacing::Md.px();
                if let Some(sw) = cell.swatch {
                    let side = gf;
                    let q = Rect::new(tx, r.y + (r.h - side) * 0.5, side, side);
                    fill_rounded_rect(scene, q, Spacing::Xxs.px(), Color::new(sw));
                } else {
                    let ink = if cell.changed { ui.text1 } else { ui.text2 };
                    paint_text(
                        ts,
                        scene,
                        &cell.text,
                        tx,
                        r.y + (r.h - gf) * 0.5,
                        gf,
                        (r.w - Spacing::Md.px() * 2.0).max(0.0),
                        ink,
                    );
                }
                if self.aim == Aim::Cell(h, c) {
                    stroke_rounded_rect(scene, inner, radius, StrokeToken::Default.px(), ui.text1);
                }
            }
        }
        scene.pop_layer();
    }
}

/// `r` cortada a `b` (um alvo meio rolado só responde na parte à vista).
pub(super) fn clip(r: Rect, b: Rect) -> Rect {
    let (x0, y0) = (r.x.max(b.x), r.y.max(b.y));
    let (x1, y1) = ((r.x + r.w).min(b.x + b.w), (r.y + r.h).min(b.y + b.h));
    Rect::new(x0, y0, (x1 - x0).max(0.0), (y1 - y0).max(0.0))
}
