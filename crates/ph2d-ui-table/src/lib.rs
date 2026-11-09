//! ⭐ **A tabela da casa** (plano 14, Onda Q1 — `docs/CodeEditor/14_plano_quadros.md`): a planilha dos
//! quadros do depurador e da Bancada, no idioma das janelas Locals/Watch do Visual Studio —
//! cabeçalho fixo, colunas arrastáveis e ocultáveis (botão direito no cabeçalho), grupos e tabelas
//! aninhadas que dobram (▸), números à direita na mono, linhas listradas, rolagem por dentro, a linha
//! que MUDOU em âmbar com o valor antigo riscado, e o traço dos últimos segundos numa célula.
//!
//! Três peças: o **modelo** ([`Column`], [`Row`], [`Cell`]) que o anfitrião escreve a cada quadro, o
//! **estado** ([`TableState`]: larguras, colunas escondidas, dobras, rolagem, arrasto) que vive com
//! ele, e a **geometria** pura ([`layout`] → [`Layout`], [`Layout::hit`]) que a pintura
//! ([`paint`]) e o clique partilham — o mesmo rectângulo desenha e recebe.

mod paint;

use std::collections::{BTreeMap, BTreeSet};

use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, ROW_H_PX, Spacing};

pub use paint::{
    CellText, UiText, changed_tint, ink_color, paint, paint_spark, spark_points, ui_font,
};

/// Onde o texto encosta na célula (os números à direita, para os dígitos se compararem).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Align {
    #[default]
    Start,
    End,
}

/// Uma coluna. A chave é estável (é ela que guarda a largura e o «escondida»).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Column {
    pub key: &'static str,
    pub title: String,
    pub align: Align,
    /// O texto das células na letra do código.
    pub mono: bool,
    /// Pode esconder-se pelo menu do cabeçalho.
    pub hideable: bool,
    /// Toma a largura que sobra.
    pub flex: bool,
    /// A célula guarda um traço (a largura natural é a do traço, não a do texto).
    pub spark: bool,
}

impl Column {
    #[must_use]
    pub fn new(key: &'static str, title: impl Into<String>) -> Self {
        Self {
            key,
            title: title.into(),
            align: Align::Start,
            mono: false,
            hideable: false,
            flex: false,
            spark: false,
        }
    }
    #[must_use]
    pub fn end(mut self) -> Self {
        self.align = Align::End;
        self
    }
    #[must_use]
    pub fn mono(mut self) -> Self {
        self.mono = true;
        self
    }
    #[must_use]
    pub fn hideable(mut self) -> Self {
        self.hideable = true;
        self
    }
    #[must_use]
    pub fn flex(mut self) -> Self {
        self.flex = true;
        self
    }
    #[must_use]
    pub fn spark(mut self) -> Self {
        self.spark = true;
        self
    }
}

/// O tom de um texto: o anfitrião decide a cor (o editor de scripts usa a do realce do código).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Ink {
    #[default]
    Plain,
    /// Rótulos e o que é secundário (`number`, `{4 fields}`).
    Muted,
    /// O nome de uma variável.
    Name,
    Number,
    Text,
    Boolean,
    Nil,
    Table,
    Function,
    /// O valor que mudou (âmbar).
    Changed,
    Danger,
    Warn,
    Accent,
}

/// Uma célula: texto, e à volta dele o que a planilha sabe pintar.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Cell {
    pub text: String,
    pub ink: Ink,
    /// O valor de antes, riscado à esquerda do novo («mudou»).
    pub old: Option<String>,
    /// Um ponto de cor antes do texto (a cor de um objecto).
    pub dot: Option<ColorToken>,
    /// O traço dos últimos segundos (amostras, a mais antiga primeiro).
    pub spark: Option<Vec<f32>>,
}

impl Cell {
    #[must_use]
    pub fn new(text: impl Into<String>, ink: Ink) -> Self {
        Self {
            text: text.into(),
            ink,
            ..Self::default()
        }
    }
}

