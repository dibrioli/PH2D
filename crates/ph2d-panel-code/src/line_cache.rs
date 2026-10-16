//! ⭐ **A cache por LINHA** — moldar texto é o custo dominante; uma linha só se molda quando o texto
//! dela, os trechos de cor ou o tamanho mudam. O que não foi usado num quadro sai no fim dele: a
//! memória segue as linhas VISÍVEIS, nunca o ficheiro.
//!
//! Prova: `shapes()` conta as moldagens (rolar uma linha molda uma; editar uma linha molda essa).

use crate::style::Ink;
use parley::setting::Tag;
use parley::{
    FontFamily, FontFeature, FontFeatures, FontVariation, FontVariations, FontWeight, Layout,
    LayoutContext, StyleProperty,
};
use ph2d_text::TextSystem;
use std::borrow::Cow;
use std::collections::BTreeMap;
use std::hash::{Hash, Hasher};
use std::ops::Range;

const WGHT_TAG: Tag = Tag::new(b"wght");

/// ⛔ Sem ligaduras: `==`, `>=`, `~=` mostram-se como os caracteres escritos (o dono, 15/10: um `=`
/// fora do sítio já confundia; a JetBrains Mono junta-os por `calt`).
const NO_LIGATURES: [FontFeature; 2] = [
    FontFeature {
        tag: Tag::new(b"calt"),
        value: 0,
    },
    FontFeature {
        tag: Tag::new(b"liga"),
        value: 0,
    },
];

/// O peso do texto de código (o eixo `wght` da JetBrains Mono variável tem de ser empurrado: sem ele
/// a fonte cai no peso de omissão).
pub const CODE_WEIGHT: FontWeight = FontWeight::MEDIUM;

#[derive(Default)]
pub struct LineCache {
    lcx: LayoutContext<Ink>,
    map: BTreeMap<u64, (Layout<Ink>, u64)>,
    frame: u64,
    shapes: u64,
}

fn key(text: &str, spans: &[(Range<usize>, Ink)], font_px: f32) -> u64 {
    let mut h = std::collections::hash_map::DefaultHasher::new();
    text.hash(&mut h);
    spans.hash(&mut h);
    font_px.to_bits().hash(&mut h);
    h.finish()
}

impl LineCache {
    pub fn begin_frame(&mut self) {
        self.frame += 1;
    }

    /// Larga as linhas que este quadro não pediu.
    pub fn end_frame(&mut self) {
        let f = self.frame;
        self.map.retain(|_, (_, used)| *used == f);
    }

    /// Quantas linhas foram moldadas desde que a cache existe.
    #[must_use]
    pub fn shapes(&self) -> u64 {
        self.shapes
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.map.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// O layout da linha `text` com os trechos `spans` (bytes de `text`, ordenados).
    pub fn get(
        &mut self,
        ts: &mut TextSystem,
        text: &str,
        spans: &[(Range<usize>, Ink)],
        font_px: f32,
    ) -> &Layout<Ink> {
        let k = self.touch(ts, text, spans, font_px);
        self.peek(k)
            .unwrap_or_else(|| unreachable!("acabou de ser inserida"))
    }

    /// Garante a linha na cache (marcada como usada neste quadro) e devolve a chave dela.
    pub fn touch(
        &mut self,
        ts: &mut TextSystem,
        text: &str,
        spans: &[(Range<usize>, Ink)],
        font_px: f32,
    ) -> u64 {
        let k = key(text, spans, font_px);
        let frame = self.frame;
        match self.map.get_mut(&k) {
            Some(e) => e.1 = frame,
            None => {
                self.shapes += 1;
                let layout = shape(&mut self.lcx, ts, text, spans, font_px);
                self.map.insert(k, (layout, frame));
            }
        }
        k
    }

    /// O layout guardado sob `k` (o clique e o cursor lêem o que a pintura usou).
    #[must_use]
    pub fn peek(&self, k: u64) -> Option<&Layout<Ink>> {
        self.map.get(&k).map(|e| &e.0)
    }

    /// Molda FORA da cache e da contagem (as métricas: o avanço e os dígitos da calha).
    pub(crate) fn shape_loose(
        &mut self,
        ts: &mut TextSystem,
        text: &str,
        font_px: f32,
    ) -> Layout<Ink> {
        shape(&mut self.lcx, ts, text, &[], font_px)
    }

    /// Esquece tudo (a medição a frio).
    pub fn clear(&mut self) {
        self.map.clear();
    }
}

fn shape(
    lcx: &mut LayoutContext<Ink>,
    ts: &mut TextSystem,
    text: &str,
    spans: &[(Range<usize>, Ink)],
    font_px: f32,
) -> Layout<Ink> {
    let (fcx, _, family) = ts.mono_contexts();
    let mut b = lcx.ranged_builder(fcx, text, 1.0, true);
    b.push_default(StyleProperty::FontFamily(FontFamily::Source(
        Cow::Borrowed(family),
    )));
    b.push_default(StyleProperty::FontSize(font_px));
    b.push_default(StyleProperty::FontWeight(CODE_WEIGHT));
    let wght = [FontVariation {
        tag: WGHT_TAG,
        value: CODE_WEIGHT.value(),
    }];
    b.push_default(StyleProperty::FontVariations(FontVariations::List(
        Cow::Borrowed(&wght),
    )));
    b.push_default(StyleProperty::FontFeatures(FontFeatures::List(
        Cow::Borrowed(&NO_LIGATURES),
    )));
    b.push_default(StyleProperty::Brush(Ink::Plain));
    for (r, ink) in spans {
        let r = r.start.min(text.len())..r.end.min(text.len());
        if !r.is_empty() && text.is_char_boundary(r.start) && text.is_char_boundary(r.end) {
            b.push(StyleProperty::Brush(*ink), r);
        }
    }
    let mut layout: Layout<Ink> = b.build(text);
    layout.break_all_lines(None);
    layout
}
