//! ⭐ **A pintura de um quadro** (E4): só as linhas VISÍVEIS, moldadas pela [`LineCache`]; por baixo
//! do texto a linha corrente, as ocorrências, a seleção e o par; por cima o sublinhado do culpado,
//! a pré-edição e os cursores; a calha e a barra de estado à parte.
//!
//! [`LineCache`]: crate::line_cache::LineCache

use crate::a11y::{clipped, selection};
use crate::display::DisplayLine;
use crate::frame::{CodeFrame, Severity};
use crate::geom;
use crate::glyphs::add_work;
use crate::metrics::Metrics;
use crate::palette::Palette;
use crate::style::Ink;
use crate::view::{CodeView, Row, Underline};
use ph2d_a11y::{Node, NodeBuilder, NodeId, Role};
use ph2d_code_buffer::Buffer;
use ph2d_editor_core::math::safe_clamp_ord;
use ph2d_editor_core::paint::{
    fill_rounded_rect, frame_radius, label_column_width, paint_text, rect_to_vello,
    stroke_polyline, stroke_rect,
};
use ph2d_editor_core::ui_scale::ao_pixel;
use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::{Radius, Spacing, StrokeToken, Theme, TypeToken};
use ph2d_vector::VectorScene;
use std::ops::Range;

impl CodeView {
    /// Pinta o documento em `area` (o fundo também).
    pub fn paint(
        &mut self,
        mut frame: CodeFrame<'_>,
        area: Rect,
        scene: &mut VectorScene,
        ts: &mut TextSystem,
        theme: &Theme,
    ) {
        let pal = Palette::new(*theme);
        self.prepare(&frame, area, ts);
        self.cache.begin_frame();
        self.build_rows(&mut frame, ts);
        let tr = self.text_rect();
        scene.fill_rect(rect_to_vello(area), pal.bg);
        scene.fill_rect(
            rect_to_vello(Rect {
                x: area.x,
                y: area.y,
                w: self.m.gutter_w.min(area.w),
                h: tr.h,
            }),
            pal.gutter,
        );
        scene.push_clip(&rect_to_vello(tr));
        self.paint_under(&frame, scene, &pal);
        self.paint_glyphs(scene, &pal);
        self.paint_problems(&frame, scene, &pal);
        self.paint_fold_pills(scene, ts, &pal);
        self.paint_carets(&frame, scene, &pal);
        scene.pop_layer();
        self.paint_gutter(&frame, scene, &pal);
        self.paint_completion(&frame, scene, ts, &pal);
        self.paint_hints(&frame, scene, ts, &pal);
        self.paint_rename(&frame, scene, ts, &pal);
        let head = frame.buffer.selection().primary().head;
        let k = (frame.buffer.revision(), head, self.tab);
        if self.status_col.is_none_or(|c| c.0 != k) {
            self.status_col = Some((k, frame.buffer.text().display_col(head, self.tab)));
        }
        crate::status::paint(self, &frame, scene, ts, &pal);
        self.cache.end_frame();
    }

    fn prepare(&mut self, frame: &CodeFrame<'_>, area: Rect, ts: &mut TextSystem) {
        let t = frame.buffer.text();
        self.line_count = t.line_count();
        self.revision = frame.buffer.revision();
        self.tab = frame.tab.max(1);
        let want = ph2d_text::displayed_font_px(TypeToken::Base.px());
        let digits = crate::metrics::count_digits(self.line_count).max(2);
        if self.digits.is_none() || self.m.font_px != want || self.m.digits != digits {
            let (m, d) = Metrics::measure(&mut self.cache, ts, self.line_count);
            self.m = m;
            self.digits = d;
        }
        self.area = area;
        self.set_hidden(frame.closed, self.line_count);
        self.clamp_scroll();
    }

