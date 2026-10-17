//! ⭐ **O estado de VISTA de um documento** — rolagem, dobras, as linhas que o último quadro pintou.
//! A pintura (`paint.rs`) enche-o; o clique, o cursor do IME e o «manter o cursor à vista» lêem-no.

use crate::display::DisplayLine;
use crate::frame::{FoldMark, Severity};
use crate::geom::{self, Column};
use crate::line_cache::LineCache;
use crate::metrics::{DigitFont, Metrics};
use ph2d_code_buffer::Buffer;
use ph2d_editor_core::ui_scale::ao_pixel;
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::Spacing;
use std::ops::Range;

/// Onde caiu um ponto da área do editor.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Hit {
    /// No texto: o byte do buffer mais próximo (o meio de cada glifo decide).
    Text(usize),
    /// Na calha, na linha (0-based).
    Gutter(usize),
    /// No marcador de dobra da linha (0-based).
    Fold(usize),
    Status,
    Outside,
}

/// Um sublinhado de problema que a pintura pôs (o instrumento dos testes e da auditoria).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Underline {
    pub line: usize,
    pub x0: f32,
    pub x1: f32,
    pub y0: f32,
    pub y1: f32,
    pub severity: Severity,
}

/// Uma linha pintada: o que o clique precisa para voltar ao byte.
pub(crate) struct Row {
    pub line: usize,
    /// Byte do buffer onde a linha começa e o comprimento dela (sem `\n`).
    pub start: usize,
    pub len: usize,
    /// A linha passa do tecto de pintura (`render_cap`): `len` é só o que se mostra.
    pub cut: bool,
    pub y: f32,
    pub key: u64,
    pub dl: DisplayLine,
}

pub struct CodeView {
    pub(crate) cache: LineCache,
    pub(crate) digits: Option<DigitFont>,
    pub(crate) m: Metrics,
    pub(crate) area: Rect,
    pub(crate) scroll_y: f32,
    pub(crate) scroll_x: f32,
    pub(crate) tab: usize,
    pub(crate) line_count: usize,
    pub(crate) revision: u64,
    /// Faixas de linhas escondidas por dobras, ordenadas e fundidas (inclusivas).
    pub(crate) hidden: Vec<(usize, usize)>,
    /// As dobras que começam nas linhas pintadas no último quadro (calha, «…», clique).
    pub(crate) folds: Vec<FoldMark>,
    pub(crate) rows: Vec<Row>,
    pub(crate) underlines: Vec<Underline>,
    /// A lista de completação do último quadro.
    pub(crate) popup: Option<crate::popup::Painted>,
    /// O balão do rato e a assinatura do último quadro.
    pub(crate) hints: crate::hint::Painted,
    /// Os bytes que uma fila mostra (o resto da linha não se prepara nem se molda).
    pub(crate) render_cap: usize,
    /// A coluna da barra de estado da última (revisão, cursor, tab).
    pub(crate) status_col: Option<((u64, usize, usize), usize)>,
}

/// ⛔ **Os bytes de uma linha que a pintura mostra** — o resto acaba num «…». ⚠️ **De que recurso
/// é:** do QUADRO (preparar, colorir, moldar e pôr na cena cada fila à vista custa a fila inteira).
/// Medido (`docs/CodeEditor/02_onda1.md` §Medições do fecho, C): 40 filas à vista, todas no tecto,
/// custam 1.97 ms parado e 2.53 ms a moldar; a 10 000 B já 8.9/11.3 ms.
pub const LINE_RENDER_BYTES_MAX: usize = 2_000;

impl Default for CodeView {
    fn default() -> Self {
        Self {
            cache: LineCache::default(),
            digits: None,
            m: Metrics::default(),
            area: Rect {
                x: 0.0,
                y: 0.0,
                w: 0.0,
                h: 0.0,
            },
            scroll_y: 0.0,
            scroll_x: 0.0,
            tab: 1,
            line_count: 1,
            revision: 0,
            hidden: Vec::new(),
            folds: Vec::new(),
            rows: Vec::new(),
            underlines: Vec::new(),
            popup: None,
            hints: crate::hint::Painted::default(),
            render_cap: LINE_RENDER_BYTES_MAX,
            status_col: None,
        }
    }
}

impl CodeView {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn metrics(&self) -> &Metrics {
        &self.m
    }

    #[must_use]
    pub fn cache(&self) -> &LineCache {
        &self.cache
    }

    /// O tecto de bytes por fila ([`LINE_RENDER_BYTES_MAX`]); a bancada troca-o em execução.
    pub fn set_render_cap(&mut self, bytes: usize) {
        self.render_cap = bytes.max(1);
    }

    /// Esquece os layouts (a medição a frio).
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    #[must_use]
    pub fn scroll(&self) -> (f32, f32) {
        (self.scroll_x, self.scroll_y)
    }

