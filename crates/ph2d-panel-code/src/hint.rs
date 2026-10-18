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
use ph2d_editor_core::widget::{
    Button, ButtonState, Popover, paint_button, paint_popover, pop_popover_clip, push_popover_clip,
};
use ph2d_editor_core::zones::Rect;
use ph2d_text::{FontWeight, TextSystem};
use ph2d_tokens::{ROW_H_PX, Spacing, control_gap_px};
use ph2d_vector::VectorScene;
use std::ops::Range;

/// Os botões das correcções de um balão e o que está sob o rato.
#[derive(Default)]
struct Fixes<'a> {
    labels: &'a [&'a str],
    hot: Option<usize>,
}

/// O que um balão pintou no último quadro.
#[derive(Clone, Debug, PartialEq)]
pub struct HintProbe {
    pub rect: Rect,
    /// As linhas pintadas, de cima para baixo.
    pub lines: Vec<String>,
    /// O texto marcado (o parâmetro activo), por linha em que aparece.
    pub marked: Vec<String>,
    /// Os botões das correcções (I8b): onde, e o rótulo.
    pub buttons: Vec<(Rect, String)>,
    /// O botão pintado com o realce do rato.
    pub hot: Option<usize>,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct Painted {
    pub hover: Option<HintProbe>,
    pub signature: Option<HintProbe>,
}

/// As ids dos dois balões, depois das da lista ([`crate::popup`]) no bloco da aba.
const A11Y_HOVER: u64 = (1 << 31) | (1 << 30);
const A11Y_SIGNATURE: u64 = A11Y_HOVER + 1;
/// Os botões das correcções do balão, a partir daqui.
const A11Y_FIX: u64 = A11Y_HOVER + 2;

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
            let cols = self.hint_cols();
            let mut lines: Vec<(String, Option<Range<usize>>)> = Vec::new();
            for (i, s) in h.sections.iter().enumerate() {
                if i > 0 {
                    lines.push((String::new(), None));
                }
                for l in s.title.map(|t| wrap(t, cols)).unwrap_or_default() {
                    let all = 0..l.len();
                    lines.push((l, Some(all)));
                }
                lines.extend(wrap(&plain(s.text), cols).into_iter().map(|l| (l, None)));
            }
            let fixes = Fixes {
                labels: &h.fixes,
                hot: h.hot,
            };
            self.hints.hover = self.paint_box(&lines, &fixes, at, false, scene, ts, pal);
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
        let none = Fixes::default();
        match list {
            Some(r) if !room_above => {
                let under = Rect::new(at.x, r.y, at.w, r.h);
                self.paint_box(&lines, &none, under, false, scene, ts, pal)
            }
            l => self.paint_box(&lines, &none, at, l.is_some(), scene, ts, pal),
        }
    }

    /// Um balão com `lines` (cada uma com a sua faixa marcada) e os botões `fixes` por baixo delas,
    /// por baixo de `at` ou por cima (`up`, ou quando por baixo não cabe nem uma linha). Os botões
    /// cabem sempre: as linhas que sobram são as que se cortam.
    #[allow(clippy::too_many_arguments)]
    fn paint_box(
        &mut self,
        lines: &[(String, Option<Range<usize>>)],
        fixes: &Fixes<'_>,
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
        let gap = control_gap_px();
        let n = fixes.labels.len();
        let buttons_h = if n == 0 {
            0.0
        } else {
            n as f32 * (ROW_H_PX + gap)
        };
        let rows_in =
            |space: f32| ((space - 2.0 * pad - buttons_h) / lh).floor().max(0.0) as usize;
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
        let font = Button::label_font_px();
        let widths: Vec<f32> = fixes
            .labels
            .iter()
            .map(|l| ts.prefix_width_weighted(l, font, FontWeight::MEDIUM) + 2.0 * Spacing::Lg.px())
            .collect();
        let widest = widths.iter().copied().fold(0.0, f32::max);
        let w = ((cols as f32 * self.m.advance).max(widest) + 2.0 * pad).min(tr.w);
        let h = rows as f32 * lh + 2.0 * pad + buttons_h;
        let x = (at.x - pad).min(tr.x + tr.w - w).max(tr.x);
        let y = if up { at.y - h } else { at.y + at.h };
        let rect = Rect::new(x, y, w, h);
        paint_popover(&Popover::new(NodeId(0)), rect, scene, pal.theme);
        push_popover_clip(rect, scene);
        let mut probe = HintProbe {
            rect,
            lines: Vec::new(),
            marked: Vec::new(),
            buttons: Vec::new(),
            hot: fixes.hot.filter(|&i| i < n),
        };
        let mut by = y + pad + rows as f32 * lh + gap;
        for (i, (label, bw)) in fixes.labels.iter().zip(widths).enumerate() {
            let r = Rect::new(x + pad, by, bw.min(w - 2.0 * pad), ROW_H_PX);
            let state = if fixes.hot == Some(i) {
                ButtonState::Hovered
            } else {
                ButtonState::Normal
            };
            let b = Button::new(NodeId(0), *label).on_elevated().state(state);
            paint_button(&b, r, scene, ts, pal.theme);
            probe.buttons.push((r, (*label).to_owned()));
            by += ROW_H_PX + gap;
        }
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

    /// O botão de correcção do balão do último quadro sob `(x, y)`.
    #[must_use]
    pub fn hint_button_at(&self, x: f32, y: f32) -> Option<usize> {
        self.hints
            .hover
            .as_ref()?
            .buttons
            .iter()
            .position(|(r, _)| r.contains(x, y))
    }

    /// ⚠️ SONDA: o balão do rato e a assinatura que o último quadro pintou.
    #[must_use]
    pub fn hints_painted(&self) -> (Option<HintProbe>, Option<HintProbe>) {
        (self.hints.hover.clone(), self.hints.signature.clone())
    }

    /// Os dois balões ao leitor de ecrã: `Tooltip`s filhos do `Document` (as primeiras ids); o da
    /// assinatura diz o parâmetro activo no valor; os botões das correcções são filhos do balão.
    pub(crate) fn hints_a11y(&self, base: NodeId) -> (Vec<NodeId>, Vec<(NodeId, Node)>) {
        let node = |p: &HintProbe, label: String, kids: Vec<NodeId>| {
            let r = p.rect;
            let mut b = NodeBuilder::new(Role::Tooltip)
                .label(label)
                .bounds(
                    f64::from(r.x),
                    f64::from(r.y),
                    f64::from(r.w),
                    f64::from(r.h),
                )
                .children(kids);
            if !p.marked.is_empty() {
                b = b.value(p.marked.concat());
            }
            b.build()
        };
        let (mut roots, mut out) = (Vec::new(), Vec::new());
        if let Some(p) = &self.hints.hover {
            let button = |i: usize| NodeId(base.0 + A11Y_FIX + i as u64);
            let id = NodeId(base.0 + A11Y_HOVER);
            let kids = (0..p.buttons.len()).map(button).collect();
            roots.push(id);
            out.push((id, node(p, p.lines.join("\n"), kids)));
            for (i, (r, label)) in p.buttons.iter().enumerate() {
                let b = Button::new(button(i), label.clone()).build_a11y(
                    f64::from(r.x),
                    f64::from(r.y),
                    f64::from(r.w),
                    f64::from(r.h),
                );
                out.push((button(i), b));
            }
        }
        if let Some(p) = &self.hints.signature {
            let id = NodeId(base.0 + A11Y_SIGNATURE);
            roots.push(id);
            out.push((id, node(p, p.lines.join(" "), Vec::new())));
        }
        (roots, out)
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