/// Uma linha. As filhas vêm logo a seguir com `depth` maior; dobrar esconde-as.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Row {
    /// Única na tabela e estável entre quadros (guarda a dobra e o realce).
    pub key: String,
    pub depth: u8,
    /// Um cabeçalho de grupo (`Locals · 1`): a 1.ª célula é o título, isto é a contagem.
    pub group: Option<String>,
    pub cells: Vec<Cell>,
    /// A linha mudou (fundo âmbar).
    pub changed: bool,
    /// Tem filhas (mostra ▸/▾).
    pub expandable: bool,
    /// Aberta enquanto ninguém lhe tocar.
    pub default_open: bool,
}

/// As medidas da tabela, dos tokens.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Metrics {
    pub row_h: f32,
    pub header_h: f32,
    pub pad: f32,
    pub indent: f32,
    pub min_col: f32,
    /// A meia largura do puxador entre colunas.
    pub handle: f32,
    /// A largura natural de um traço.
    pub spark_w: f32,
}

impl Metrics {
    #[must_use]
    pub fn standard() -> Self {
        Self {
            row_h: ROW_H_PX,
            header_h: ROW_H_PX,
            pad: Spacing::Sm.px(),
            indent: Spacing::Lg.px(),
            min_col: ROW_H_PX * 2.0,
            handle: Spacing::Xs.px(),
            spark_w: ROW_H_PX * 4.0,
        }
    }

    /// A caixa do ▸ (e a reserva dela na 1.ª coluna).
    #[must_use]
    pub fn chevron(&self) -> f32 {
        self.row_h * 0.5 + self.pad
    }
}

/// Um arrasto de largura em curso.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Drag {
    key: &'static str,
    x0: f32,
    w0: f32,
}

/// O que a tabela lembra entre quadros.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TableState {
    /// As larguras que o utilizador arrastou.
    pub widths: BTreeMap<&'static str, f32>,
    pub hidden: BTreeSet<&'static str>,
    /// As linhas cuja dobra difere da de omissão.
    pub toggled: BTreeSet<String>,
    pub scroll: f32,
    /// A linha sob o rato.
    pub hover: Option<String>,
    drag: Option<Drag>,
    /// O arrasto da barra de rolagem.
    bar: Option<ph2d_editor_core::widget::scroll_area::BarDrag>,
}