    /// Molda (ou acha na cache) cada linha visível; os trechos de cor pedem-se por bloco contíguo.
    fn build_rows(&mut self, frame: &mut CodeFrame<'_>, ts: &mut TextSystem) {
        let buffer = frame.buffer;
        let t = buffer.text();
        let span = self.row_span();
        let pre = frame
            .preedit
            .map(|s| (buffer.selection().primary().head, s));
        let inlays = frame.inlays;
        self.inlay_cols.clear();
        self.inlay_cols
            .extend(inlays.iter().map(|i| (i.at, i.label.chars().count())));
        let mut rows = std::mem::take(&mut self.rows);
        rows.clear();
        self.folds.clear();
        let cap = self.render_cap.max(1);
        let mut r = span.start;
        while r < span.end {
            let first = self.line_of_row(r);
            let mut n = 1;
            while r + n < span.end && self.line_of_row(r + n) == first + n {
                n += 1;
            }
            self.folds.extend((frame.folds)(first..first + n));
            // Cada fila mostra a linha até ao tecto (`render_cap`); os trechos de cor pedem-se por
            // bloco de bytes contíguos — uma linha cortada fecha o bloco dela.
            let mut i = 0;
            while i < n {
                let mut block: Vec<(usize, usize, usize, usize)> = Vec::new();
                while i < n {
                    let line = first + i;
                    let (ls, le) = (t.byte_of_line(line), t.line_end(line));
                    let shown = if le - ls > cap {
                        t.floor_char(ls + cap)
                    } else {
                        le
                    };
                    block.push((r + i, ls, le, shown));
                    i += 1;
                    if shown < le {
                        break;
                    }
                }
                let (bs, be) = (block[0].1, block[block.len() - 1].3);
                let mut spans = (frame.spans)(bs..be);
                spans.sort_by_key(|s| s.0.start);
                let mut k = 0;
                for (row, ls, le, shown) in block {
                    let line = self.line_of_row(row);
                    let pre_here = pre
                        .filter(|(h, _)| (ls..=shown).contains(h))
                        .map(|(h, s)| (h - ls, s));
                    let a = inlays.partition_point(|i| i.at < ls);
                    let b = inlays.partition_point(|i| i.at <= shown);
                    let here: Vec<(usize, &str)> = inlays[a..b.max(a)]
                        .iter()
                        .map(|i| (i.at - ls, i.label.as_str()))
                        .collect();
                    let dl = DisplayLine::build(&t.slice(ls..shown), self.tab, pre_here, &here);
                    add_work(dl.text.len(), 0);
                    while k < spans.len() && spans[k].0.end <= ls {
                        k += 1;
                    }
                    let ds = display_spans(&spans[k..], ls, shown, &dl);
                    let key = self.cache.touch(ts, &dl.text, &ds, self.m.font_px);
                    rows.push(Row {
                        line,
                        start: ls,
                        len: shown - ls,
                        cut: shown < le,
                        y: self.row_y(row),
                        key,
                        dl,
                    });
                }
            }
            r += n;
        }
        self.rows = rows;
    }

    /// As filas pintadas cujas linhas caem em `lines` (as filas estão por ordem de linha).
    pub(crate) fn rows_in(&self, lines: Range<usize>) -> &[Row] {
        let a = self.rows.partition_point(|r| r.line < lines.start);
        let b = self.rows.partition_point(|r| r.line < lines.end);
        &self.rows[a..b.max(a)]
    }

    pub(crate) fn row_of(&self, line: usize) -> Option<&Row> {
        self.rows_in(line..line + 1).first()
    }

    /// O retângulo dos bytes `s..e` na fila; `eol` estica-o meia coluna quando a faixa passa do fim.
    fn seg_rect(&self, row: &Row, s: usize, e: usize, eol: bool) -> Option<Rect> {
        let (rs, re) = (row.start, row.start + row.len);
        if e < rs || s > re {
            return None;
        }
        let layout = self.cache.peek(row.key)?;
        let x = self.text_x0();
        let x0 = x + geom::x_of_byte_after(layout, &row.dl, s.max(rs) - rs);
        let mut x1 = x + geom::x_of_byte(layout, &row.dl, e.min(re) - rs);
        if eol && e > re {
            x1 += self.m.advance * 0.5;
        }
        (x1 > x0).then_some(Rect {
            x: x0,
            y: row.y,
            w: x1 - x0,
            h: self.m.line_h,
        })
    }

    /// Cada fila tocada por `s..e`, com o retângulo dela.
    fn each_seg(
        &self,
        buffer: &Buffer,
        s: usize,
        e: usize,
        eol: bool,
        mut f: impl FnMut(&Row, Rect),
    ) {
        let t = buffer.text();
        let lines = t.line_of_byte(s)..t.line_of_byte(e) + 1;
        for row in self.rows_in(lines) {
            if let Some(rect) = self.seg_rect(row, s, e, eol) {
                f(row, rect);
            }
        }
    }

