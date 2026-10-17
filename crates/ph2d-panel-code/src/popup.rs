//! ⭐ **A janelinha da lista de completação** (CodeEditor Onda 2, I4): por baixo da palavra (por cima
//! se não couber), sete linhas no máximo, as letras que casaram realçadas, o tipo à direita e a
//! documentação da seleccionada numa caixa ao lado. A superfície é o `Popover` do design system.

use crate::frame::{COMPLETION_MAX_CHARS, COMPLETION_ROWS, CodeFrame, CompletionView};
use crate::palette::Palette;
use crate::style::Ink;
use crate::view::CodeView;
use ph2d_a11y::{Node, NodeBuilder, NodeId, Role};
use ph2d_editor_core::math::safe_clamp_ord;
use ph2d_editor_core::paint::rect_to_vello;
use ph2d_editor_core::widget::{Popover, paint_popover, pop_popover_clip, push_popover_clip};
use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::Spacing;
use ph2d_vector::VectorScene;

/// O que a lista pintou no último quadro (o clique, a roda e o leitor de ecrã lêem-no).
#[derive(Clone, Debug)]
pub(crate) struct Painted {
    pub rect: Rect,
    /// Cada linha à vista: o índice na lista inteira, a caixa, o rótulo e se é a seleccionada.
    pub rows: Vec<(usize, Rect, String, bool)>,
    pub doc: Option<Rect>,
}

/// Uma linha pintada da lista, como a sonda a dá: o índice, o rótulo, se é a seleccionada e o centro.
pub type CompletionPaintedRow = (usize, String, bool, (f32, f32));

/// A `ListBox`, a opção activa e os nós de todas as opções.
type ListA11y = (NodeId, Option<NodeId>, Vec<(NodeId, Node)>);

/// Onde caiu um clique na lista.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompletionHit {
    /// Na linha deste índice (da lista inteira).
    Row(usize),
    /// Na caixa, fora de uma linha (a caixa da documentação, a borda).
    Body,
}

/// A distância da id da lista ao `Document`, dentro do bloco de 2³² ids da aba e acima de qualquer
/// linha que um `TextRun` use.
const A11Y_LIST: u64 = 1 << 31;

impl CodeView {
    fn pad(&self) -> f32 {
        Spacing::Xs.px()
    }

    pub(crate) fn paint_completion(
        &mut self,
        frame: &CodeFrame<'_>,
        scene: &mut VectorScene,
        ts: &mut TextSystem,
        pal: &Palette,
    ) {
        self.popup = None;
        let Some(c) = &frame.completion else { return };
        if c.rows.is_empty() {
            return;
        }
        let Some(at) = self.caret_rect(frame.buffer, c.anchor) else {
            return;
        };
        let tr = self.text_rect();
        let pad = self.pad();
        let lh = self.m.line_h;
        let cols = safe_clamp_ord(c.width_chars, 1, COMPLETION_MAX_CHARS) as f32;
        let w = (cols * self.m.advance + 2.0 * pad).min(tr.w);
        let h = c.rows.len().min(COMPLETION_ROWS) as f32 * lh + 2.0 * pad;
        let x = (at.x - pad).min(tr.x + tr.w - w).max(tr.x);
        let below = at.y + at.h;
        let y = if below + h <= tr.y + tr.h || at.y - h < tr.y {
            below
        } else {
            at.y - h
        };
        let rect = Rect::new(x, y, w, h);
        paint_popover(&Popover::new(NodeId(0)), rect, scene, pal.theme);
        push_popover_clip(rect, scene);
        let mut painted = Painted {
            rect,
            rows: Vec::new(),
            doc: None,
        };
        for (k, row) in c.rows.iter().take(COMPLETION_ROWS).enumerate() {
            let i = c.top + k;
            let r = Rect::new(x, y + pad + k as f32 * lh, w, lh);
            let selected = i == c.selected;
            if selected {
                ph2d_editor_core::widget::paint_popover_row_highlight(scene, r, pal.theme);
            }
            let x0 = x + pad;
            for m in &row.matched {
                let a = row.label[..m.start].chars().count() as f32;
                let n = row.label[m.clone()].chars().count() as f32;
                scene.fill_rect(
                    rect_to_vello(Rect::new(
                        x0 + a * self.m.advance,
                        r.y,
                        n * self.m.advance,
                        lh,
                    )),
                    pal.match_,
                );
            }
            let right = x + w - pad;
            let label = self.paint_mono(ts, scene, pal, &row.label, row.ink, x0, r.y, right);
            if let Some(d) = &row.detail {
                let room = ((right - label) / self.m.advance).floor() as usize;
                if room > 2 {
                    let d = elide(d, room - 1);
                    let dw = d.chars().count() as f32 * self.m.advance;
                    self.paint_mono(ts, scene, pal, &d, Ink::Comment, right - dw, r.y, right);
                }
            }
            painted.rows.push((i, r, row.label.clone(), selected));
        }
        pop_popover_clip(scene);
        if let Some(doc) = c.doc.filter(|d| !d.trim().is_empty()) {
            painted.doc = self.paint_doc(scene, ts, pal, doc, rect, tr);
        }
        self.popup = Some(painted);
    }