/// Onde o rato está na tabela.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Hit {
    /// O puxador à direita da coluna.
    Resize(&'static str),
    Header(&'static str),
    /// A barra de rolagem do corpo (polegar ou trilha).
    Bar,
    /// O ▸ de uma linha (ou um cabeçalho de grupo inteiro).
    Toggle(usize),
    Cell {
        row: usize,
        col: &'static str,
    },
}

impl TableState {
    #[must_use]
    pub fn is_open(&self, row: &Row) -> bool {
        row.default_open != self.toggled.contains(&row.key)
    }

    pub fn toggle_row(&mut self, key: &str) {
        if !self.toggled.remove(key) {
            self.toggled.insert(key.to_owned());
        }
    }

    pub fn set_hidden(&mut self, key: &'static str, hidden: bool) {
        if hidden {
            self.hidden.insert(key);
        } else {
            self.hidden.remove(key);
        }
    }

    /// As linhas à vista (nenhum antepassado dobrado), pela ordem.
    #[must_use]
    pub fn visible_rows(&self, rows: &[Row]) -> Vec<usize> {
        let mut out = Vec::with_capacity(rows.len());
        // A profundidade do 1.º antepassado dobrado em curso.
        let mut closed_at: Option<u8> = None;
        for (i, r) in rows.iter().enumerate() {
            if let Some(d) = closed_at {
                if r.depth > d {
                    continue;
                }
                closed_at = None;
            }
            out.push(i);
            if r.expandable && !self.is_open(r) {
                closed_at = Some(r.depth);
            }
        }
        out
    }

    /// Um botão premido em `hit` (`(x, y)` do rato): começa o arrasto (puxador ou barra) ou vira a
    /// dobra. `true` = mudou.
    pub fn press(&mut self, hit: &Hit, (x, y): (f32, f32), lay: &Layout, rows: &[Row]) -> bool {
        match hit {
            Hit::Bar => {
                self.bar = lay.bar().press(x, y, self.scroll).map(|(d, s)| {
                    self.scroll = s;
                    d
                });
                self.bar.is_some()
            }
            Hit::Resize(key) => {
                let w0 = lay.col(key).map_or(0.0, |c| c.w);
                self.drag = Some(Drag { key, x0: x, w0 });
                true
            }
            Hit::Toggle(i) => {
                if let Some(r) = rows.get(*i) {
                    let k = r.key.clone();
                    self.toggle_row(&k);
                    if self.is_open(r) {
                        self.reveal_children(*i, lay, rows);
                    }
                }
                true
            }
            _ => false,
        }
    }

    /// Abrir uma dobra rola (em linhas inteiras) até as filhas caberem à vista, sem esconder a mãe.
    fn reveal_children(&mut self, i: usize, lay: &Layout, rows: &[Row]) {
        let Some(m) = lay.m else { return };
        let shown = self.visible_rows(rows);
        let Some(order) = shown.iter().position(|&k| k == i) else {
            return;
        };
        let depth = rows[i].depth;
        let kids = shown[order + 1..]
            .iter()
            .take_while(|&&k| rows[k].depth > depth)
            .count();
        let fits = (lay.body.h / m.row_h).floor().max(1.0);
        let last = (order + kids) as f32;
        let first = (last + 1.0 - fits).min(order as f32).max(0.0);
        if first * m.row_h > self.scroll {
            self.scroll = first * m.row_h;
        }
    }

    /// O rato moveu-se com o botão premido. `true` = a largura (ou a rolagem) mudou.
    pub fn drag_to(&mut self, (x, y): (f32, f32), m: &Metrics) -> bool {
        if let Some(b) = self.bar {
            let before = std::mem::replace(&mut self.scroll, b.scroll_at(y));
            return (self.scroll - before).abs() > f32::EPSILON;
        }
        let Some(d) = self.drag else { return false };
        let w = (d.w0 + x - d.x0).max(m.min_col);
        self.widths.insert(d.key, w) != Some(w)
    }

    /// Larga o arrasto. `true` = havia um.
    pub fn release(&mut self) -> bool {
        self.bar.take().is_some() | self.drag.take().is_some()
    }

    #[must_use]
    pub fn dragging(&self) -> bool {
        self.drag.is_some() || self.bar.is_some()
    }

    /// A barra de rolagem está a ser arrastada?
    #[must_use]
    pub fn bar_dragging(&self) -> bool {
        self.bar.is_some()
    }

    /// A roda sobre o corpo. `true` = rolou.
    pub fn wheel(&mut self, lay: &Layout, x: f32, y: f32, dy: f32) -> bool {
        if !lay.body.contains(x, y) || lay.max_scroll() <= 0.0 {
            return false;
        }
        let before = self.scroll;
        // Linhas inteiras: um toque da roda anda pelo menos uma linha (o `layout` arredonda).
        let rh = lay.m.map_or(1.0, |m| m.row_h.max(1.0));
        let step = (dy.abs() / rh).ceil().max(1.0) * rh * dy.signum();
        self.scroll = ph2d_editor_core::math::safe_clamp(self.scroll - step, 0.0, lay.max_scroll());
        (self.scroll - before).abs() > f32::EPSILON
    }

    /// O menu do cabeçalho: `(chave, título, à vista)` de cada coluna que se pode esconder.
    #[must_use]
    pub fn column_menu<'a>(&self, cols: &'a [Column]) -> Vec<(&'static str, &'a str, bool)> {
        cols.iter()
            .filter(|c| c.hideable)
            .map(|c| (c.key, c.title.as_str(), !self.hidden.contains(c.key)))
            .collect()
    }
}

/// Uma coluna à vista na geometria.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ColSpan {
    pub key: &'static str,
    /// O índice em `cols` (e nas células de cada linha).
    pub index: usize,
    pub x: f32,
    pub w: f32,
}

/// Uma linha à vista na geometria.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RowSpan {
    /// O índice em `rows`.
    pub index: usize,
    /// A ordem à vista (a listra).
    pub order: usize,
    pub rect: Rect,
}

