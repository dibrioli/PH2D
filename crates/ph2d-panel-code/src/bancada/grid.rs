//! ⭐ **«Who has what»** (plano 13 Onda B, item 8; plano 14 Q4 B) — a grelha objectos × propriedades no
//! pé do código, dobrável, sobre a tabela da casa (`ph2d-ui-table`): a coluna dos nomes (o ponto da cor
//! do objecto) e uma por propriedade; a linha e a coluna sob o rato acesas (`AccentSoft`), a coluna do
//! cursor do texto de leve, a célula ALTERADA em âmbar (dono 09/11: âmbar = mudou). O clique edita
//! pelo MESMO comando do cartão Script do Inspector ([`super::Intent`]); a barra ↕ rola em linhas
//! inteiras e o separador ↔ ([`crate::splitter`]) dá a largura ao mapa «Where» do lado.

use super::{Aim, BancadaView, Who};
use crate::chrome::{self, Ui};
use crate::splitter::{Split, band};
use crate::table_text::CodeCells;
use crate::view::CodeView;
use ph2d_code_viewmodel::{CellKind, Holder};
use ph2d_editor_core::doc_surface::PointerLook;
use ph2d_editor_core::icons::IconId;
use ph2d_editor_core::paint::rect_to_vello;
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::{tr, tr_with};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, ROW_H_PX, Spacing, Theme};
use ph2d_ui_table::{Cell, Column, Hit, Ink, Key, Layout, Row, natural_widths};
use ph2d_vector::VectorScene;

/// A parte da coluna do código que a grelha pode tomar: o código manda, a grelha rola.
const BENCH_SHARE: f32 = 1.0 / 3.0; // LITERAL-PX-OK: fração da coluna (um terço), não píxeis

/// A chave da coluna dos nomes: não é um nome de Luau, nunca colide com uma propriedade.
const NAME: &str = "@name";

/// O que a grelha pintou no último quadro: a geometria e as linhas (o clique, a roda e o ponteiro
/// lêem-nas — o mesmo rectângulo desenha e recebe).
pub(super) struct Painted {
    lay: Layout,
    rows: Vec<Row>,
}

/// As colunas: os nomes e uma por propriedade (os números à direita, na mono).
fn columns(who: &Who) -> Vec<Column> {
    let props = who.columns.iter().map(|c| {
        let col = Column::new(c.name.clone(), c.name.clone());
        if c.kind == Some(CellKind::Number) {
            col.end().mono()
        } else {
            col
        }
    });
    // O título da primeira coluna é o da folha do Stopped (a tabela irmã na mesma aba).
    std::iter::once(Column::new(NAME, tr("code.debug.col.name")))
        .chain(props)
        .collect()
}

/// Uma linha por objecto (chave = o índice): o nome com o ponto da cor dele e as células.
fn rows(who: &Who) -> Vec<Row> {
    let ink = |c: usize| match who.columns[c].kind {
        Some(CellKind::Number) => Ink::Number,
        _ => Ink::Plain,
    };
    let row = |(h, holder): (usize, &Holder)| {
        let name = Cell {
            dot: Some(ColorToken::layer(holder.color)),
            ..Cell::new(holder.name.clone(), Ink::Name)
        };
        let cells = (0..who.columns.len()).map(|c| {
            who.cell(h, c).map_or_else(Cell::default, |x| Cell {
                changed: x.changed,
                swatch: x.swatch,
                ..Cell::new(x.text.clone(), ink(c))
            })
        });
        Row {
            key: h.to_string(),
            cells: std::iter::once(name).chain(cells).collect(),
            ..Row::default()
        }
    };
    who.holders.iter().enumerate().map(row).collect()
}

/// A célula de um alvo da grelha, `(chave da linha, coluna)`: a moldura assenta nela.
fn pair(who: &Who, aim: Aim) -> Option<(String, String)> {
    match aim {
        Aim::Cell(h, c) => Some((h.to_string(), who.columns.get(c)?.name.clone())),
        Aim::Holder(h) => Some((h.to_string(), NAME.to_owned())),
        _ => None,
    }
}

