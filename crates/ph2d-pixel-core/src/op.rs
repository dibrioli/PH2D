//! **A porta única de mutação**: [`apply`] muda o sprite e devolve o inverso exato.

use crate::sprite::{Cel, CelKey, FrameId, LayerId, Rect, Sprite};

/// Uma mutação do documento. O catálogo cresce por onda; cada variante tem inverso exato.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Op {
    /// Células de um cel (traço, borracha, preenchimento…).
    Patch(Patch),
}

/// «Põe o cel `(layer, frame)` com a caixa `bbox` (`None` = sem cel), guardando o que já estava
/// dentro dela, e escreve `pixels` em `rect` (cortado à caixa).»
///
/// O inverso é o mesmo op com a caixa e os bytes de antes — por isso desfazer devolve o cel ao
/// byte, inclusive a caixa que o traço tinha alargado.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Patch {
    pub layer: LayerId,
    pub frame: FrameId,
    pub bbox: Option<Rect>,
    pub rect: Rect,
    /// `rect.area() * bpp` bytes, linha a linha.
    pub pixels: Vec<u8>,
}

/// Op recusado; o documento fica intacto.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OpError {
    UnknownLayer(LayerId),
    UnknownFrame(FrameId),
    /// O tamanho de `pixels` não bate com `rect` × bpp.
    PixelCount {
        expected: usize,
        got: usize,
    },
}

impl Op {
    /// Os bytes que o op segura — a régua do tecto do histórico.
    #[must_use]
    pub fn bytes(&self) -> usize {
        match self {
            Op::Patch(p) => p.pixels.len() + std::mem::size_of::<Patch>(),
        }
    }
}

/// Aplica `op` e devolve o inverso. Falha sem mudar nada.
pub fn apply(sprite: &mut Sprite, op: Op) -> Result<Op, OpError> {
    match op {
        Op::Patch(p) => apply_patch(sprite, p).map(Op::Patch),
    }
}

fn apply_patch(s: &mut Sprite, p: Patch) -> Result<Patch, OpError> {
    if s.layer(p.layer).is_none() {
        return Err(OpError::UnknownLayer(p.layer));
    }
    if !s.frames.iter().any(|f| f.id == p.frame) {
        return Err(OpError::UnknownFrame(p.frame));
    }
    let bpp = s.mode.bpp();
    let expected = p.rect.area() * bpp;
    if p.pixels.len() != expected {
        return Err(OpError::PixelCount {
            expected,
            got: p.pixels.len(),
        });
    }
    let key: CelKey = (p.layer, p.frame);
    let inverse = Patch {
        layer: p.layer,
        frame: p.frame,
        bbox: s.cels.get(&key).map(|c| c.bbox),
        rect: p.rect,
        pixels: s.read_rect(key, p.rect),
    };
    let Some(bbox) = p.bbox.filter(|b| !b.is_empty()) else {
        s.cels.remove(&key);
        return Ok(inverse);
    };
    let transparent = s.transparent_value().to_le_bytes();
    let old = s.cels.remove(&key);
    let mut data = Vec::with_capacity(bbox.area() * bpp);
    for y in 0..bbox.h as i32 {
        for x in 0..bbox.w as i32 {
            let (cx, cy) = (bbox.x + x, bbox.y + y);
            match old.as_ref().filter(|c| c.bbox.contains(cx, cy)) {
                Some(c) => {
                    let i = ((cy - c.bbox.y) as usize * c.bbox.w as usize
                        + (cx - c.bbox.x) as usize)
                        * bpp;
                    data.extend_from_slice(&c.data[i..i + bpp]);
                }
                None => data.extend_from_slice(&transparent[..bpp]),
            }
        }
    }
    let clip = p.rect.intersect(bbox);
    for y in 0..clip.h as i32 {
        for x in 0..clip.w as i32 {
            let (cx, cy) = (clip.x + x, clip.y + y);
            let src =
                ((cy - p.rect.y) as usize * p.rect.w as usize + (cx - p.rect.x) as usize) * bpp;
            let dst = ((cy - bbox.y) as usize * bbox.w as usize + (cx - bbox.x) as usize) * bpp;
            data[dst..dst + bpp].copy_from_slice(&p.pixels[src..src + bpp]);
        }
    }
    s.cels.insert(key, Cel { bbox, data });
    Ok(inverse)
}
