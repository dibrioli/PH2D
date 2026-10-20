//! As perguntas ao último quadro da página: onde caiu o clique, que ponteiro, os links pela ordem do
//! Tab, o texto de um exemplo ou de uma seleção, e a sonda dos testes.

use super::layout::{Laid, Run};
use super::view::{HelpHit, HelpView, byte_in_run};
use ph2d_editor_core::doc_surface::PointerLook;
use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;
use std::ops::Range;

/// O que a página pintou no último quadro (o instrumento dos testes da família).
#[derive(Clone, Debug, PartialEq)]
pub struct HelpProbe {
    pub title: String,
    /// As linhas de texto pintadas, de cima para baixo.
    pub lines: Vec<String>,
    /// Cada link à vista: o número, o 1.º retângulo pintado e o texto.
    pub links: Vec<(u32, Rect, String)>,
    /// O membro cuja âncora está no topo da vista (ou logo acima).
    pub top_member: Option<u32>,
    pub scroll: f32,
    /// A altura da página com o enchimento (o que rola).
    pub height: f32,
    /// A secção marcada no índice.
    pub section: Option<usize>,
    /// Os trechos pintados.
    pub runs: usize,
}

impl HelpView {
    fn laid(&self) -> Option<&Laid> {
        self.laid.as_ref().map(|(_, l)| l)
    }

    fn run_rect(&self, r: &Run) -> Rect {
        self.screen(r.x, r.y, r.w, r.h)
    }

    /// O botão de copiar o exemplo `n` no ecrã.
    fn copy_rect(&self, n: u32) -> Option<Rect> {
        self.laid()?.shapes.iter().find_map(|(s, r)| {
            (*s == super::layout::Shape::Copy(n)).then(|| self.screen(r[0], r[1], r[2], r[3]))
        })
    }

    /// O texto da página, linha a linha (a família procura nele; as faixas do [`super::HelpFrame`]
    /// indexam-no).
    #[must_use]
    pub fn plain(&self) -> &str {
        self.laid().map_or("", |l| l.plain.as_str())
    }

    /// O texto de `range` (a cópia de uma seleção), preso às fronteiras de carácter.
    #[must_use]
    pub fn text(&self, range: Range<usize>) -> String {
        let p = self.plain();
        let fit = |b: usize| {
            (0..=b.min(p.len()))
                .rev()
                .find(|&i| p.is_char_boundary(i))
                .unwrap_or(0)
        };
        let (a, b) = (fit(range.start), fit(range.end));
        p[a..b.max(a)].to_owned()
    }

    /// O texto do exemplo `n` (o botão de copiar).
    #[must_use]
    pub fn example_text(&self, n: u32) -> Option<String> {
        let l = self.laid()?;
        let (_, r) = l.examples.iter().find(|e| e.0 == n)?;
        Some(l.plain[r.clone()].to_owned())
    }

    /// Os links pela ordem da página, cada um uma vez (o Tab e o Shift+Tab andam por eles).
    #[must_use]
    pub fn links_in_order(&self) -> Vec<u32> {
        let mut out: Vec<u32> = Vec::new();
        for n in self
            .laid()
            .into_iter()
            .flat_map(|l| &l.runs)
            .filter_map(|r| r.link)
        {
            if !out.contains(&n) {
                out.push(n);
            }
        }
        out
    }

    /// O 1.º retângulo do link `n` no ecrã (à vista ou não).
    #[must_use]
    pub fn link_rect(&self, n: u32) -> Option<Rect> {
        let r = self.laid()?.runs.iter().find(|r| r.link == Some(n))?;
        Some(self.run_rect(r))
    }

    /// O byte de [`Self::plain`] onde o link `n` começa (o [`Self::reveal`] do Tab).
    #[must_use]
    pub fn link_byte(&self, n: u32) -> Option<usize> {
        Some(self.laid()?.runs.iter().find(|r| r.link == Some(n))?.at)
    }

    /// O retângulo de um alvo do último quadro (os testes clicam nele); `Text`/`Find` = `None`.
    #[must_use]
    pub fn target_rect(&self, hit: HelpHit) -> Option<Rect> {
        match hit {
            HelpHit::Back => Some(self.back.0),
            HelpHit::Forward => Some(self.forward.0),
            HelpHit::Section(i) => self.rows.get(i).copied(),
            HelpHit::Link(n) => self.link_rect(n),
            HelpHit::Copy(n) => self.copy_rect(n),
            HelpHit::Text(_) | HelpHit::Find(_) => None,
        }
    }