    fn paint_under(&self, frame: &CodeFrame<'_>, scene: &mut VectorScene, pal: &Palette) {
        let buffer = frame.buffer;
        let t = buffer.text();
        let tr = self.text_rect();
        let primary = buffer.selection().primary();
        if let Some(row) = self.row_of(t.line_of_byte(primary.head)) {
            scene.fill_rect(
                rect_to_vello(Rect {
                    x: tr.x,
                    y: row.y,
                    w: tr.w,
                    h: self.m.line_h,
                }),
                pal.current_line,
            );
        }
        let (vs, ve) = self.visible_bytes(buffer);
        for (i, m) in frame.matches.iter().enumerate() {
            if m.end < vs || m.start > ve {
                continue;
            }
            let current = frame.current_match == Some(i);
            self.each_seg(buffer, m.start, m.end, false, |_, r| {
                scene.fill_rect(
                    rect_to_vello(r),
                    if current {
                        pal.match_current
                    } else {
                        pal.match_
                    },
                );
                if current {
                    stroke_rect(scene, r, StrokeToken::Thin.px(), pal.match_edge);
                }
            });
        }
        for o in frame.occurrences {
            if o.end >= vs && o.start <= ve {
                self.each_seg(buffer, o.start, o.end, false, |_, r| {
                    scene.fill_rect(rect_to_vello(r), pal.occurrence);
                });
            }
        }
        // As dicas de tipo (I9) assentam numa pastilha própria: o olho separa-as do código.
        let x = self.text_x0();
        let radius = frame_radius(pal.theme, Radius::Sm.px());
        for row in &self.rows {
            let Some(layout) = self.cache.peek(row.key) else {
                continue;
            };
            for (d, n) in row.dl.inlays() {
                let (x0, x1) = (geom::x_of_disp(layout, d), geom::x_of_disp(layout, d + n));
                let r = Rect {
                    x: x + x0,
                    y: row.y + Spacing::Xxs.px(),
                    w: x1 - x0,
                    h: self.m.line_h - Spacing::Xxs.px() * 2.0,
                };
                fill_rounded_rect(scene, r, radius, pal.pill);
            }
        }
        // Os buracos do modelo que esperam o Tab (I12): uma caixa fina, como no VS Code.
        for s in frame.stops {
            if s.end >= vs && s.start <= ve {
                self.each_seg(buffer, s.start, s.end, false, |_, r| {
                    stroke_rect(scene, r, StrokeToken::Thin.px(), pal.bracket);
                });
            }
        }
        for r in buffer.selection().ranges() {
            if !r.is_empty() && r.to() >= vs && r.from() <= ve {
                self.each_seg(buffer, r.from(), r.to(), true, |_, rect| {
                    scene.fill_rect(rect_to_vello(rect), pal.selection)
                });
            }
        }
        if let Some((a, b)) = frame.bracket {
            for p in [a, b] {
                let next = t.next_grapheme(p);
                self.each_seg(buffer, p, next, false, |_, r| {
                    stroke_rect(scene, r, StrokeToken::Thin.px(), pal.bracket)
                });
            }
        }
    }

    /// Bytes do início da primeira à fim da última linha pintada.
    fn visible_bytes(&self, buffer: &Buffer) -> (usize, usize) {
        match (self.rows.first(), self.rows.last()) {
            (Some(a), Some(b)) => (a.start, b.start + b.len),
            _ => (0, buffer.text().len()),
        }
    }

