//! ⭐ **A letra do código fora do editor** (dono 10/11: «exatamente a fonte de `alive`»): o nome
//! do script na lista, as entradas do «In this script» e os menus da barra da aba pintam-se pelo
//! MESMO pintor das variáveis do quadro parado ([`crate::CodeView::paint_code_value`]): a cache de
//! linhas da mono (família, corpo `TypeToken::Base` mostrado, [`crate::line_cache::CODE_WEIGHT`]),
//! [`crate::glyphs::paint_mono`] e a linha do código centrada na caixa. Só a cor é a do rótulo.
//! A medida ([`width`], [`elided`]) é a do layout que a pintura usa.

use crate::line_cache::LineCache;
use crate::metrics::Metrics;
use crate::palette::Palette;
use crate::style::Ink;
use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::Theme;
use ph2d_vector::{Color, VectorScene};
use std::borrow::Cow;
use std::cell::RefCell;

/// Quantos rótulos moldados se guardam antes de esvaziar a cache (nomes, entradas e menus à vista).
const CACHE_CAP: usize = 512;
const ELLIPSIS: &str = "\u{2026}";

#[derive(Default)]
struct Label {
    cache: LineCache,
    m: Option<Metrics>,
}

thread_local! {
    static LABEL: RefCell<Label> = RefCell::new(Label::default());
}

fn with<R>(
    ts: &mut TextSystem,
    f: impl FnOnce(&mut LineCache, &Metrics, &mut TextSystem) -> R,
) -> R {
    LABEL.with_borrow_mut(|l| {
        let px = crate::metrics::code_font_px();
        if l.m.is_none_or(|m| m.font_px.to_bits() != px.to_bits()) {
            l.cache.clear();
            l.m = Some(Metrics::measure(&mut l.cache, ts, 1).0);
        }
        if l.cache.len() > CACHE_CAP {
            l.cache.clear();
        }
        let m = l.m.unwrap_or_default();
        f(&mut l.cache, &m, ts)
    })
}

fn spans(text: &str) -> [(std::ops::Range<usize>, Ink); 1] {
    [(0..text.len(), Ink::Plain)]
}

/// A largura de `text` na mono do código, a do layout que [`paint`] desenha.
pub fn width(ts: &mut TextSystem, text: &str) -> f32 {
    with(ts, |c, m, ts| {
        c.get(ts, text, &spans(text), m.font_px).width()
    })
}

/// `text` como cabe em `w`: inteiro, ou o maior começo seguido de `…`.
pub fn elided<'a>(ts: &mut TextSystem, text: &'a str, w: f32) -> Cow<'a, str> {
    if width(ts, text) <= w {
        return Cow::Borrowed(text);
    }
    let mut ends: Vec<usize> = text.char_indices().map(|(i, _)| i).collect();
    while let Some(end) = ends.pop() {
        let s = format!("{}{ELLIPSIS}", &text[..end]);
        if width(ts, &s) <= w {
            return Cow::Owned(s);
        }
    }
    Cow::Borrowed("")
}

/// Pinta `text` (cortado em `…` se não couber em `r.w`) na mono do código, na cor `color`,
/// centrado na altura de `r`, sobre o fundo `bg` (o REAL debaixo do rótulo: cartão, linha acesa,
/// popover — o escurecimento do modo do script decide por ele, não pelo poço do código); devolve
/// onde acabou.
pub fn paint_on(
    ts: &mut TextSystem,
    scene: &mut VectorScene,
    theme: Theme,
    text: &str,
    color: Color,
    bg: Color,
    r: Rect,
) -> f32 {
    let shown = elided(ts, text, r.w.max(0.0));
    with(ts, |c, m, ts| {
        let mut pal = Palette::new(theme);
        pal.tint(Ink::Plain, color);
        pal.bg = bg;
        let row_y = r.y + (r.h - m.line_h) * 0.5;
        crate::glyphs::paint_mono(
            c,
            m,
            ts,
            scene,
            &pal,
            &shown,
            Ink::Plain,
            r.x,
            row_y,
            f32::INFINITY,
        )
    })
}
