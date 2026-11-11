//! The f32 level of before B38.3, verbatim (`nnf.rs`/`vote.rs` at 59b02eea28): kept only so the B38
//! board draws «before» in the same process (`prancha_b38_3_nivel_f32_de_antes`). Never in the product.

mod nnf;
mod vote;

use crate::mask::{Mask, Regions};
use crate::plane::Plane;

pub(crate) fn nivel(
    content: &mut Plane,
    src: &Plane,
    mask: &Mask,
    reg: &Regions,
    r: i32,
    em_iters: u32,
    seed: u64,
) {
    let mut nnf = nnf::Nnf::init(content, src, reg, r, seed);
    for it in 0..em_iters {
        nnf.e_step(content, src, reg, r, u64::from(it));
        vote::vote(content, src, mask, reg, &nnf, r);
    }
}