    /// O símbolo culpado sublinhado (ondulado) no lugar EXACTO de cada problema.
    fn paint_problems(&mut self, frame: &CodeFrame<'_>, scene: &mut VectorScene, pal: &Palette) {
        let mut out = std::mem::take(&mut self.underlines);
        out.clear();
        let amp = StrokeToken::Thick.px();
        for p in frame.problems {
            let Some(c) = p.culprit.clone() else { continue };
            let color = pal.severity(p.severity);
            let mut push = |row: &Row, mut r: Rect| {
                if r.w <= 0.0 {
                    r.w = self.m.advance;
                }
                let base = (row.y + self.m.text_dy + self.m.baseline + Spacing::Xxs.px())
                    .min(row.y + self.m.line_h - amp);
                let base = ao_pixel(base);
                let pts = wave(r.x, r.x + r.w, base, amp);
                stroke_polyline(scene, &pts, StrokeToken::Thin.px(), color);
                out.push(Underline {
                    line: row.line,
                    x0: r.x,
                    x1: r.x + r.w,
                    y0: base,
                    y1: base + amp,
                    severity: p.severity,
                });
            };
            if c.is_empty() {
                let line = frame.buffer.text().line_of_byte(c.start);
                if let Some(row) = self.row_of(line)
                    && let Some(layout) = self.cache.peek(row.key)
                {
                    let x = self.text_x0() + geom::x_of_byte(layout, &row.dl, c.start - row.start);
                    push(
                        row,
                        Rect {
                            x,
                            y: row.y,
                            w: 0.0,
                            h: self.m.line_h,
                        },
                    );
                }
            } else {
                self.each_seg(frame.buffer, c.start, c.end, false, |row, r| push(row, r));
            }
        }
        self.underlines = out;
    }

    /// O «…» no fim de cada linha com dobra fechada.
    fn paint_fold_pills(&self, scene: &mut VectorScene, ts: &mut TextSystem, pal: &Palette) {
        let label = ph2d_i18n::tr("code.fold.ellipsis");
        let font = ph2d_text::displayed_font_px(TypeToken::Sm.px());
        let pad = Spacing::Xs.px();
        let w = label_column_width(ts, font, [label]) + pad * 2.0;
        // O «…» também no fim de uma linha mostrada só até ao tecto (`render_cap`).
        let folded = self
            .folds
            .iter()
            .filter(|f| f.folded && f.end > f.start)
            .map(|f| f.start);
        let cut = self.rows.iter().filter(|r| r.cut).map(|r| r.line);
        for line in folded.chain(cut) {
            let Some(row) = self.row_of(line) else {
                continue;
            };
            let Some(layout) = self.cache.peek(row.key) else {
                continue;
            };
            let x = self.text_x0() + geom::width(layout) + Spacing::Sm.px();
            let pill = Rect {
                x,
                y: row.y + Spacing::Xxs.px(),
                w,
                h: self.m.line_h - Spacing::Xxs.px() * 2.0,
            };
            fill_rounded_rect(
                scene,
                pill,
                frame_radius(pal.theme, Radius::Sm.px()),
                pal.pill,
            );
            paint_text(
                ts,
                scene,
                label,
                x + pad,
                row.y + (self.m.line_h - font) * 0.5,
                font,
                w,
                pal.pill_text,
            );
        }
    }

    /// Os cursores (na fase acesa) e o sublinhado da pré-edição no cursor primário.
    fn paint_carets(&self, frame: &CodeFrame<'_>, scene: &mut VectorScene, pal: &Palette) {
        let buffer = frame.buffer;
        let t = buffer.text();
        let sel = buffer.selection();
        let x = self.text_x0();
        for (i, r) in sel.ranges().iter().enumerate() {
            let primary = i == sel.primary_index();
            let Some(row) = self.row_of(t.line_of_byte(r.head)) else {
                continue;
            };
            let Some(layout) = self.cache.peek(row.key) else {
                continue;
            };
            let pre = primary.then(|| row.dl.preedit_span()).flatten();
            let cx = match pre {
                Some((d, n)) => {
                    let (x0, x1) = (geom::x_of_disp(layout, d), geom::x_of_disp(layout, d + n));
                    let y = ao_pixel(row.y + self.m.text_dy + self.m.baseline + Spacing::Xxs.px());
                    scene.fill_rect(
                        rect_to_vello(Rect {
                            x: x + x0,
                            y,
                            w: x1 - x0,
                            h: StrokeToken::Thin.px(),
                        }),
                        pal.caret,
                    );
                    x1
                }
                None => geom::x_of_byte(layout, &row.dl, r.head - row.start),
            };
            if frame.caret_on {
                let c = Rect {
                    x: ao_pixel(x + cx),
                    y: row.y,
                    w: self.m.caret_w,
                    h: self.m.line_h,
                };
                scene.fill_rect(
                    rect_to_vello(c),
                    if primary { pal.caret } else { pal.caret_other },
                );
            }
        }
    }
}

