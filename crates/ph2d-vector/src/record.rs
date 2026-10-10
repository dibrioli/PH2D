//! ⭐ **A GRAVAÇÃO de uma cena** — as operações que um desenho emitiu, lidas de volta (W8 do Quadro,
//! `docs/MiroClone/06_equipa_e_saida.md` §2 E). O PNG sai da placa pela cena; o SVG e o PDF saem
//! desta lista — a MESMA lei de desenho para os três, sem um segundo pintor que divirja.
//!
//! Só grava quem a liga ([`VectorScene::record`]): sem ela, cada porta paga um `Option` vazio.
//! ⚠️ As portas que gravam são as desta lista; quem desenha por [`VectorScene::inner_mut`] sai da
//! gravação — o desenho do quadro tem um gate que o proíbe (`board_paint_never_bypasses_the_recording`).

use vello::kurbo::{Affine, BezPath, Rect, Shape, Stroke};
use vello::peniko::{BlendMode, Brush, Color, Fill, FontData};
use vello::{Glyph, NormalizedCoord};

use crate::VectorScene;

/// Uma operação gravada (coordenadas ANTES do `transform`, como o Vello as recebe).
#[derive(Clone, Debug)]
pub enum DrawOp {
    Fill {
        rule: Fill,
        transform: Affine,
        brush: Brush,
        path: BezPath,
    },
    Stroke {
        style: Stroke,
        transform: Affine,
        brush: Brush,
        path: BezPath,
    },
    /// Uma camada: recorta a `clip` e compõe o que vier até ao [`DrawOp::Pop`] com `alpha`.
    Layer {
        rule: Fill,
        blend: BlendMode,
        alpha: f32,
        transform: Affine,
        clip: BezPath,
    },
    Pop,
    Glyphs(Box<GlyphRun>),
    /// O rectângulo de cantos redondos desfocado do Vello (`draw_blurred_rounded_rect`): a sombra das
    /// notas.
    BlurredRect {
        transform: Affine,
        rect: Rect,
        color: Color,
        radius: f64,
        std_dev: f64,
    },
}

/// Uma corrida de glifos de uma fonte, e o TEXTO que ela mostra (o PDF devolve-o à busca e à cópia).
#[derive(Clone, Debug)]
pub struct GlyphRun {
    pub font: FontData,
    pub size: f32,
    pub coords: Vec<NormalizedCoord>,
    /// A inclinação SINTÉTICA de cada glifo (o itálico que a fonte não tem).
    pub skew: Option<Affine>,
    pub brush: Color,
    pub transform: Affine,
    pub glyphs: Vec<Glyph>,
    pub text: String,
}

impl VectorScene {
    /// Liga a gravação (e esquece o que tinha gravado).
    pub fn record(&mut self) {
        self.rec = Some(Vec::new());
    }

    /// O que se gravou desde [`Self::record`]; a gravação continua ligada, vazia.
    pub fn take_record(&mut self) -> Vec<DrawOp> {
        self.rec.as_mut().map(std::mem::take).unwrap_or_default()
    }

    pub(crate) fn rec(&mut self, op: impl FnOnce() -> DrawOp) {
        if let Some(r) = &mut self.rec {
            r.push(op());
        }
    }

    /// Contorna `shape` (a porta que grava do `Scene::stroke`).
    pub fn stroke_path(
        &mut self,
        style: &Stroke,
        transform: Affine,
        brush: &Brush,
        shape: &impl Shape,
    ) {
        self.inner.stroke(style, transform, brush, None, shape);
        self.rec(|| DrawOp::Stroke {
            style: style.clone(),
            transform,
            brush: brush.clone(),
            path: shape.to_path(0.1),
        });
    }

    /// A sombra desfocada (a porta que grava do `Scene::draw_blurred_rounded_rect`).
    pub fn blurred_rounded_rect(
        &mut self,
        transform: Affine,
        rect: Rect,
        color: Color,
        radius: f64,
        std_dev: f64,
    ) {
        self.inner
            .draw_blurred_rounded_rect(transform, rect, color, radius, std_dev);
        self.rec(|| DrawOp::BlurredRect {
            transform,
            rect,
            color,
            radius,
            std_dev,
        });
    }

    /// Uma corrida de glifos sem dica (`hint(false)`), a cheio. ⚠️ Sem gravação não aloca nada (é o
    /// texto de TODOS os quadros do ecrã): os glifos só se copiam quando alguém grava.
    pub fn glyph_run(&mut self, run: &GlyphStyle<'_>, glyphs: impl Iterator<Item = Glyph> + Clone) {
        if let Some(r) = &mut self.rec {
            r.push(DrawOp::Glyphs(Box::new(GlyphRun {
                font: run.font.clone(),
                size: run.size,
                coords: run.coords.to_vec(),
                skew: run.skew,
                brush: run.brush,
                transform: run.transform,
                glyphs: glyphs.clone().collect(),
                text: run.text.to_owned(),
            })));
        }
        self.inner
            .draw_glyphs(run.font)
            .font_size(run.size)
            .hint(false)
            .normalized_coords(run.coords)
            .glyph_transform(run.skew)
            .brush(run.brush)
            .transform(run.transform)
            .draw(Fill::NonZero, glyphs);
    }
}

/// O estilo de uma corrida de glifos, emprestado (o par de [`GlyphRun`] para quem desenha).
pub struct GlyphStyle<'a> {
    pub font: &'a FontData,
    pub size: f32,
    pub coords: &'a [NormalizedCoord],
    pub skew: Option<Affine>,
    pub brush: Color,
    pub transform: Affine,
    pub text: &'a str,
}

#[cfg(test)]
#[path = "record_tests.rs"]
mod tests;
