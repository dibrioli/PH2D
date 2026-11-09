//! O `Op::Patch` de um rectângulo pintado por uma função, para as cenas que o commitam pelo app.

use ph2d_pixel_core::{FrameId, LayerId, Op, Patch, Rect};

/// `r` pintado por `f(x, y)` (os 4 bytes do valor guardado), com a caixa do cel `bbox`.
pub fn op(
    layer: LayerId,
    frame: FrameId,
    bbox: Rect,
    r: Rect,
    f: impl Fn(i32, i32) -> [u8; 4],
) -> Op {
    let mut pixels = Vec::with_capacity(r.area() * 4);
    for y in r.y..r.y + r.h as i32 {
        for x in r.x..r.x + r.w as i32 {
            pixels.extend_from_slice(&f(x, y));
        }
    }
    Op::Patch(Patch {
        layer,
        frame,
        bbox: Some(bbox),
        rect: r,
        pixels,
    })
}
