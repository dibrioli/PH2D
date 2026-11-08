//! As slices entre o documento e o `.ase` (P11).

use ph2d_aseprite::{AseFile, AseSlice, AseSliceKey};
use ph2d_pixel_core::{Rect, Slice, SliceId, SliceKey, SliceShape, Sprite};

/// As slices do ficheiro, na ordem dele, com ids a partir de `primeiro`. Nada se corta (F3).
pub(super) fn slices_de(file: &AseFile, primeiro: u32) -> Vec<Slice> {
    file.slices
        .iter()
        .enumerate()
        .map(|(i, s)| Slice {
            id: SliceId(primeiro + i as u32),
            name: s.name.clone(),
            color: s.color,
            data: s.text.clone(),
            keys: s
                .keys
                .iter()
                .map(|k| SliceKey {
                    frame: k.frame,
                    shape: SliceShape {
                        bounds: Rect::new(k.x, k.y, k.w, k.h),
                        center: k.center.map(|(x, y, w, h)| Rect::new(x, y, w, h)),
                        pivot: k.pivot,
                    },
                })
                .collect(),
        })
        .collect()
}

pub(super) fn slices_saida(s: &Sprite) -> Vec<AseSlice> {
    s.slices()
        .iter()
        .map(|c| AseSlice {
            name: c.name.clone(),
            color: c.color,
            text: c.data.clone(),
            maps: Vec::new(),
            keys: c
                .keys
                .iter()
                .map(|k| {
                    let b = k.shape.bounds;
                    AseSliceKey {
                        frame: k.frame,
                        x: b.x,
                        y: b.y,
                        w: b.w,
                        h: b.h,
                        center: k.shape.center.map(|r| (r.x, r.y, r.w, r.h)),
                        pivot: k.shape.pivot,
                    }
                })
                .collect(),
        })
        .collect()
}
