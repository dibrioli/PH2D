//! **Efeitos de camada** (P13 §4): não destrutivos, aplicados por camada ANTES do blend com a
//! MESMA função do filtro destrutivo (uma lei, dois chamadores): [`imagem_com_efeitos`] é o que
//! a composição lê e o que [`aplicar_efeitos`] assa.

use crate::filter::{Filter, Source};
use crate::op::{DocOp, Op, Patch};
use crate::sprite::{FrameId, Layer, LayerId, Sprite};
use crate::stroke::Stroke;

/// Os parâmetros do contorno são os do filtro Edit ▸ FX ▸ Outline.
pub type OutlineParams = crate::filter::outline::Outline;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EfeitoDeCamada {
    Outline(OutlineParams),
}

impl EfeitoDeCamada {
    fn filtro(&self) -> &dyn Filter {
        match self {
            Self::Outline(o) => o,
        }
    }
}

/// A imagem da camada no quadro (cel + traço em curso) com os efeitos dela, em VALORES do modo do
/// sprite, `w * h`, linha a linha. Cada efeito é o filtro destrutivo sobre o sprite inteiro, com
/// todos os canais e sem selecção, e grava como ele (`value_of_rgba`, o índice mais próximo em
/// Indexed). `None`: sem efeitos, camada de tilemap (o filtro recusa-a) ou nada a desenhar.
#[must_use]
pub fn imagem_com_efeitos(
    s: &Sprite,
    layer: &Layer,
    frame: FrameId,
    pending: Option<&Stroke>,
) -> Option<Vec<u32>> {
    if layer.efeitos.is_empty() || layer.tileset().is_some() {
        return None;
    }
    let key = (layer.id, frame);
    let stroke = pending.filter(|t| t.layer == layer.id && t.frame == frame);
    if s.cel(key).is_none() && stroke.is_none() {
        return None;
    }
    let (w, h) = (s.width() as i32, s.height() as i32);
    let mut vals = Vec::with_capacity((w * h) as usize);
    for y in 0..h {
        for x in 0..w {
            let v = stroke.and_then(|t| t.value_at(s, x, y));
            vals.push(v.unwrap_or_else(|| s.value_at(key, x, y)));
        }
    }
    for e in &layer.efeitos {
        let src = Source::from_rgba(
            w as u32,
            h as u32,
            vals.iter().map(|&v| s.rgba_of_value(v)).collect(),
        );
        let f = e.filtro();
        for y in 0..h {
            for x in 0..w {
                vals[(y * w + x) as usize] = s.value_of_rgba(f.at(&src, x, y));
            }
        }
    }
    Some(vals)
}

/// **Apply** (Layer ▸ Effects ▸ Apply): os efeitos da camada assados nos cels de TODOS os quadros
/// (os ligados contam uma vez) e a lista esvaziada, num passo só (um `Batch`, inverso exacto).
/// `None`: a camada não existe ou não tem efeitos.
#[must_use]
pub fn aplicar_efeitos(s: &Sprite, id: LayerId) -> Option<Op> {
    let layer = s.layer(id)?;
    if layer.efeitos.is_empty() {
        return None;
    }
    let rect = s.bounds();
    let bpp = s.mode().bpp();
    let mut donos: Vec<_> = s.frames().iter().map(|f| s.owner((id, f.id))).collect();
    donos.sort_unstable();
    donos.dedup();
    let mut ops: Vec<Op> = donos
        .into_iter()
        .filter_map(|key| {
            let vals = imagem_com_efeitos(s, layer, key.1, None)?;
            let pixels: Vec<u8> = vals
                .iter()
                .flat_map(|v| v.to_le_bytes().into_iter().take(bpp))
                .collect();
            (pixels != s.read_rect(key, rect)).then(|| {
                Op::Patch(Patch {
                    layer: id,
                    frame: key.1,
                    bbox: Some(s.cel(key).map_or(rect, |c| c.bbox.union(rect))),
                    rect,
                    pixels,
                })
            })
        })
        .collect();
    ops.push(Op::Doc(Box::new(DocOp::SetLayerEffects {
        layer: id,
        efeitos: Vec::new(),
    })));
    Some(Op::Batch(ops))
}
