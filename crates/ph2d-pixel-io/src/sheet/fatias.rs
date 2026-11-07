//! As slices na folha: `meta.slices` e o corte por slice (P11).

use ph2d_pixel_core::{Rect, Sprite};

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