    /// A documentação da seleccionada ao lado da lista (à direita; à esquerda se não couber), no
    /// máximo tão alta quanto a lista mais alta.
    fn paint_doc(
        &mut self,
        scene: &mut VectorScene,
        ts: &mut TextSystem,
        pal: &Palette,
        doc: &str,
        list: Rect,
        tr: Rect,
    ) -> Option<Rect> {
        let pad = self.pad();
        let lh = self.m.line_h;
        let cols = COMPLETION_MAX_CHARS;
        let w = cols as f32 * self.m.advance + 2.0 * pad;
        let x = if list.x + list.w + w <= tr.x + tr.w {
            list.x + list.w
        } else if list.x - w >= tr.x {
            list.x - w
        } else {
            return None;
        };
        let mut lines = wrap(&plain(doc), cols);
        if lines.len() > COMPLETION_ROWS {
            lines.truncate(COMPLETION_ROWS);
            if let Some(l) = lines.last_mut() {
                *l = elide(&format!("{l} …"), cols);
            }
        }
        let h = lines.len() as f32 * lh + 2.0 * pad;
        let r = Rect::new(x, list.y, w, h);
        paint_popover(&Popover::new(NodeId(0)), r, scene, pal.theme);
        push_popover_clip(r, scene);
        for (k, l) in lines.iter().enumerate() {
            let y = r.y + pad + k as f32 * lh;
            self.paint_mono(ts, scene, pal, l, Ink::Plain, x + pad, y, x + w - pad);
        }
        pop_popover_clip(scene);
        Some(r)
    }

    /// Onde caiu `(x, y)` na lista do último quadro (`None` = fora dela).
    #[must_use]
    pub fn completion_hit(&self, x: f32, y: f32) -> Option<CompletionHit> {
        let p = self.popup.as_ref()?;
        if let Some((i, ..)) = p.rows.iter().find(|r| r.1.contains(x, y)) {
            return Some(CompletionHit::Row(*i));
        }
        (p.rect.contains(x, y) || p.doc.is_some_and(|d| d.contains(x, y)))
            .then_some(CompletionHit::Body)
    }

    /// ⚠️ SONDA: as linhas que a lista pintou no último quadro (índice, rótulo, seleccionada, centro).
    #[must_use]
    pub fn completion_painted(&self) -> Option<Vec<CompletionPaintedRow>> {
        self.popup.as_ref().map(|p| {
            p.rows
                .iter()
                .map(|r| {
                    (
                        r.0,
                        r.2.clone(),
                        r.3,
                        (r.1.x + r.1.w * 0.5, r.1.y + r.1.h * 0.5),
                    )
                })
                .collect()
        })
    }