/// ⭐ A geometria de um quadro: a mesma para pintar e para clicar.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Layout {
    pub area: Rect,
    pub header: Rect,
    pub body: Rect,
    pub cols: Vec<ColSpan>,
    /// Só as linhas que tocam o corpo.
    pub rows: Vec<RowSpan>,
    pub content_h: f32,
    pub scroll: f32,
    pub m: Option<Metrics>,
}

impl Layout {
    #[must_use]
    pub fn max_scroll(&self) -> f32 {
        max_scroll_rows(self.content_h, self.body.h, self.m.map_or(0.0, |m| m.row_h))
    }

    /// A barra de rolagem do corpo (a da casa: a mesma geometria pinta e agarra).
    #[must_use]
    pub fn bar(&self) -> ph2d_editor_core::widget::scroll_area::OwnBar {
        ph2d_editor_core::widget::scroll_area::OwnBar {
            body: self.body,
            content_h: self.max_scroll() + self.body.h,
            visible_h: self.body.h,
        }
    }

    #[must_use]
    pub fn col(&self, key: &str) -> Option<&ColSpan> {
        self.cols.iter().find(|c| c.key == key)
    }

    /// O que está em `(x, y)`.
    #[must_use]
    pub fn hit(&self, rows: &[Row], x: f32, y: f32) -> Option<Hit> {
        let m = self.m?;
        if !self.area.contains(x, y) {
            return None;
        }
        let last = self.cols.len().saturating_sub(1);
        if self.header.contains(x, y) {
            for (i, c) in self.cols.iter().enumerate() {
                if i < last && (x - (c.x + c.w)).abs() <= m.handle {
                    return Some(Hit::Resize(c.key));
                }
            }
            return self
                .cols
                .iter()
                .find(|c| x >= c.x && x < c.x + c.w)
                .map(|c| Hit::Header(c.key));
        }
        if self.bar().hit().is_some_and(|b| b.contains(x, y)) {
            return Some(Hit::Bar);
        }
        let span = self
            .rows
            .iter()
            .find(|r| r.rect.contains(x, y) && y >= self.body.y && y < self.body.y + self.body.h)?;
        let row = rows.get(span.index)?;
        if row.group.is_some() {
            return Some(Hit::Toggle(span.index));
        }
        let first = self.cols.first()?;
        let cx = first.x + m.pad + f32::from(row.depth) * m.indent;
        if row.expandable && x >= cx && x < cx + m.chevron() {
            return Some(Hit::Toggle(span.index));
        }
        self.cols
            .iter()
            .find(|c| x >= c.x && x < c.x + c.w)
            .map(|c| Hit::Cell {
                row: span.index,
                col: c.key,
            })
    }

    /// A célula `(linha, coluna)` à vista.
    #[must_use]
    pub fn cell_rect(&self, row: usize, col: &str) -> Option<Rect> {
        let r = self.rows.iter().find(|s| s.index == row)?;
        let c = self.col(col)?;
        Some(Rect::new(c.x, r.rect.y, c.w, r.rect.h))
    }
}

/// A largura natural de cada coluna (o título e a célula mais larga), medida por `text`.
pub fn natural_widths(
    cols: &[Column],
    rows: &[Row],
    m: &Metrics,
    ts: &mut TextSystem,
    text: &mut dyn CellText,
) -> Vec<f32> {
    cols.iter()
        .enumerate()
        .map(|(ci, c)| {
            let head = text.width(ts, &c.title, false);
            let cells = rows
                .iter()
                .filter(|r| r.group.is_none())
                .filter_map(|r| r.cells.get(ci).map(|cell| (r, cell)))
                .map(|(r, cell)| {
                    let mut w = text.width(ts, &cell.text, c.mono);
                    if let Some(old) = &cell.old {
                        w += text.width(ts, old, c.mono) + m.pad;
                    }
                    if cell.dot.is_some() {
                        w += m.row_h * 0.5;
                    }
                    if cell.spark.is_some() {
                        w = w.max(m.spark_w);
                    }
                    if ci == 0 {
                        w += f32::from(r.depth) * m.indent + m.chevron();
                    }
                    w
                })
                .fold(0.0f32, f32::max);
            let natural = if c.spark { m.spark_w } else { head.max(cells) };
            (natural + m.pad * 2.0).max(m.min_col)
        })
        .collect()
}

