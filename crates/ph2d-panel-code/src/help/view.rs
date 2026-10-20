//! ⭐ **A vista de uma página de ajuda** (`docs/CodeEditor/04_pagina_de_ajuda.md` §3.3): o cabeçalho
//! (‹ › e o título), o índice das secções à esquerda, o conteúdo disposto por [`layout`](super::layout) e pintado só
//! na faixa à vista, a barra de busca em baixo (como no Godot). A família guarda uma por aba e
//! entrega o [`HelpFrame`]; o clique, o ponteiro, o leitor de ecrã e a sonda lêem o último quadro
//! (`paint.rs`, `query.rs`, `a11y.rs`).

use super::layout::{Dims, Font, Laid, Run};
use super::model::HelpDoc;
use crate::chrome::NO_RECT;
use crate::find_bar::{FindBar, FindBarView, FindHit};
use crate::line_cache::{CODE_WEIGHT, LineCache};
use crate::metrics::Metrics;
use ph2d_editor_core::math::safe_clamp;
use ph2d_editor_core::ui_scale::ao_pixel;
use ph2d_editor_core::zones::Rect;
use ph2d_text::{FontWeight, TextSystem};
use ph2d_tokens::{Spacing, TypeToken};
use std::ops::Range;

/// Onde caiu um ponto da página.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HelpHit {
    Back,
    Forward,
    /// Uma linha do índice.
    Section(usize),
    Link(u32),
    /// O botão de copiar o exemplo `n`.
    Copy(u32),
    /// No conteúdo: o byte de [`HelpView::plain`] mais próximo (arrastar seleciona).
    Text(usize),
    Find(FindHit),
}

/// O que a página mostra neste quadro. As faixas de bytes indexam [`HelpView::plain`].
pub struct HelpFrame<'a> {
    pub doc: &'a HelpDoc,
    /// Há para onde voltar / avançar (o ‹ e o › acesos).
    pub back: bool,
    pub forward: bool,
    pub hover: Option<HelpHit>,
    /// O link com o foco do teclado (Tab).
    pub focus_link: Option<u32>,
    pub selection: Option<Range<usize>>,
    pub find: Option<FindBar<'a>>,
    pub matches: &'a [Range<usize>],
    pub current: Option<usize>,
}

/// Um pedido de rolagem que espera pela disposição (a página ainda não foi pintada nesta largura).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Goal {
    Member(u32),
    Section(usize),
    End,
}

/// O enchimento do conteúdo, à volta da página.
pub(super) fn pad() -> f32 {
    Spacing::Xl.px()
}

/// O tamanho e o peso de cada letra: os MESMOS na medida e na pintura (`paint_text` = MEDIUM,
/// `paint_text_title` = SEMI_BOLD).
pub(super) fn face(f: Font) -> (f32, FontWeight) {
    let px = |t: TypeToken| ph2d_text::displayed_font_px(t.px());
    match f {
        Font::Prose => (px(TypeToken::Base), FontWeight::MEDIUM),
        Font::Strong => (px(TypeToken::Base), FontWeight::SEMI_BOLD),
        Font::Heading => (px(TypeToken::Lg), FontWeight::SEMI_BOLD),
        Font::Title => (px(TypeToken::Xl), FontWeight::SEMI_BOLD),
        Font::Mono => (px(TypeToken::Base), CODE_WEIGHT),
    }
}

pub struct HelpView {
    pub(super) cache: LineCache,
    pub(super) m: Metrics,
    pub(super) dims: Dims,
    /// A disposição, guardada por (`doc.key`, largura).
    pub(super) laid: Option<((u64, u32), Laid)>,
    pub(super) scroll: f32,
    pub(super) goal: Option<Goal>,
    pub(super) area: Rect,
    pub(super) header: Rect,
    /// O ‹ e o ›: o retângulo e se estavam acesos.
    pub(super) back: (Rect, bool),
    pub(super) forward: (Rect, bool),
    /// A coluna do índice (`NO_RECT` na área estreita) e as linhas dela.
    pub(super) index: Rect,
    pub(super) rows: Vec<Rect>,
    /// O conteúdo à vista (sem a barra de busca).
    pub(super) viewport: Rect,
    pub(super) find: FindBarView,
    pub(super) find_rect: Option<Rect>,
    pub(super) title: String,
    pub(super) labels: Vec<String>,
    /// Os trechos pintados no último quadro.
    pub(super) shown: Range<usize>,
    pub(super) focus: Option<u32>,
    /// A secção corrente (a marcada no índice).
    pub(super) section: Option<usize>,
}

impl Default for HelpView {
    fn default() -> Self {
        Self {
            cache: LineCache::default(),
            m: Metrics::default(),
            dims: Dims::default(),
            laid: None,
            scroll: 0.0,
            goal: None,
            area: NO_RECT,
            header: NO_RECT,
            back: (NO_RECT, false),
            forward: (NO_RECT, false),
            index: NO_RECT,
            rows: Vec::new(),
            viewport: NO_RECT,
            find: FindBarView::default(),
            find_rect: None,
            title: String::new(),
            labels: Vec::new(),
            shown: 0..0,
            focus: None,
            section: None,
        }
    }
}

impl HelpView {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// As medidas da mono (as do editor).
    #[must_use]
    pub fn metrics(&self) -> &Metrics {
        &self.m
    }

