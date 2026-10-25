//! A vista derivada: o RGBA de um frame, camadas visíveis de baixo para cima.

use crate::sprite::{ColorMode, FrameId, LayerId, Sprite};
use crate::stroke::Stroke;

/// Compõe o frame `frame` em `out` (`w * h * 4` bytes, RGBA de alfa reto). `pending` é o traço em
/// curso, visto por cima da camada dele sem tocar o documento.
///
/// ⚠️ A mistura é a Normal (source-over, alfa reto) em inteiros; os 19 modos e a opacidade de
/// camada chegam com as camadas (P3), contra a tabela-fixtura.
pub fn compose_rgba(sprite: &Sprite, frame: FrameId, pending: Option<&Stroke>, out: &mut Vec<u8>) {
    let (w, h) = (sprite.width() as usize, sprite.height() as usize);
    out.clear();
    out.resize(w * h * 4, 0);
    for layer in sprite.layers().iter().filter(|l| l.visible) {
        let key = (layer.id, frame);
        let stroke = pending.filter(|s| s.layer == layer.id && s.frame == frame);
        let cel = sprite.cel(key);
        if cel.is_none() && stroke.is_none() {
            continue;
        }
        let region = match (cel, stroke) {
            (Some(c), Some(s)) => c.bbox.union(s.rect()),
            (Some(c), None) => c.bbox,
            (None, Some(s)) => s.rect(),
            (None, None) => continue,
        }
        .intersect(sprite.bounds());
        for y in region.y..region.y + region.h as i32 {
            for x in region.x..region.x + region.w as i32 {
                let v = stroke
                    .and_then(|s| s.value_at(sprite, x, y))
                    .unwrap_or_else(|| sprite.value_at(key, x, y));
                let src = sprite.rgba_of_value(v);
                if src[3] == 0 {
                    continue;
                }
                let i = (y as usize * w + x as usize) * 4;
                let dst = [out[i], out[i + 1], out[i + 2], out[i + 3]];
                out[i..i + 4].copy_from_slice(&over(dst, src));
            }
        }
    }
}

/// Source-over de alfa reto, arredondado (`floor(x + 0.5)`).
pub(crate) fn over(dst: [u8; 4], src: [u8; 4]) -> [u8; 4] {
    let sa = u32::from(src[3]);
    if sa == 255 || dst[3] == 0 {
        return src;
    }
    let da = u32::from(dst[3]) * (255 - sa);
    // Alfas na escala 255²: oa = sa·255 + da·(255−sa).
    let oa = sa * 255 + da;
    let mut o = [0u8; 4];
    for k in 0..3 {
        let num = u32::from(src[k]) * sa * 255 + u32::from(dst[k]) * da;
        o[k] = ((num + oa / 2) / oa) as u8;
    }
    o[3] = ((oa + 127) / 255) as u8;
    o
}

/// O que o conta-gotas lê (Aseprite: *Sample*).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sample {
    /// A imagem composta (RGBA/Gray: a cor misturada; Indexed: o índice da camada visível mais
    /// alta que não é transparente ali).
    AllLayers,
    /// Só a camada dada.
    Layer(LayerId),
}

/// O valor (no modo do sprite) na célula `(x, y)` do frame; `None` fora do sprite.
#[must_use]
pub fn pick(sprite: &Sprite, frame: FrameId, x: i32, y: i32, sample: Sample) -> Option<u32> {
    if !sprite.bounds().contains(x, y) {
        return None;
    }
    match sample {
        Sample::Layer(l) => Some(sprite.value_at((l, frame), x, y)),
        Sample::AllLayers => match sprite.mode() {
            ColorMode::Indexed => Some(
                sprite
                    .layers()
                    .iter()
                    .rev()
                    .filter(|l| l.visible)
                    .map(|l| sprite.value_at((l.id, frame), x, y))
                    .find(|&v| v != sprite.transparent_value())
                    .unwrap_or_else(|| sprite.transparent_value()),
            ),
            _ => {
                let mut px = [0u8; 4];
                for l in sprite.layers().iter().filter(|l| l.visible) {
                    let src = sprite.rgba_of_value(sprite.value_at((l.id, frame), x, y));
                    if src[3] != 0 {
                        px = over(px, src);
                    }
                }
                Some(sprite.value_of_rgba(px))
            }
        },
    }
}

#[cfg(test)]
mod tests {
    use super::over;

    #[test]
    fn over_is_identity_on_opaque_source_and_on_empty_destination() {
        assert_eq!(over([1, 2, 3, 4], [9, 8, 7, 255]), [9, 8, 7, 255]);
        assert_eq!(over([1, 2, 3, 0], [9, 8, 7, 100]), [9, 8, 7, 100]);
    }

    #[test]
    fn half_over_opaque_lands_halfway() {
        assert_eq!(
            over([0, 0, 0, 255], [255, 255, 255, 128]),
            [128, 128, 128, 255]
        );
    }
}