/// ⭐ **A geometria** de `rows` em `area`: as colunas à vista com a largura arrastada (ou a natural),
/// a coluna `flex` com o que sobra, o corpo rolado (o `scroll` do estado é clampado aqui).
#[must_use]
pub fn layout(
    cols: &[Column],
    rows: &[Row],
    natural: &[f32],
    st: &mut TableState,
    area: Rect,
    m: &Metrics,
) -> Layout {
    let shown: Vec<usize> = (0..cols.len())
        .filter(|&i| !st.hidden.contains(cols[i].key))
        .collect();
    let mut widths: Vec<f32> = shown
        .iter()
        .map(|&i| {
            st.widths
                .get(cols[i].key)
                .copied()
                .unwrap_or_else(|| natural.get(i).copied().unwrap_or(m.min_col))
                .max(m.min_col)
        })
        .collect();
    if let Some(f) = shown.iter().position(|&i| cols[i].flex)
        && !st.widths.contains_key(cols[shown[f]].key)
    {
        let others: f32 = widths
            .iter()
            .enumerate()
            .filter(|&(k, _)| k != f)
            .map(|(_, w)| *w)
            .sum();
        widths[f] = widths[f].max(area.w - others);
    }
    let mut x = area.x;
    let spans: Vec<ColSpan> = shown
        .iter()
        .zip(&widths)
        .map(|(&i, &w)| {
            let s = ColSpan {
                key: cols[i].key,
                index: i,
                x,
                w,
            };
            x += w;
            s
        })
        .collect();
    let header = Rect::new(area.x, area.y, area.w, m.header_h.min(area.h));
    let body = Rect::new(
        area.x,
        header.y + header.h,
        area.w,
        (area.h - header.h).max(0.0),
    );
    let visible = st.visible_rows(rows);
    let content_h = visible.len() as f32 * m.row_h;
    // ⭐ A rolagem anda em LINHAS INTEIRAS (roda, revelar, barra): nenhuma linha a meio por baixo
    // do cabeçalho (foto da cena 14: a linha cortada lia-se dentro dele). O fim arredonda para
    // cima: a última linha fica inteira à vista.
    let max = max_scroll_rows(content_h, body.h, m.row_h);
    let snapped = if m.row_h > 0.0 {
        (st.scroll / m.row_h).round() * m.row_h
    } else {
        st.scroll
    };
    st.scroll = ph2d_editor_core::math::safe_clamp(snapped, 0.0, max);
    let row_spans = visible
        .into_iter()
        .enumerate()
        .filter_map(|(order, index)| {
            let y = body.y - st.scroll + order as f32 * m.row_h;
            // Só linhas INTEIRAS: nem a meio por baixo do cabeçalho, nem cortadas no pé.
            let eps = 0.01; // LITERAL-PX-OK: folga de arredondamento em píxeis, não medida de UI
            (y >= body.y - eps && y + m.row_h <= body.y + body.h + eps).then_some(RowSpan {
                index,
                order,
                rect: Rect::new(area.x, y, area.w, m.row_h),
            })
        })
        .collect();
    Layout {
        area,
        header,
        body,
        cols: spans,
        rows: row_spans,
        content_h,
        scroll: st.scroll,
        m: Some(*m),
    }
}

/// A rolagem máxima em linhas inteiras (a parte que não cabe, arredondada para cima à linha).
fn max_scroll_rows(content_h: f32, body_h: f32, row_h: f32) -> f32 {
    let over = (content_h - body_h).max(0.0);
    if row_h > 0.0 {
        (over / row_h).ceil() * row_h
    } else {
        over
    }
}

/// A altura que a tabela pede para mostrar `rows` sem rolar (o cabeçalho e as linhas à vista).
#[must_use]
pub fn natural_height(rows: &[Row], st: &TableState, m: &Metrics) -> f32 {
    m.header_h + st.visible_rows(rows).len() as f32 * m.row_h
}
