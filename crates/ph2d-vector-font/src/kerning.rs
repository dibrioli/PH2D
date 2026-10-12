//! **O kerning de uma linha** — o único módulo que toca o `harfrust` (o moldador que o `parley` já
//! traz para a árvore). Devolve só o que o kerning MUDA; os glifos e os avanços nominais continuam a
//! vir do [`crate::skrifa_bridge`], e é por isso que um texto sem pares sai byte-idêntico.
//!
//! ⚠️ A resposta é a DIFERENÇA entre duas moldagens da mesma linha, com e sem a feature `kern`. Um
//! avanço moldado sozinho vem arredondado à grelha do moldador, e subtraí-lo do avanço exacto da
//! fonte deixaria um resto de `1/128` em todo par de uma instância variável; a diferença é zero onde
//! não há par, por construção.

use harfrust::{
    Direction, Feature, ShapeOptions, ShaperData, ShaperInstance, Tag, UnicodeBuffer, Variation,
};

use crate::axis::AxisTag;
use crate::skrifa_bridge::VariableFont;

/// Subdivisões da unidade de design na grelha do moldador (as posições dele são inteiras).
const SUBUNITS: i32 = 64;

/// O que o kerning muda num glifo, em UNIDADES DE DESIGN.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct KernAdjust {
    /// Quanto o pen anda a MAIS depois deste glifo (negativo = o seguinte chega-se).
    pub advance: f32,
    /// Quanto o próprio glifo se desloca, sem mover o pen (o `XPlacement` do GPOS).
    pub offset: f32,
}

impl VariableFont {
    /// O kerning de `text` — um [`KernAdjust`] por `char`, na ordem — nos `axis_values` dados.
    ///
    /// `text` tem de trazer SÓ caracteres que a fonte mapeia (o layout já salta os outros); o
    /// contexto do par é o vizinho que se desenha.
    ///
    /// `None` quando a moldagem TROCA glifos (ligadura obrigatória, formas contextuais de uma escrita
    /// que as exige): o cozedor é advance-only e desenha o glifo de cada `char`, e um kerning medido
    /// sobre outra sequência poria os glifos dele em posições de glifos que não desenha. `liga`,
    /// `clig` e `calt` vão desligadas para que o latim nunca caia aqui.
    #[must_use]
    pub fn kerning(&self, text: &str, axis_values: &[(AxisTag, f32)]) -> Option<Vec<KernAdjust>> {
        let font = self.font();
        let data = self.shaping.get_or_init(|| ShaperData::new(&font));
        let instance = ShaperInstance::from_variations(
            &font,
            axis_values.iter().map(|(t, v)| Variation {
                tag: Tag::new(&t.to_bytes()),
                value: *v,
            }),
        );
        let shaper = data.shaper(&font).instance(Some(&instance)).build();
        let off = |tag: &[u8; 4]| Feature::new(Tag::new(tag), 0, ..);
        let kerned = [off(b"liga"), off(b"clig"), off(b"calt")];
        let plain = [off(b"liga"), off(b"clig"), off(b"calt"), off(b"kern")];
        let scale = i32::from(self.units_per_em()) * SUBUNITS;
        let shape = |features: &[Feature]| {
            let mut buf = UnicodeBuffer::new();
            buf.push_str(text);
            buf.guess_segment_properties();
            buf.set_direction(Direction::LeftToRight);
            shaper.shape(
                buf,
                ShapeOptions::new().scale(Some(scale)).features(features),
            )
        };
        let (with, without) = (shape(&kerned), shape(&plain));
        let mut gids = Vec::new();
        for ch in text.chars() {
            gids.push(self.glyph_for_char(ch)?.0);
        }
        let same = |b: &harfrust::GlyphBuffer| {
            b.len() == gids.len()
                && b.glyph_infos()
                    .iter()
                    .zip(&gids)
                    .all(|(i, g)| i.glyph_id == *g)
        };
        if !same(&with) || !same(&without) {
            return None;
        }
        let sub = SUBUNITS as f32;
        Some(
            with.glyph_positions()
                .iter()
                .zip(without.glyph_positions())
                .map(|(k, p)| KernAdjust {
                    advance: (k.x_advance - p.x_advance) as f32 / sub,
                    offset: (k.x_offset - p.x_offset) as f32 / sub,
                })
                .collect(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn inter() -> VariableFont {
        VariableFont::new(include_bytes!("../../ph2d-text/fonts/InterVariable.ttf").to_vec())
            .expect("InterVariable parses")
    }

    #[test]
    fn a_kerned_pair_moves_and_a_plain_pair_does_not() {
        let f = inter();
        let av = f.kerning("AV", &[]).expect("latim não troca glifos");
        assert!(av[0].advance < 0.0, "AV aproxima-se: {av:?}");
        assert_eq!(
            av[1],
            KernAdjust::default(),
            "o último glifo não tem par à direita"
        );
        let hh = f.kerning("HH", &[]).expect("latim não troca glifos");
        assert!(
            hh.iter().all(|k| *k == KernAdjust::default()),
            "HH não tem par: {hh:?}"
        );
    }

    #[test]
    fn a_ligature_the_font_would_make_is_not_made() {
        // `liga` desligada: o `fi` continua a ser dois glifos, e por isso tem resposta.
        assert_eq!(inter().kerning("fi", &[]).map(|k| k.len()), Some(2));
    }
}