/// A chave da coluna da propriedade `c`.
fn key(who: &Who, c: Option<usize>) -> Option<Key> {
    Some(Key::from(who.columns.get(c?)?.name.clone()))
}

impl BancadaView {
    /// A altura do cartão da grelha (e do mapa ao lado) para `who`, com `avail` de altura na coluna
    /// do documento: dobrada, só o título; aberta, o cabeçalho e uma linha por objecto, até um terço
    /// em linhas inteiras (o resto rola).
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
        let fit = ((avail * BENCH_SHARE - title) / ROW_H_PX).floor().max(2.0);
        title + ROW_H_PX * (rows as f32).min(fit) + s
    }

    /// Pinta a grelha (e o mapa ao lado, ligado e com objectos, depois do separador ↔) em `area`,
    /// cada um no seu cartão. `caret` = a coluna da propriedade sob o cursor do texto; os números na
    /// mono de `view` (a do código).
    #[allow(clippy::too_many_arguments)]
    pub fn paint_bench(
        &mut self,
        who: &Who,
        area: Rect,
        caret: Option<usize>,
        view: &mut CodeView,
        scene: &mut VectorScene,
        ts: &mut TextSystem,
        theme: &Theme,
    ) {
        self.grid_hits.clear();
        self.map_hits.clear();
        let map = self.map_on && !self.folded && !who.holders.is_empty();
        // Cada lado guarda ao menos duas colunas mínimas da tabela (os nomes e uma propriedade).
        let min = crate::peso::metricas().min_col * 2.0;
        let split = map.then(|| Split::measure(self.split.side_w, area.w, min, min));
        let side = split.map_or(0.0, |s| s.side_w + band());
        let grid_r = Rect::new(area.x, area.y, (area.w - side).max(0.0), area.h);
        crate::cards::open(scene, grid_r, theme);
        self.paint_grid(who, grid_r, (caret, view), scene, ts, theme);
        crate::cards::close(scene);
        let Some(s) = split else {
            self.split.painted = None;
            return;
        };
        let sx = grid_r.x + grid_r.w;
        self.split.paint(s, (sx, area.y, area.h), scene, *theme);
        let m = Rect::new(sx + band(), area.y, s.side_w, area.h);
        crate::cards::open(scene, m, theme);
        self.paint_map(who, m, scene, ts, theme);
        crate::cards::close(scene);
    }

    fn paint_grid(
        &mut self,
        who: &Who,
        area: Rect,
        (caret, view): (Option<usize>, &mut CodeView),
        scene: &mut VectorScene,
        ts: &mut TextSystem,
        theme: &Theme,
    ) {
        let ui = Ui::new(*theme);
        scene.fill_rect(rect_to_vello(area), ui.bg);
        self.painted = None;
        let s = Spacing::Sm.px();
        let x = area.x + s;
        let w = (area.w - s * 2.0).max(0.0);
        // O título, a contagem e a dobra (a linha inteira dobra, como o cabeçalho de um cartão).
        let title_r = Rect::new(x, area.y + s, w, ROW_H_PX);
        self.fold_rect = title_r;
        // O título no idioma dos cartões da coluna, «SCRIPTS» e «IN THIS SCRIPT» (dono 10/11).
        let title = tr("code.bancada.grid.title");
        super::outline::card_title(scene, ts, title, title_r, *theme);
        let count = tr_with(
            "code.bancada.grid.count",
            &[
                ("objects", &who.holders.len()),
                ("props", &who.columns.len()),
            ],
        );
        let tw = chrome::card_title_w(ts, title) + Spacing::Md.px();
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
            self.steady.reset();
            return;
        }
        let top = title_r.y + ROW_H_PX;
        if who.holders.is_empty() || who.columns.is_empty() {
            let key = if who.holders.is_empty() {
                "code.bancada.grid.no_objects"
            } else {
                "code.bancada.grid.no_props"
            };
            let r = Rect::new(x, top, w, ROW_H_PX);
            if who.holders.is_empty() {
                chrome::label_in(scene, ts, tr(key), r, ui.text3);
            } else {
                // Na letra de «alive» (dono 10/11): a frase traz o código a escrever.
                crate::mono_label::paint(ts, scene, *theme, tr(key), ui.text3, r);
            }
            return;
        }
        let table_r = Rect::new(x, top, w, (area.y + area.h - s - top).max(0.0));
        self.paint_table(who, table_r, (caret, view), scene, ts, theme);
    }

    /// A tabela da grelha em `table_r`, e os alvos dela saídos da MESMA geometria.
    fn paint_table(
        &mut self,
        who: &Who,
        table_r: Rect,
        (caret, view): (Option<usize>, &mut CodeView),
        scene: &mut VectorScene,
        ts: &mut TextSystem,
        theme: &Theme,
    ) {
        let (cols, rows, m) = (columns(who), rows(who), crate::peso::metricas());
        let mut text = CodeCells {
            view,
            theme: *theme,
        };
        let natural = natural_widths(&cols, &rows, &m, ts, &mut text);
        let natural = self.steady.widths(&cols, &natural);
        let editing = self.editing.as_ref().map(|e| (e.holder, e.column));
        let lit_row = match self.aim {
            Aim::Cell(h, _) | Aim::Holder(h) | Aim::Select(h) => Some(h),
            _ => self.peeking,
        }
        .or(editing.map(|e| e.0));
        let lit_col = match self.aim {
            Aim::Cell(_, c) | Aim::Column(c) => Some(c),
            _ => editing.map(|e| e.1),
        };
        self.table.hover = lit_row.map(|h| h.to_string());
        self.table.lit_col = key(who, lit_col);
        self.table.soft_col = key(who, caret);
        let lay = ph2d_ui_table::layout(&cols, &rows, &natural, &mut self.table, table_r, &m);
        let tones = (crate::cards::card_tone(), crate::cards::ground_tone());
        ph2d_ui_table::paint(
            &lay,
            &cols,
            &rows,
            &self.table,
            tones,
            scene,
            ts,
            &mut text,
            *theme,
        );
        let hover = pair(who, self.aim);
        let pressed = self.pressed.and_then(|a| pair(who, a));
        let h = hover.as_ref().map(|(r, c)| (r.as_str(), c.as_str()));
        let p = pressed.as_ref().map(|(r, c)| (r.as_str(), c.as_str()));
        ph2d_ui_table::paint_cell_frames(&lay, &rows, h, p, scene, *theme);
        if let Some(e) = self.editing.as_ref()
            && let Some((row, col)) = pair(who, Aim::Cell(e.holder, e.column))
            && let Some(r) = lay.cell_frame(&rows, &row, &col)
        {
            chrome::text_field(scene, ts, &e.buf, "", r, true, &Ui::new(*theme));
        }
        // Os alvos cortados ao que se vê (cabeçalho, corpo): fora do cartão não tomam o clique.
        let mut push = |aim, r: Rect, b: Rect| {
            let r = clip(r, b);
            if r.w > 0.0 && r.h > 0.0 {
                self.grid_hits.push((aim, r));
            }
        };
        for c in lay.cols.iter().filter(|c| c.index > 0) {
            let r = Rect::new(c.x, lay.header.y, c.w, lay.header.h);
            push(Aim::Column(c.index - 1), r, lay.header);
        }
        for span in &lay.rows {
            let h = span.index;
            for c in &lay.cols {
                let aim = match c.index {
                    0 => Aim::Holder(h),
                    i if who.cell(h, i - 1).is_some() => Aim::Cell(h, i - 1),
                    _ => continue,
                };
                if let Some(r) = lay.cell_rect(h, &c.key) {
                    push(aim, r, lay.body);
                }
            }
        }
        self.painted = Some(Painted { lay, rows });
    }

    /// Carregar na barra ↕, num puxador de coluna ou no separador ↔ começa um arrasto. `true` = tomado.
    pub(super) fn grid_press(&mut self, x: f32, y: f32) -> bool {
        if self.split.press(x, y) {
            return true;
        }
        let Some(p) = &self.painted else { return false };
        match p.lay.hit(&p.rows, x, y) {
            Some(hit @ (Hit::Bar | Hit::Resize(_))) => {
                let (lay, rows) = (p.lay.clone(), p.rows.clone());
                self.table.press(&hit, (x, y), &lay, &rows)
            }
            _ => false,
        }
    }

    /// O rato moveu-se com um arrasto da grelha em curso. `true` = havia um (tomado).
    pub(super) fn grid_drag(&mut self, x: f32, y: f32) -> bool {
        if self.split.drag_to(x) {
            return true;
        }
        self.table.dragging() && {
            self.table.drag_to((x, y), &crate::peso::metricas());
            true
        }
    }

    /// Largar o botão: acaba o arrasto (barra, coluna, separador) e a moldura do clique. `true` =
    /// havia arrasto.
    pub fn release(&mut self) -> bool {
        self.pressed = None;
        self.split.dragging.take().is_some() | self.table.release()
    }

    /// O ponteiro: ↔ no separador, ↕ na barra (e a arrastá-la), a mão no que se clica da grelha
    /// (nomes, células, cabeçalhos). `None` = fora da grelha.
    #[must_use]
    pub fn look(&self, x: f32, y: f32) -> Option<PointerLook> {
        if self.split.look(x, y) {
            return Some(PointerLook::ResizeEw);
        }
        if self.pop_rect.is_some_and(|r| r.contains(x, y)) {
            // No balão, a mão nas linhas dos objectos e no «Select»; a seta no resto.
            let over = |(a, r): &(Aim, Rect)| !matches!(a, Aim::Mark(_)) && r.contains(x, y);
            return self.pop_hits.iter().any(over).then_some(PointerLook::Hand);
        }
        let p = self.painted.as_ref();
        let hit = p.and_then(|p| p.lay.hit(&p.rows, x, y));
        if self.table.bar_dragging() || hit == Some(Hit::Bar) {
            return Some(PointerLook::ResizeNs);
        }
        if matches!(hit, Some(Hit::Resize(_))) {
            return Some(PointerLook::ResizeEw);
        }
        let over = self.grid_hits.iter().any(|(_, r)| r.contains(x, y));
        over.then_some(PointerLook::Hand)
    }

    /// A roda sobre o corpo da grelha: rola em linhas inteiras. `true` = sobre o corpo (tomada).
    pub(super) fn grid_wheel(&mut self, x: f32, y: f32, dy: f32) -> bool {
        let Some(p) = &self.painted else { return false };
        if !p.lay.body.contains(x, y) {
            return false;
        }
        let lay = p.lay.clone();
        let _ = self.table.wheel(&lay, x, y, dy);
        true
    }

    /// A tabela da grelha no último quadro: a geometria, as linhas e o estado (o que acende).
    #[must_use]
    pub fn grid_table(&self) -> Option<(&Layout, &[Row], &ph2d_ui_table::TableState)> {
        let p = self.painted.as_ref()?;
        Some((&p.lay, p.rows.as_slice(), &self.table))
    }

    /// O separador ↔ entre a grelha e o mapa no último quadro (`None` = sem mapa).
    #[must_use]
    pub fn split(&self) -> Option<Split> {
        self.split.painted
    }
}

/// `r` cortada a `b` (um alvo meio rolado só responde na parte à vista).
pub(super) fn clip(r: Rect, b: Rect) -> Rect {
    if r.x >= b.x && r.y >= b.y && r.x + r.w <= b.x + b.w && r.y + r.h <= b.y + b.h {
        return r; // inteira à vista: o mesmo rect, sem o arredondamento de x1 − x0
    }
    let (x0, y0) = (r.x.max(b.x), r.y.max(b.y));
    let (x1, y1) = ((r.x + r.w).min(b.x + b.w), (r.y + r.h).min(b.y + b.h));
    Rect::new(x0, y0, (x1 - x0).max(0.0), (y1 - y0).max(0.0))
}
