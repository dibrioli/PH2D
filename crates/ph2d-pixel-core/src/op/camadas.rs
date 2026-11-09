//! As leis da árvore de camadas e o `SetLayerNormal` (P14), partidos de `op.rs`.

use super::{Op, OpError};
use crate::sprite::{Layer, LayerId, Sprite};

/// As leis da árvore de camadas: ids únicos e já emitidos; pai existe, é grupo e vem de uma
/// cadeia sem ciclo; a Background é no máximo uma, na raiz, a primeira e de imagem; há pelo
/// menos uma camada de imagem.
pub(crate) fn check_layers(layers: &[Layer], next_id: u32) -> Result<(), OpError> {
    let bad = OpError::BadLayers;
    if !layers.iter().any(|l| !l.is_group()) {
        return Err(bad("no image layer"));
    }
    let mut seen = std::collections::BTreeSet::new();
    for l in layers {
        if l.id.0 == 0 || l.id.0 >= next_id || !seen.insert(l.id) {
            return Err(bad("layer.id"));
        }
    }
    for (i, l) in layers.iter().enumerate() {
        let mut p = l.parent;
        let mut hops = 0;
        while let Some(id) = p {
            let g = layers
                .iter()
                .find(|x| x.id == id)
                .ok_or(bad("layer.parent"))?;
            if !g.is_group() || hops > layers.len() {
                return Err(bad("layer.parent"));
            }
            p = g.parent;
            hops += 1;
        }
        if l.background && (i != 0 || l.parent.is_some() || l.is_group()) {
            return Err(bad("layer.background"));
        }
    }
    Ok(())
}

/// `Op::LayerNormal`: troca a bandeira e devolve a de antes; o grupo não tem pixels.
pub(super) fn apply_normal(s: &mut Sprite, layer: LayerId, normal: bool) -> Result<Op, OpError> {
    let l = s
        .layers
        .iter_mut()
        .find(|l| l.id == layer)
        .ok_or(OpError::UnknownLayer(layer))?;
    if l.is_group() {
        return Err(OpError::BadLayers("normal_group"));
    }
    Ok(Op::LayerNormal {
        layer,
        normal: std::mem::replace(&mut l.normal, normal),
    })
}
