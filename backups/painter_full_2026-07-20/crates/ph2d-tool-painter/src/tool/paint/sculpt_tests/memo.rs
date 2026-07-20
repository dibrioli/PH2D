//! Gates for the blur **memo** — the property the whole spatula rests on.
//!
//! A child of [`super`] so the fixtures are shared and cannot drift into two subtly different canvases.
//!
//! `blur(pre, r)` is memoised tile by tile, and the memo is not an approximation of a whole-canvas blur —
//! it is supposed to BE one, bit for bit. Everything else the Smooth and Sharpen verbs promise is downstream
//! of that, so it is gated here on its own rather than inferred from a verb behaving plausibly.

use super::*;

/// **A tile's blur is bit-for-bit the canvas's blur.**
///
/// No threshold, no tolerance, no "close enough": the memo is either exactly the whole-canvas blur or the
/// spatula is a different tool at every tile boundary — a seam the eye finds instantly and the arithmetic
/// never mentions. The read window is the tile grown by `r`, and the outer ring is grown *in order to be
/// thrown away*; this asserts that throwing it away costs nothing.
///
/// **Mutation that must bleed:** in `blur_tile`, grow the window by `r - 1` (or not at all). Then a
/// tile's edge texels read a clamped neighbourhood instead of their real one, and the assert names the
/// count. (Checked: `r` → `r-1` reddens it.)
///
/// ## Both fill routes, against the same oracle
///
/// The fill runs the tiles **concurrently** past `PAR_MIN_TILES` and serially below it, and the two must
/// not be allowed to become two answers to one question. They are compared to a whole-canvas blur — an
/// oracle that knows nothing of tiles OR of threads — so a divergence in either route lands here.
///
/// The crossing is **asserted, not assumed**. This gate happened to cover 16 tiles and therefore happened
/// to exercise the parallel route; raise the threshold and it would quietly drop to the serial one and stay
/// green, testing the old path against itself while the new one went unmeasured — the ADR-0120 trap, which
/// this module has already sprung twice.
#[test]
fn the_tile_memo_is_byte_identical_to_a_whole_canvas_blur() {
    let size = 200u32; // NOT a multiple of TILE=64 → truncated edge tiles, where the clamp argument lives
    // 4×4 tiles over the whole canvas, 1 tile over the corner: one either side of the threshold.
    for (route, rect) in [
        (
            "parallel",
            Region {
                x: 0,
                y: 0,
                w: size,
                h: size,
            },
        ),
        (
            "serial",
            Region {
                x: 0,
                y: 0,
                w: 64,
                h: 64,
            },
        ),
    ] {
        let (mut t, _layer, relief) = sculpt_canvas(size);
        arm_sculpt(&mut t, 0, 1.0, 1.0); // Radius 1.0 → the maximum radius, the widest window
        assert!(
            t.ensure_sculpt_session(),
            "fixture: no session — the layer must carry relief for any of this to exist"
        );
        let r = t.sculpt_radius_px();
        assert_eq!(r, 16, "fixture: Radius 1.0 is the 16-px cap");

        let tiles = ((rect.w - 1) / 64 + 1) as usize * ((rect.h - 1) / 64 + 1) as usize;
        let is_par = tiles >= super::sculpt_blur::PAR_MIN_TILES;
        assert_eq!(
            is_par,
            route == "parallel",
            "fixture: the {route} case covers {tiles} tiles against a threshold of {} — it is about to \
             measure the OTHER route, so whichever one this case exists for is now untested",
            super::sculpt_blur::PAR_MIN_TILES
        );

        t.ensure_memo_tiles(rect, super::sculpt::MemoKey::Blur(r));
        let memo = t.paint.sculpt.memo.clone();

        let mut oracle = relief;
        super::impasto_settle::box_blur(&mut oracle, size, size, r);

        // Inside `rect` only: the tiles it covers are the ones that were asked for, and an unfilled tile is
        // legitimately still 0.0.
        let mut differing = 0usize;
        for y in rect.y..rect.y + rect.h {
            for x in rect.x..rect.x + rect.w {
                let i = (y * size + x) as usize;
                if memo[i].to_bits() != oracle[i].to_bits() {
                    differing += 1;
                }
            }
        }
        assert_eq!(
            differing, 0,
            "the {route} per-tile memo differs from a whole-canvas blur at {differing} texels. It is not \
             an approximation of one — it is supposed to BE one, bit for bit. The read window (the tile \
             grown by the radius) is what makes the edge-clamp inside a tile coincide with the canvas's."
        );
    }
}
