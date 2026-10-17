//! ⭐ **O balão do rato e a ajuda de assinatura** (CodeEditor Onda 2, I5) — o `Popover` da lista de
//! completação, com a mesma largura da caixa da documentação dela.
//!
//! - O balão fica por baixo da palavra (por cima se não couber).
//! - A assinatura fica por baixo da linha do cursor (o `put_callhint_tooltip_below_current_line` de
//!   fábrica do Godot, I0b §7); por cima quando a lista de completação está por baixo ou não cabe. O
//!   parâmetro activo vai marcado.
//! - Ambos mostram as linhas que cabem do lado escolhido: o tecto é a altura da zona do texto.

use crate::frame::{COMPLETION_MAX_CHARS, CodeFrame};
use crate::palette::Palette;
use crate::popup::{plain, wrap};
use crate::style::Ink;
use crate::view::CodeView;
use ph2d_a11y::{Node, NodeBuilder, NodeId, Role};
use ph2d_editor_core::paint::rect_to_vello;
use ph2d_editor_core::widget::{Popover, paint_popover, pop_popover_clip, push_popover_clip};
use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::Spacing;
use ph2d_vector::VectorScene;
use std::ops::Range;

/// O que um balão pintou no último quadro.
#[derive(Clone, Debug, PartialEq)]
pub struct HintProbe {
    pub rect: Rect,
    /// As linhas pintadas, de cima para baixo.
    pub lines: Vec<String>,
    /// O texto marcado (o parâmetro activo), por linha em que aparece.
    pub marked: Vec<String>,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct Painted {
    pub hover: Option<HintProbe>,
    pub signature: Option<HintProbe>,
}

/// As ids dos dois balões, depois das da lista ([`crate::popup`]) no bloco da aba.
const A11Y_HOVER: u64 = (1 << 31) | (1 << 30);
const A11Y_SIGNATURE: u64 = A11Y_HOVER + 1;

/// As linhas de `s` partidas nos espaços para caber em `cols` colunas, como faixas de bytes de `s`
/// (a marca do parâmetro activo segue-as).
fn wrap_ranges(s: &str, cols: usize) -> Vec<Range<usize>> {
    let mut out = Vec::new();
    let (mut start, mut n, mut last_space) = (0, 0, None);
    for (b, c) in s.char_indices() {
        if n >= cols
            && let Some(sp) = last_space
            && sp > start
        {
            out.push(start..sp);
            start = sp + 1;
            n = s[start..b].chars().count();
            last_space = None;
        }
        if c == ' ' {
            last_space = Some(b);
        }
        n += 1;
    }
    out.push(start..s.len());
    out
}

impl CodeView {
    pub(crate) fn paint_hints(
        &mut self,
        frame: &CodeFrame<'_>,
        scene: &mut VectorScene,
        ts: &mut TextSystem,
        pal: &Palette,
    ) {
        self.hints = Painted::default();
        if let Some(s) = &frame.signature {
            let head = frame.buffer.selection().primary().head;
            if let Some(at) = self.caret_rect(frame.buffer, head) {
                self.hints.signature = self.paint_signature(s, at, scene, ts, pal);
            }
        }
        if let Some(h) = &frame.hover
            && let Some(at) = self.caret_rect(frame.buffer, h.word.start)
        {
            let lines = wrap(&plain(h.text), self.hint_cols());
            let lines: Vec<(String, Option<Range<usize>>)> =
                lines.into_iter().map(|l| (l, None)).collect();
            self.hints.hover = self.paint_box(&lines, at, false, scene, ts, pal);
        }
    }

    /// As colunas de um balão: a caixa da documentação da lista, sem passar da zona do texto.
    fn hint_cols(&self) -> usize {
        let pad = Spacing::Xs.px();
        let room = ((self.text_rect().w - 2.0 * pad) / self.m.advance.max(1.0)).floor() as usize;
        COMPLETION_MAX_CHARS.min(room).max(1)
    }

    fn paint_signature(
        &mut self,
        s: &crate::frame::SignatureView<'_>,
        at: Rect,
        scene: &mut VectorScene,
        ts: &mut TextSystem,
        pal: &Palette,
    ) -> Option<HintProbe> {
        let cols = self.hint_cols();
        let mut lines: Vec<(String, Option<Range<usize>>)> = wrap_ranges(s.label, cols)
            .into_iter()
            .map(|r| {
                let mark = s.active.as_ref().and_then(|a| {
                    let (lo, hi) = (a.start.max(r.start), a.end.min(r.end));
                    (lo < hi).then(|| lo - r.start..hi - r.start)
                });
                (s.label[r].to_owned(), mark)
            })
            .collect();
        if let Some(doc) = s.doc.filter(|d| !d.trim().is_empty()) {
            lines.extend(wrap(&plain(doc), cols).into_iter().map(|l| (l, None)));
        }
        // A lista de completação por baixo da linha empurra a assinatura para cima; sem espaço lá
        // (a 1.ª linha à vista), a assinatura vai por baixo da lista.
        let list = self
            .popup
            .as_ref()
            .map(|p| p.rect)
            .filter(|r| r.y >= at.y + at.h);
        let room_above = at.y - self.text_rect().y >= self.m.line_h + 2.0 * Spacing::Xs.px();
        match list {
            Some(r) if !room_above => {
                let under = Rect::new(at.x, r.y, at.w, r.h);
                self.paint_box(&lines, under, false, scene, ts, pal)
            }
            l => self.paint_box(&lines, at, l.is_some(), scene, ts, pal),
        }
    }

