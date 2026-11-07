//! As slices na folha: `meta.slices` e o corte por slice (P11).

use ph2d_pixel_core::{Rect, SliceId, Sprite};

/// `meta.slices`: na ordem inversa à do documento e em quadros absolutos, com qualquer recorte da
/// folha (medido, F5).
pub(super) fn json_slices(s: &Sprite) -> Vec<ph2d_sprite_sheet::aseprite::JsonSlice> {
    use ph2d_sprite_sheet::aseprite::{JsonSlice, JsonSliceKey};
    s.slices()
        .iter()
        .rev()
        .map(|c| JsonSlice {
            name: c.name.clone(),
            color: c.color,
            data: c.data.clone(),
            keys: c
                .keys
                .iter()
                .map(|k| JsonSliceKey {
                    frame: k.frame,
                    bounds: rect_json(k.shape.bounds),
                    center: k.shape.center.map(rect_json),
                    pivot: k.shape.pivot,
                })
                .collect(),
        })
        .collect()
}

fn rect_json(r: Rect) -> (i32, i32, u32, u32) {
    (r.x, r.y, r.w, r.h)
}

/// O nome de cada slice na saída. Sem comparação medida (o Trial não exporta imagem): os gémeos levam
/// um número, senão duas peças teriam a mesma chave no JSON.
pub(super) fn rotulos(s: &Sprite) -> Vec<(SliceId, String)> {
    let mut vistos: std::collections::BTreeMap<&str, u32> = std::collections::BTreeMap::new();
    s.slices()
        .iter()
        .map(|c| {
            let n = vistos.entry(c.name.as_str()).or_insert(0);
            *n += 1;
            let rotulo = if *n == 1 {
                c.name.clone()
            } else {
                format!("{} {n}", c.name)
            };
            (c.id, rotulo)
        })
        .collect()
}

/// A caixa de `id` no quadro `i`, cortada ao canvas. `None`: escondida, ou toda fora.
pub(crate) fn caixa_no_quadro(s: &Sprite, id: SliceId, i: usize) -> Option<Rect> {
    let k = s.slice(id)?.visible_at(i as u32)?;
    let r = k.bounds.intersect(s.bounds());
    (!r.is_empty()).then_some(r)
}

/// A caixa que cobre `id` em todos os `quadros`: um formato animado tem um só tamanho.
pub(crate) fn caixa_comum(s: &Sprite, id: SliceId, quadros: &[usize]) -> Rect {
    quadros
        .iter()
        .filter_map(|&i| caixa_no_quadro(s, id, i))
        .fold(Rect::default(), Rect::union)
}