impl CodeView {
    /// O id do `TextRun` da linha `line`, com `base` o id do `Document`.
    #[must_use]
    pub fn a11y_run_id(base: NodeId, line: usize) -> NodeId {
        NodeId(base.0 + 1 + line as u64)
    }

    /// Os nós do último quadro: o `Document` primeiro (filhos = as linhas visíveis), depois um
    /// `TextRun` por linha. A família reserva os ids `base ..= base + nº de linhas`.
    #[must_use]
    pub fn a11y_nodes(&self, buffer: &Buffer, base: NodeId) -> Vec<(NodeId, Node)> {
        let t = buffer.text();
        let tr = self.text_rect();
        let lines: Vec<usize> = self
            .rows
            .iter()
            .map(|r| r.line)
            .filter(|&l| l < t.line_count())
            .collect();
        let mut out = Vec::with_capacity(lines.len() + 1);
        let mut runs = Vec::with_capacity(lines.len());
        for r in self.rows.iter().filter(|r| r.line < t.line_count()) {
            let (value, lens) = clipped(&crate::a11y::run_text(t, r.line));
            let id = Self::a11y_run_id(base, r.line);
            let mut node = NodeBuilder::new(Role::TextRun)
                .bounds(
                    f64::from(tr.x),
                    f64::from(r.y),
                    f64::from(tr.w),
                    f64::from(self.m.line_h),
                )
                .build();
            node.set_value(value);
            node.set_character_lengths(lens);
            runs.push((id, node));
        }
        let mut doc = NodeBuilder::new(Role::Document)
            .label(ph2d_i18n::tr("code.a11y.document"))
            .bounds(
                f64::from(tr.x),
                f64::from(tr.y),
                f64::from(tr.w),
                f64::from(tr.h),
            )
            .children(lines.iter().map(|&l| Self::a11y_run_id(base, l)))
            .build();
        if let Some(sel) = selection(buffer, &lines, base) {
            doc.set_text_selection(sel);
        }
        let list = self.completion_a11y(base);
        if let Some((id, active, _)) = &list {
            doc.push_child((*id).into());
            if let Some(a) = active {
                doc.set_active_descendant((*a).into());
            }
        }
        let (roots, hints) = self.hints_a11y(base);
        for id in roots {
            doc.push_child(id.into());
        }
        let rename = self.rename_a11y(base);
        if let Some((id, _)) = &rename {
            doc.push_child((*id).into());
        }
        out.push((base, doc));
        out.extend(runs);
        out.extend(list.into_iter().flat_map(|l| l.2));
        out.extend(hints);
        out.extend(rename);
        out
    }
}

/// Os trechos de cor de uma linha, em bytes MOSTRADOS (as tabs esticam os trechos que as cobrem).
fn display_spans(
    spans: &[(Range<usize>, Ink)],
    ls: usize,
    le: usize,
    dl: &DisplayLine,
) -> Vec<(Range<usize>, Ink)> {
    let mut out = Vec::new();
    for (rg, ink) in spans.iter().take_while(|s| s.0.start < le.max(ls + 1)) {
        let s = safe_clamp_ord(rg.start, ls, le) - ls;
        let e = safe_clamp_ord(rg.end, ls, le) - ls;
        if s < e {
            out.push((dl.to_disp_after(s)..dl.to_disp(e), *ink));
        }
    }
    // A dica pinta-se na tinta dela, nunca na do código à volta.
    out.extend(dl.inlays().map(|(d, n)| (d..d + n, Ink::Hint)));
    out.sort_by_key(|s| s.0.start);
    out
}

/// O ziguezague do sublinhado: meia onda de `amp` de largura.
fn wave(x0: f32, x1: f32, y: f32, amp: f32) -> Vec<(f32, f32)> {
    let mut pts = Vec::new();
    let mut x = x0;
    let mut up = false;
    while x < x1 {
        pts.push((x, if up { y } else { y + amp }));
        up = !up;
        x += amp;
    }
    pts.push((x1, if up { y } else { y + amp }));
    pts
}

impl Palette {
    pub(crate) fn severity(&self, s: Severity) -> ph2d_vector::Color {
        match s {
            Severity::Error => self.error,
            Severity::Warning => self.warn,
            Severity::Info => self.info,
        }
    }
}