    /// O alvo de um clique sob `(x, y)` que não é texto: botões, índice, copiar, links.
    fn target_at(&self, x: f32, y: f32) -> Option<HelpHit> {
        let (back, fwd) = (self.back, self.forward);
        if back.1 && back.0.contains(x, y) {
            return Some(HelpHit::Back);
        }
        if fwd.1 && fwd.0.contains(x, y) {
            return Some(HelpHit::Forward);
        }
        if let Some(i) = self.rows.iter().position(|r| r.contains(x, y)) {
            return Some(HelpHit::Section(i));
        }
        if !self.viewport.contains(x, y) {
            return None;
        }
        let l = self.laid()?;
        for (s, _) in &l.shapes {
            if let super::layout::Shape::Copy(n) = s
                && self.copy_rect(*n).is_some_and(|r| r.contains(x, y))
            {
                return Some(HelpHit::Copy(*n));
            }
        }
        let shown = &l.runs[self.shown.clone()];
        let r = shown
            .iter()
            .find(|r| r.link.is_some() && self.run_rect(r).contains(x, y))?;
        r.link.map(HelpHit::Link)
    }

    /// Onde caiu `(x, y)`; `None` fora da página ou num sítio que não responde (o ‹ apagado).
    #[must_use]
    pub fn hit(&self, ts: &mut TextSystem, x: f32, y: f32) -> Option<HelpHit> {
        if !self.area.contains(x, y) {
            return None;
        }
        if let Some(r) = self.find_rect
            && r.contains(x, y)
        {
            return self.find.hit(ts, x, y).map(HelpHit::Find);
        }
        if let Some(h) = self.target_at(x, y) {
            return Some(h);
        }
        if !self.viewport.contains(x, y) {
            return None;
        }
        let l = self.laid()?;
        let (ox, oy) = self.origin();
        let (px, py) = (x - ox, y - oy);
        // A linha sob o ponteiro (ou a primeira abaixo, entre parágrafos); nela, o trecho mais perto.
        let i = l.runs.partition_point(|r| r.y + r.h <= py);
        let Some(first) = l.runs.get(i) else {
            return Some(HelpHit::Text(l.plain.len()));
        };
        let line = l.runs[i..].iter().take_while(|r| r.y == first.y);
        let r = line
            .min_by(|a, b| gap_x(a, px).total_cmp(&gap_x(b, px)))
            .unwrap_or(first);
        if py < r.y {
            return Some(HelpHit::Text(r.at));
        }
        Some(HelpHit::Text(r.at + byte_in_run(ts, &self.m, r, px - r.x)))
    }

    /// O ponteiro sob `(x, y)`: a mão sobre o que leva a algum sítio, o I nos campos da busca, a
    /// seta no resto da página (medido no Godot: seta sobre o texto, mão sobre o link).
    #[must_use]
    pub fn pointer_look(&self, x: f32, y: f32) -> Option<PointerLook> {
        if !self.area.contains(x, y) {
            return None;
        }
        if let Some(over_field) = self.find.over(x, y).filter(|_| self.find_rect.is_some()) {
            return Some(if over_field {
                PointerLook::Text
            } else {
                PointerLook::Arrow
            });
        }
        Some(if self.target_at(x, y).is_some() {
            PointerLook::Hand
        } else {
            PointerLook::Arrow
        })
    }

    /// ⚠️ SONDA: o que o último quadro pintou.
    #[must_use]
    pub fn probe(&self) -> HelpProbe {
        let mut p = HelpProbe {
            title: self.title.clone(),
            lines: Vec::new(),
            links: Vec::new(),
            top_member: None,
            scroll: self.scroll,
            height: self.height(),
            section: self.section,
            runs: self.shown.len(),
        };
        let Some(l) = self.laid() else { return p };
        let shown = &l.runs[self.shown.clone()];
        let mut k = 0;
        while k < shown.len() {
            let y = shown[k].y;
            let n = shown[k..].iter().take_while(|r| r.y == y).count();
            let last = &shown[k + n - 1];
            p.lines
                .push(l.plain[shown[k].at..last.at + last.text.len()].to_owned());
            k += n;
        }
        for r in shown {
            let Some(n) = r.link else { continue };
            match p.links.iter_mut().find(|e| e.0 == n) {
                Some(e) => {
                    e.2.push(' ');
                    e.2.push_str(&r.text);
                }
                None => p.links.push((n, self.run_rect(r), r.text.clone())),
            }
        }
        let top = self.top_line();
        p.top_member = l.anchors.iter().rev().find(|a| a.1 <= top).map(|a| a.0);
        p
    }
}

/// A distância horizontal de `px` ao trecho (0 dentro dele).
fn gap_x(r: &Run, px: f32) -> f32 {
    if px < r.x {
        r.x - px
    } else {
        (px - r.x - r.w).max(0.0)
    }
}