    /// Um balão com `lines` (cada uma com a sua faixa marcada), por baixo de `at` ou por cima (`up`,
    /// ou quando por baixo não cabe nem uma linha).
    fn paint_box(
        &mut self,
        lines: &[(String, Option<Range<usize>>)],
        at: Rect,
        up: bool,
        scene: &mut VectorScene,
        ts: &mut TextSystem,
        pal: &Palette,
    ) -> Option<HintProbe> {
        let tr = self.text_rect();
        let pad = Spacing::Xs.px();
        let lh = self.m.line_h;
        if lines.is_empty() || lh <= 0.0 {
            return None;
        }
        let rows_in = |space: f32| ((space - 2.0 * pad) / lh).floor().max(0.0) as usize;
        let below = rows_in(tr.y + tr.h - (at.y + at.h));
        let above = rows_in(at.y - tr.y);
        let up = (up && above > 0)
            || (below == 0 && above > 0)
            || (below < lines.len() && above > below);
        let rows = lines.len().min(if up { above } else { below });
        if rows == 0 {
            return None;
        }
        let cols = lines
            .iter()
            .take(rows)
            .map(|l| l.0.chars().count())
            .max()
            .unwrap_or(1)
            .max(1);
        let w = (cols as f32 * self.m.advance + 2.0 * pad).min(tr.w);
        let h = rows as f32 * lh + 2.0 * pad;
        let x = (at.x - pad).min(tr.x + tr.w - w).max(tr.x);
        let y = if up { at.y - h } else { at.y + at.h };
        let rect = Rect::new(x, y, w, h);
        paint_popover(&Popover::new(NodeId(0)), rect, scene, pal.theme);
        push_popover_clip(rect, scene);
        let mut probe = HintProbe {
            rect,
            lines: Vec::new(),
            marked: Vec::new(),
        };
        for (k, (l, mark)) in lines.iter().take(rows).enumerate() {
            let ly = y + pad + k as f32 * lh;
            let x0 = x + pad;
            if let Some(m) = mark {
                let a = l[..m.start].chars().count() as f32;
                let n = l[m.clone()].chars().count() as f32;
                scene.fill_rect(
                    rect_to_vello(Rect::new(
                        x0 + a * self.m.advance,
                        ly,
                        n * self.m.advance,
                        lh,
                    )),
                    pal.match_,
                );
                probe.marked.push(l[m.clone()].to_owned());
            }
            self.paint_mono(ts, scene, pal, l, Ink::Plain, x0, ly, x + w - pad);
            probe.lines.push(l.clone());
        }
        pop_popover_clip(scene);
        Some(probe)
    }

    /// O rato está sobre o balão do último quadro (ele não se fecha enquanto se lê).
    #[must_use]
    pub fn hint_hover_contains(&self, x: f32, y: f32) -> bool {
        self.hints
            .hover
            .as_ref()
            .is_some_and(|h| h.rect.contains(x, y))
    }

    /// ⚠️ SONDA: o balão do rato e a assinatura que o último quadro pintou.
    #[must_use]
    pub fn hints_painted(&self) -> (Option<HintProbe>, Option<HintProbe>) {
        (self.hints.hover.clone(), self.hints.signature.clone())
    }

    /// Os dois balões ao leitor de ecrã: `Tooltip`s filhos do `Document`; o da assinatura diz o
    /// parâmetro activo no valor.
    pub(crate) fn hints_a11y(&self, base: NodeId) -> Vec<(NodeId, Node)> {
        let node = |p: &HintProbe, label: String| {
            let r = p.rect;
            let mut b = NodeBuilder::new(Role::Tooltip).label(label).bounds(
                f64::from(r.x),
                f64::from(r.y),
                f64::from(r.w),
                f64::from(r.h),
            );
            if !p.marked.is_empty() {
                b = b.value(p.marked.concat());
            }
            b.build()
        };
        let mut out = Vec::new();
        if let Some(p) = &self.hints.hover {
            out.push((NodeId(base.0 + A11Y_HOVER), node(p, p.lines.join("\n"))));
        }
        if let Some(p) = &self.hints.signature {
            out.push((NodeId(base.0 + A11Y_SIGNATURE), node(p, p.lines.join(" "))));
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_assinatura_parte_nos_espacos_sem_perder_bytes() {
        let s = "function ph2d.set(entity: number, field: string, value: number): ()";
        let r = wrap_ranges(s, 20);
        assert!(
            r.iter().all(|r| s[r.clone()].chars().count() <= 20),
            "{r:?}"
        );
        let junto: Vec<&str> = r.iter().map(|r| &s[r.clone()]).collect();
        assert_eq!(junto.join(" "), s);
        let curto = wrap_ranges("curto", 20);
        assert_eq!((curto.len(), curto[0].start, curto[0].end), (1, 0, 5));
    }
}