    /// Os sublinhados de problema do último quadro.
    #[must_use]
    pub fn last_underlines(&self) -> &[Underline] {
        &self.underlines
    }

    /// A zona do texto (sem calha nem barra de estado).
    #[must_use]
    pub fn text_rect(&self) -> Rect {
        let a = self.area;
        let h = (a.h - self.m.status_h).max(0.0);
        Rect {
            x: a.x + self.m.gutter_w,
            y: a.y,
            w: (a.w - self.m.gutter_w).max(0.0),
            h,
        }
    }

    /// O x onde a coluna 0 do texto começa (com a rolagem horizontal), no píxel do ecrã.
    pub(crate) fn text_x0(&self) -> f32 {
        ao_pixel(self.text_rect().x + self.m.text_pad - self.scroll_x)
    }

    /// A rolagem vertical no píxel do ecrã (a guardada acumula as fracções da roda).
    pub(crate) fn sy(&self) -> f32 {
        ao_pixel(self.scroll_y)
    }

    /// O y do topo da fila visual `row`.
    pub(crate) fn row_y(&self, row: usize) -> f32 {
        self.text_rect().y + row as f32 * self.m.line_h - self.sy()
    }

    /// As linhas escondidas pelas dobras fechadas `closed`.
    pub(crate) fn set_hidden(&mut self, closed: &[FoldMark], line_count: usize) {
        self.hidden.clear();
        let last = line_count.saturating_sub(1);
        let mut v: Vec<(usize, usize)> = closed
            .iter()
            .filter(|f| f.folded && f.end > f.start && f.start < last)
            .map(|f| (f.start + 1, f.end.min(last)))
            .collect();
        v.sort_unstable();
        for (a, b) in v {
            match self.hidden.last_mut() {
                Some(l) if a <= l.1 + 1 => l.1 = l.1.max(b),
                _ => self.hidden.push((a, b)),
            }
        }
    }

    pub(crate) fn total_rows(&self) -> usize {
        let hid: usize = self.hidden.iter().map(|(a, b)| b - a + 1).sum();
        self.line_count.saturating_sub(hid).max(1)
    }

    /// A linha da fila visual `row` (as escondidas não contam).
    pub(crate) fn line_of_row(&self, row: usize) -> usize {
        let mut line = row;
        for &(a, b) in &self.hidden {
            if a <= line {
                line += b - a + 1;
            } else {
                break;
            }
        }
        line.min(self.line_count.saturating_sub(1))
    }

    /// A fila de `line`; uma linha escondida dá a fila da linha da dobra.
    pub(crate) fn row_of_line(&self, line: usize) -> usize {
        let mut row = line;
        for &(a, b) in &self.hidden {
            if b < line {
                row -= b - a + 1;
            } else if a <= line {
                return row - (line - a) - 1;
            } else {
                break;
            }
        }
        row
    }

    pub(crate) fn is_hidden(&self, line: usize) -> bool {
        self.hidden.iter().any(|&(a, b)| a <= line && line <= b)
    }

    /// Quantas linhas inteiras cabem na vista (Page Up/Down).
    #[must_use]
    pub fn page_rows(&self) -> usize {
        if self.m.line_h <= 0.0 {
            return 1;
        }
        ((self.text_rect().h / self.m.line_h) as usize).max(1)
    }

    pub(crate) fn row_span(&self) -> Range<usize> {
        if self.m.line_h <= 0.0 {
            return 0..0;
        }
        let first = (self.sy() / self.m.line_h) as usize;
        let last = ((self.sy() + self.text_rect().h) / self.m.line_h).ceil() as usize;
        first.min(self.total_rows())..last.min(self.total_rows())
    }

    /// As linhas (0-based) entre a primeira e a última visíveis — as escondidas no meio incluídas.
    #[must_use]
    pub fn visible_lines(&self) -> Range<usize> {
        let r = self.row_span();
        if r.is_empty() {
            return 0..0;
        }
        self.line_of_row(r.start)..self.line_of_row(r.end - 1) + 1
    }

    fn max_scroll_y(&self) -> f32 {
        (self.total_rows() as f32 * self.m.line_h - self.text_rect().h).max(0.0)
    }

    pub(crate) fn clamp_scroll(&mut self) {
        self.scroll_y = ph2d_editor_core::math::safe_clamp(self.scroll_y, 0.0, self.max_scroll_y());
        self.scroll_x = self.scroll_x.max(0.0);
    }

    pub fn scroll_by(&mut self, dy_px: f32) {
        self.scroll_y += dy_px;
        self.clamp_scroll();
    }

    pub fn scroll_x_by(&mut self, dx_px: f32) {
        self.scroll_x += dx_px;
        self.clamp_scroll();
    }

