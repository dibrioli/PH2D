//! A vista derivada: o RGBA de um frame, camadas visíveis de baixo para cima.

use crate::blend::blend;
use crate::sprite::{ColorMode, FrameId, Layer, LayerId, LayerKind, Rect, Sprite};
use crate::stroke::Stroke;

/// Para quem é a imagem composta: a VISTA mostra as camadas de referência; a EXPORTAÇÃO e as
/// fusões (Merge Down, Flatten) não as levam (Aseprite: a referência não entra no ficheiro).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Purpose {
    View,
    Export,
}

/// Compõe o frame `frame` em `out` (`w * h * 4` bytes, RGBA de alfa reto). `pending` é o traço em
/// curso, visto por cima da camada dele sem tocar o documento.
pub fn compose_rgba(sprite: &Sprite, frame: FrameId, pending: Option<&Stroke>, out: &mut Vec<u8>) {
    let (w, h) = (sprite.width() as usize, sprite.height() as usize);
    out.clear();
    out.resize(w * h * 4, 0);
    compose_region(sprite, frame, pending, sprite.bounds(), Purpose::View, out);
}

/// Compõe só `region` (cortada ao sprite) dentro do `out` de tamanho cheio, que tem de vir já com
/// `w * h * 4` bytes; fora de `region` nada se toca. É o mesmo laço da composição cheia (uma só
/// resposta para «o que está por baixo deste pixel»).
pub fn compose_region(
    sprite: &Sprite,
    frame: FrameId,
    pending: Option<&Stroke>,
    region: Rect,
    purpose: Purpose,
    out: &mut [u8],
) {
    let region = region.intersect(sprite.bounds());
    if region.is_empty() {
        return;
    }
    let w = sprite.width() as usize;
    let buf = compose_window(sprite, frame, pending, region, purpose);
    for y in 0..region.h as usize {
        let src = &buf[y * region.w as usize * 4..(y + 1) * region.w as usize * 4];
        let o = ((region.y as usize + y) * w + region.x as usize) * 4;
        out[o..o + src.len()].copy_from_slice(src);
    }
}

/// A janela `region` (já dentro do sprite) composta num buffer só dela, `region.area() * 4` bytes.
#[must_use]
pub fn compose_window(
    sprite: &Sprite,
    frame: FrameId,
    pending: Option<&Stroke>,
    region: Rect,
    purpose: Purpose,
) -> Vec<u8> {
    let mut buf = vec![0u8; region.area() * 4];
    Ctx {
        sprite,
        frame,
        pending,
        region,
        purpose,
    }
    .children(None, &mut buf);
    buf
}

struct Ctx<'a> {
    sprite: &'a Sprite,
    frame: FrameId,
    pending: Option<&'a Stroke>,
    region: Rect,
    purpose: Purpose,
}

impl Ctx<'_> {
    /// Compõe os filhos de `parent`, de baixo para cima, sobre `buf` (a janela `region`).
    fn children(&self, parent: Option<LayerId>, buf: &mut [u8]) {
        for layer in self.sprite.layers().iter().filter(|l| l.parent == parent) {
            if !layer.visible || (layer.reference && self.purpose == Purpose::Export) {
                continue;
            }
            match layer.kind {
                LayerKind::Image => self.image(layer, buf),
                // O grupo é PASS-THROUGH: os filhos misturam-se direto com o que está por baixo, e o
                // modo do grupo não entra (Trial, g*/gm*.ase: 6 144/6 144 por modo).
                LayerKind::Group { .. } => self.children(Some(layer.id), buf),
            }
        }
    }

    fn image(&self, layer: &Layer, buf: &mut [u8]) {
        let key = (layer.id, self.frame);
        let stroke = self
            .pending
            .filter(|s| s.layer == layer.id && s.frame == self.frame);
        let cel = self.sprite.cel(key);
        let area = match (cel, stroke) {
            (Some(c), Some(s)) => c.bbox.union(s.rect()),
            (Some(c), None) => c.bbox,
            (None, Some(s)) => s.rect(),
            (None, None) => return,
        }
        .intersect(self.region);
        let rw = self.region.w as usize;
        for y in area.y..area.y + area.h as i32 {
            for x in area.x..area.x + area.w as i32 {
                let v = stroke
                    .and_then(|s| s.value_at(self.sprite, x, y))
                    .unwrap_or_else(|| self.sprite.value_at(key, x, y));
                let src = self.sprite.rgba_of_value(v);
                if src[3] == 0 {
                    continue;
                }
                let i = ((y - self.region.y) as usize * rw + (x - self.region.x) as usize) * 4;
                let dst = [buf[i], buf[i + 1], buf[i + 2], buf[i + 3]];
                buf[i..i + 4].copy_from_slice(&blend(layer.blend, dst, src, layer.opacity));
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
                    .filter(|l| !l.is_group() && sprite.is_shown(l.id))
                    .map(|l| sprite.value_at((l.id, frame), x, y))
                    .find(|&v| v != sprite.transparent_value())
                    .unwrap_or_else(|| sprite.transparent_value()),
            ),
            _ => {
                let px = compose_window(sprite, frame, None, Rect::new(x, y, 1, 1), Purpose::View);
                Some(sprite.value_of_rgba([px[0], px[1], px[2], px[3]]))
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
