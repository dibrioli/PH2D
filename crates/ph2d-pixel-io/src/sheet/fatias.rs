//! As slices na folha: `meta.slices` e o corte por slice (P11).

use std::collections::BTreeSet;

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
            color: (c.user.color != [0; 4]).then_some(c.user.color),
            data: c.user.text.clone(),
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

/// O nome de cada slice na saída. Sem comparação medida (o Trial não exporta imagem): o gémeo leva
/// o primeiro ` N` (de 2) livre contra TODOS os nomes, senão duas peças teriam a mesma chave no JSON.
pub(super) fn rotulos(s: &Sprite) -> Vec<(SliceId, String)> {
    let reservados: BTreeSet<&str> = s.slices().iter().map(|c| c.name.as_str()).collect();
    let mut dados: BTreeSet<String> = BTreeSet::new();
    s.slices()
        .iter()
        .map(|c| {
            // O próprio nome só choca com um já dado; um sufixado também com o de outra slice.
            let (mut rotulo, mut n) = (c.name.clone(), 1);
            while dados.contains(&rotulo) || (n > 1 && reservados.contains(rotulo.as_str())) {
                n += 1;
                rotulo = format!("{} {n}", c.name);
            }
            dados.insert(rotulo.clone());
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