    /// Os nós da lista aberta: a `ListBox` e uma opção por linha à vista; a id da seleccionada é o
    /// `active_descendant` do `Document`.
    pub(crate) fn completion_a11y(&self, base: NodeId) -> Option<ListA11y> {
        let p = self.popup.as_ref()?;
        let list = NodeId(base.0 + A11Y_LIST);
        let opt = |i: usize| NodeId(list.0 + 1 + i as u64);
        let b = |r: Rect| {
            (
                f64::from(r.x),
                f64::from(r.y),
                f64::from(r.w),
                f64::from(r.h),
            )
        };
        let (x, y, w, h) = b(p.rect);
        let mut out = vec![(
            list,
            NodeBuilder::new(Role::ListBox)
                .label(ph2d_i18n::tr("code.a11y.completion"))
                .bounds(x, y, w, h)
                .children(p.rows.iter().map(|r| opt(r.0)))
                .build(),
        )];
        let mut active = None;
        for (i, r, label, selected) in &p.rows {
            let (x, y, w, h) = b(*r);
            let mut n = NodeBuilder::new(Role::ListBoxOption)
                .label(label.clone())
                .bounds(x, y, w, h)
                .build();
            n.set_selected(*selected);
            if *selected {
                active = Some(opt(*i));
            }
            out.push((opt(*i), n));
        }
        Some((list, active, out))
    }
}

/// `s` em no máximo `cols` colunas, com «…» no fim se foi cortado.
pub(crate) fn elide(s: &str, cols: usize) -> String {
    if s.chars().count() <= cols {
        return s.to_owned();
    }
    let mut out: String = s.chars().take(cols.saturating_sub(1)).collect();
    out.push('…');
    out
}

/// O texto de uma documentação em Markdown, sem a marcação que a mono mostraria crua (a régua
/// `----------` do `luau-lsp` vira uma linha em branco).
pub(crate) fn plain(md: &str) -> String {
    md.lines()
        .filter(|l| !l.trim_start().starts_with("```"))
        .map(|l| {
            let t = l.trim();
            if t.len() >= 3 && t.chars().all(|c| c == '-') {
                String::new()
            } else {
                l.replace("**", "").replace('`', "")
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// As linhas de `s` partidas nos espaços para caber em `cols` colunas.
pub(crate) fn wrap(s: &str, cols: usize) -> Vec<String> {
    let mut out = Vec::new();
    for para in s.lines() {
        let mut line = String::new();
        for word in para.split_whitespace() {
            let n = line.chars().count();
            if n > 0 && n + 1 + word.chars().count() > cols {
                out.push(std::mem::take(&mut line));
            }
            if !line.is_empty() {
                line.push(' ');
            }
            line.push_str(word);
        }
        out.push(line);
    }
    while out.last().is_some_and(String::is_empty) {
        out.pop();
    }
    out
}

impl CompletionView<'_> {
    /// A largura em colunas de uma lista: o rótulo mais longo e o tipo, com duas de folga.
    #[must_use]
    pub fn width_of<'a>(rows: impl IntoIterator<Item = (&'a str, Option<&'a str>)>) -> usize {
        rows.into_iter()
            .map(|(l, d)| l.chars().count() + d.map_or(0, |d| d.chars().count() + 2))
            .max()
            .unwrap_or(0)
            .min(COMPLETION_MAX_CHARS)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_documentacao_parte_nos_espacos() {
        assert_eq!(wrap("aa bb cc", 5), ["aa bb", "cc"]);
        assert_eq!(wrap("um\n\ndois", 10), ["um", "", "dois"]);
        assert_eq!(plain("**Set** a `field`\n```lua\nx\n```"), "Set a field\nx");
        assert_eq!(plain("```luau\nf()\n```\n----------\ndoc"), "f()\n\ndoc");
        assert_eq!(elide("abcdef", 4), "abc…");
    }
}