    #[must_use]
    pub fn scroll(&self) -> f32 {
        self.scroll
    }

    /// A rolagem guardada de uma entrada do histórico (presa à página na próxima pintura).
    pub fn set_scroll(&mut self, y: f32) {
        self.goal = None;
        self.scroll = y;
        self.clamp();
    }

    pub fn scroll_by(&mut self, dy: f32) {
        self.set_scroll(self.scroll + dy);
    }

    /// PgDn / PgUp: a altura da vista.
    pub fn page(&mut self, down: bool) {
        let h = self.viewport.h;
        self.scroll_by(if down { h } else { -h });
    }

    pub fn home(&mut self) {
        self.set_scroll(0.0);
    }

    pub fn end(&mut self) {
        self.go(Goal::End);
    }

    /// A assinatura do membro `m` no topo da vista (o Ctrl+clique, como no Godot).
    pub fn scroll_to_member(&mut self, m: u32) {
        self.go(Goal::Member(m));
    }

    /// O título da secção `i` no topo da vista (o índice).
    pub fn scroll_to_section(&mut self, i: usize) {
        self.go(Goal::Section(i));
    }

    /// Rola o mínimo para mostrar o byte `byte` de [`Self::plain`] (a busca, o Tab).
    pub fn reveal(&mut self, byte: usize) {
        let Some((_, l)) = &self.laid else { return };
        let i = l.runs.partition_point(|r| r.at + r.text.len() < byte);
        let Some(r) = l.runs.get(i) else { return };
        let (top, bottom) = (r.y + pad(), r.y + r.h + pad());
        let y = if top < self.scroll {
            top
        } else if bottom > self.scroll + self.viewport.h {
            bottom - self.viewport.h
        } else {
            return;
        };
        self.set_scroll(y);
    }

    /// Guarda o pedido e cumpre-o já se a página está disposta (a pintura cumpre-o de novo sobre a
    /// disposição fresca).
    fn go(&mut self, g: Goal) {
        self.goal = Some(g);
        self.apply_goal();
    }

    pub(super) fn apply_goal(&mut self) {
        let (Some(g), Some((_, l))) = (self.goal, &self.laid) else {
            return;
        };
        let y = match g {
            Goal::Member(m) => l.anchors.iter().find(|a| a.0 == m).map(|a| a.1 + pad()),
            Goal::Section(i) => l.sections.get(i).map(|y| y + pad()),
            Goal::End => Some(self.max_scroll()),
        };
        if let Some(y) = y {
            self.scroll = y;
        }
        self.clamp();
    }

    /// A altura da página com o enchimento (o que rola).
    pub(super) fn height(&self) -> f32 {
        self.laid
            .as_ref()
            .map_or(0.0, |(_, l)| l.height + 2.0 * pad())
    }

    fn max_scroll(&self) -> f32 {
        (self.height() - self.viewport.h).max(0.0)
    }

    pub(super) fn clamp(&mut self) {
        self.scroll = if self.laid.is_some() {
            safe_clamp(self.scroll, 0.0, self.max_scroll())
        } else {
            self.scroll.max(0.0)
        };
    }

    /// O canto da página (`0, 0` da disposição) no ecrã.
    pub(super) fn origin(&self) -> (f32, f32) {
        (
            self.viewport.x + pad(),
            self.viewport.y + pad() - ao_pixel(self.scroll),
        )
    }

    /// Um retângulo da página no ecrã.
    pub(super) fn screen(&self, x: f32, y: f32, w: f32, h: f32) -> Rect {
        let (ox, oy) = self.origin();
        Rect::new(ox + x, oy + y, w, h)
    }

    /// O `y` da página no topo da vista, mais a folga de uma linha que «já chegou».
    pub(super) fn top_line(&self) -> f32 {
        self.scroll - pad() + self.dims.gap
    }
}

/// O byte `k` de `s`, na fronteira de carácter de baixo.
fn floor_char(s: &str, k: usize) -> usize {
    let k = k.min(s.len());
    (0..=k).rev().find(|&i| s.is_char_boundary(i)).unwrap_or(0)
}

/// O x (relativo ao início do trecho) do byte `k` dele: a prosa mede-se na fonte da pintura, a mono
/// conta colunas. A mesma conta da pintura das marcas e do clique.
pub(super) fn x_in_run(ts: &mut TextSystem, m: &Metrics, r: &Run, k: usize) -> f32 {
    let head = &r.text[..floor_char(&r.text, k)];
    if r.font == Font::Mono {
        head.chars().count() as f32 * m.advance
    } else {
        let (px, w) = face(r.font);
        ts.prefix_width_weighted(head, px, w)
    }
}

/// O byte do trecho mais próximo de `dx` (relativo ao início dele): o meio de cada carácter decide.
pub(super) fn byte_in_run(ts: &mut TextSystem, m: &Metrics, r: &Run, dx: f32) -> usize {
    let mut prev = (0usize, 0.0f32);
    let ends = r
        .text
        .char_indices()
        .map(|(i, _)| i)
        .skip(1)
        .chain(std::iter::once(r.text.len()));
    for i in ends {
        let x = x_in_run(ts, m, r, i);
        if dx < (prev.1 + x) * 0.5 {
            return prev.0;
        }
        prev = (i, x);
    }
    r.text.len()
}