    /// A linha pintada `line`, se o último quadro a pintou sobre ESTE texto.
    fn row(&self, buffer: &Buffer, line: usize) -> Option<&Row> {
        if buffer.revision() != self.revision {
            return None;
        }
        self.rows.iter().find(|r| r.line == line)
    }

    /// O x (relativo à coluna 0) do byte: o layout pintado, ou o avanço da mono se a linha não está
    /// na cache.
    fn x_in_line(&self, buffer: &Buffer, byte: usize) -> f32 {
        let t = buffer.text();
        let line = t.line_of_byte(byte);
        if let Some(r) = self.row(buffer, line)
            && let Some(layout) = self.cache.peek(r.key)
        {
            return geom::x_of_byte(layout, &r.dl, byte - r.start);
        }
        t.display_col(byte, self.tab.max(1)) as f32 * self.m.advance
    }

    /// O cursor nunca fica fora da vista: rola o mínimo para mostrar `byte`.
    pub fn ensure_visible(&mut self, buffer: &Buffer, byte: usize) {
        self.line_count = buffer.text().line_count();
        let byte = byte.min(buffer.text().len());
        let line = buffer.text().line_of_byte(byte);
        let y = self.row_of_line(line) as f32 * self.m.line_h;
        let h = self.text_rect().h;
        if y < self.scroll_y {
            self.scroll_y = y;
        } else if y + self.m.line_h > self.scroll_y + h {
            self.scroll_y = y + self.m.line_h - h;
        }
        let x = self.x_in_line(buffer, byte);
        let w = (self.text_rect().w - self.m.text_pad).max(0.0);
        let margin = Spacing::Xl.px().min(w * 0.5);
        if x < self.scroll_x + margin {
            self.scroll_x = x - margin;
        } else if x > self.scroll_x + w - margin {
            self.scroll_x = x - w + margin;
        }
        self.clamp_scroll();
    }

    /// O retângulo do cursor em `byte` (a janela de candidatos do IME); `None` fora da vista.
    #[must_use]
    pub fn caret_rect(&self, buffer: &Buffer, byte: usize) -> Option<Rect> {
        let byte = byte.min(buffer.text().len());
        let line = buffer.text().line_of_byte(byte);
        if self.is_hidden(line) {
            return None;
        }
        let tr = self.text_rect();
        let y = self.row_y(self.row_of_line(line));
        let x = self.text_x0() + self.x_in_line(buffer, byte);
        let visible = y + self.m.line_h > tr.y && y < tr.y + tr.h && x >= tr.x && x <= tr.x + tr.w;
        visible.then_some(Rect {
            x,
            y,
            w: self.m.caret_w,
            h: self.m.line_h,
        })
    }

    /// Onde caiu `(x, y)`.
    #[must_use]
    pub fn hit(&self, buffer: &Buffer, x: f32, y: f32) -> Hit {
        let a = self.area;
        if !a.contains(x, y) || self.m.line_h <= 0.0 {
            return Hit::Outside;
        }
        let tr = self.text_rect();
        if y >= tr.y + tr.h {
            return Hit::Status;
        }
        let row = (((y - tr.y + self.sy()) / self.m.line_h) as usize).min(self.total_rows() - 1);
        let line = self.line_of_row(row).min(buffer.text().line_count() - 1);
        if x < tr.x {
            let fold_x0 = a.x + self.m.gutter_w - self.m.fold_w;
            let is_fold = self
                .folds
                .iter()
                .any(|f| f.start == line && f.end > f.start);
            return if is_fold && x >= fold_x0 {
                Hit::Fold(line)
            } else {
                Hit::Gutter(line)
            };
        }
        let local = x - self.text_x0();
        if let Some(r) = self.row(buffer, line)
            && let Some(layout) = self.cache.peek(r.key)
        {
            return Hit::Text(r.start + geom::byte_at_x(layout, &r.dl, local, r.len));
        }
        let col = (local / self.m.advance + 0.5).max(0.0) as usize;
        Hit::Text(
            buffer
                .text()
                .byte_at_display_col(line, col, self.tab.max(1)),
        )
    }

    /// ⚠️ SONDA (testes/auditoria): as colunas do último quadro em coordenadas de ecrã, com o byte
    /// ABSOLUTO do buffer e o meio vertical da linha — lidas dos MESMOS layouts e origens que a
    /// pintura usou.
    #[must_use]
    pub fn probe_columns(&self) -> Vec<(Column, f32)> {
        let x0 = self.text_x0();
        let mut out = Vec::new();
        for r in &self.rows {
            let Some(layout) = self.cache.peek(r.key) else {
                continue;
            };
            let mid = r.y + self.m.line_h * 0.5;
            for c in geom::columns(layout, &r.dl) {
                out.push((
                    Column {
                        byte: r.start + c.byte,
                        x0: x0 + c.x0,
                        x1: x0 + c.x1,
                    },
                    mid,
                ));
            }
        }
        out
    }
}
