//! O toque de um disco contra a forma do `sim.collide` — o caminho de quem NÃO declarou colisor.

use super::{SHAPE_BOWL, SHAPE_BOX, SHAPE_DISC, box_contact};

/// The contact at `p` for a disc of radius `r`: the outward unit normal, and how deep inside the
/// surface it is. `None` when it is clear of the surface.
///
/// `r` enters as the **Minkowski inflation** — the same shape grown outward — so there is still
/// one contact test per shape and one response for all of them. `r = 0` is the point collider,
/// term for term.
///
/// `plane_n` is the plane's normal (from [`super::plane_normal`]); the Disc and the Bowl ignore it,
/// because a circle turned is the same circle.
#[allow(clippy::too_many_arguments)]
pub(super) fn contact(
    shape: i32,
    p: [f32; 2],
    height: f32,
    c: [f32; 2],
    radius: f32,
    r: f32,
    plane_n: [f32; 2],
    half: [f32; 2],
) -> Option<([f32; 2], f32)> {
    match shape {
        SHAPE_DISC | SHAPE_BOWL => {
            let (dx, dy) = (p[0] - c[0], p[1] - c[1]);
            let dist = (dx * dx + dy * dy).sqrt();
            // Dead centre of a disc has no "way out" — any direction is as good, so pick one
            // rather than dividing by zero and turning the element into a NaN.
            let n = if dist > f32::EPSILON {
                [dx / dist, dy / dist]
            } else {
                [0.0, 1.0]
            };
            if shape == SHAPE_DISC {
                // A solid obstacle GROWS by the particle's radius: the centre of a disc of
                // radius `r` can never be closer than `r` to the surface.
                let grown = radius + r;
                (dist < grown).then_some((n, grown - dist))
            } else {
                // A container SHRINKS by it, for the same reason — and is clamped at 0: a
                // particle wider than its bowl has nowhere to be, and the honest answer is the
                // centre, not a push through it and out the other side.
                let inner = (radius - r).max(0.0);
                (dist > inner).then_some(([-n[0], -n[1]], dist - inner))
            }
        }
        SHAPE_BOX => box_contact(p, c, half, r, plane_n),
        // The plane: the world is the side its normal points to, so "out" IS the normal — and
        // what touches it is the element's near face, `sd − r`. At `angle = 0` the normal is
        // `(0, 1)` to the bit, `sd` is `p[1]`, and this is the floor test verbatim.
        _ => {
            let sd = p[0] * plane_n[0] + p[1] * plane_n[1];
            (sd - r < height).then_some((plane_n, height - (sd - r)))
        }
    }
}
