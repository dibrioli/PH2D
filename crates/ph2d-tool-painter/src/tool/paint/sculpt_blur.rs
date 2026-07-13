//! The Sculpt **kernel** and the memo that makes it affordable.
//!
//! Split from [`super::sculpt`] (the state, the session, the dab walk) for the workspace file-LOC cap,
//! and because it is one coherent thing: everything here answers *what does the relief become*, given a
//! frozen `pre` and an accumulated intensity.
//!
//! ## The memo, and why it is not an optimisation you can skip
//!
//! `blur(pre, r)` is a **constant of the stroke** — `pre` is frozen at the first dab and never moves. But
//! the dabs that read it overlap almost completely (at 10% spacing a texel is under ~10 of them), so
//! blurring each dab's footprint on demand re-does ~90% of the same arithmetic, and the box blur is
//! deliberately `O(n·r)` (the sliding sum was written for the settle and REJECTED — it drifts along the
//! row, and the byte-identity of a cropped window rests on it not drifting; see `impasto_settle`).
//!
//! So the blur is memoised **tile by tile**: each texel is blurred exactly once per stroke, whatever the
//! spacing, whatever the overlap. That is the difference between a spatula and a slideshow.
//!
//! ## Why a tile's blur is bit-for-bit the canvas's blur
//!
//! This is the property the whole memo rests on, so it is argued rather than asserted, and then gated
//! (`sculpt_tile_memo_is_byte_identical_to_a_whole_canvas_blur`).
//!
//! A box blur's output at texel `p` reads `pre` only inside `[p−r, p+r]²`. A tile is therefore blurred
//! through a **read window** grown by `r` on every side and clipped to the canvas, of which only the
//! inner tile is kept. Two cases, and they are the whole proof:
//!
//! * The window is **not** truncated on a side ⇒ every `p` in the tile is at least `r` from that window
//!   edge, so its window never reaches the edge and no clamp is ever consulted for it. The taps are the
//!   canvas's taps.
//! * The window **is** truncated on a side ⇒ that window edge IS the canvas edge, so the blur's
//!   edge-clamp reads exactly what a whole-canvas blur's edge-clamp would have read.
//!
//! Same taps, summed in the same order, by the same kernel. Not "close enough": identical.

use super::impasto::H_CEIL;
use super::region::grow_region;
use super::sculpt::SculptMode;
use super::{Region, impasto_settle};
use crate::tool::PainterTool;
use std::sync::Arc;

/// The memo's tile edge, in texels.
///
/// It trades two overheads against each other: a tile is blurred through a window grown by `r`, so a
/// SMALL tile re-reads its border many times ((64+2r)²/64² = 2.25× at the maximum radius), while a LARGE
/// one blurs texels a thin stroke never touches. 64 sits where those two curves cross for the strokes
/// this tool actually sees. // CLAMP-OK
const TILE: u32 = 64;

/// How many tiles a `w × h` canvas takes — the length of `SculptState::blur_done`.
pub(super) fn tile_count(w: u32, h: u32) -> usize {
    (w.div_ceil(TILE) as usize) * (h.div_ceil(TILE) as usize)
}

impl PainterTool {
    /// Blur every tile `rect` overlaps that this session has not blurred yet, into the memo.
    ///
    /// Each tile is computed at most ONCE per stroke (that is the point), and each one is bit-for-bit
    /// what a whole-canvas blur would have written there (see the module docs).
    pub(super) fn ensure_blur_tiles(&mut self, rect: Region, r: u32) {
        let (w, h) = self.source_size;
        if w == 0 || h == 0 || r == 0 {
            return;
        }
        let tiles_x = w.div_ceil(TILE);
        let tx0 = rect.x / TILE;
        let ty0 = rect.y / TILE;
        let tx1 = (rect.x + rect.w - 1) / TILE;
        let ty1 = (rect.y + rect.h - 1) / TILE;
        for ty in ty0..=ty1 {
            for tx in tx0..=tx1 {
                let ti = (ty * tiles_x + tx) as usize;
                // Fail CLOSED. `unwrap_or(true)` — "an unknown tile is already done" — leaves that tile's
                // memo at 0.0, and Smooth lerps toward the memo: the relief would be pulled toward ZERO,
                // flattening the paint away instead of averaging it. A silent visual catastrophe hiding
                // behind a defensive default. If a tile is not in the grid, do not touch it at all.
                match self.paint.sculpt.blur_done.get(ti) {
                    Some(true) => continue,
                    None => continue,
                    Some(false) => {}
                }
                let tile = Region {
                    x: tx * TILE,
                    y: ty * TILE,
                    w: TILE.min(w - tx * TILE),
                    h: TILE.min(h - ty * TILE),
                };
                self.blur_one_tile(tile, r);
                self.paint.sculpt.blur_done[ti] = true;
            }
        }
    }

    /// Blur ONE tile: lift the read window (the tile grown by `r`, clipped to the canvas) out of the
    /// frozen `pre`, run the shared box blur over it, and keep the inner tile.
    fn blur_one_tile(&mut self, tile: Region, r: u32) {
        let (w, h) = self.source_size;
        let Some(win) = grow_region(tile, r, w, h) else {
            return;
        };
        let (ww, wh) = (win.w as usize, win.h as usize);
        let mut scratch: Vec<f32> = Vec::with_capacity(ww * wh);
        let pre = &self.paint.sculpt.pre;
        for row in 0..wh {
            let start = (win.y as usize + row) * (w as usize) + win.x as usize;
            scratch.extend_from_slice(&pre[start..start + ww]);
        }
        // The SAME box blur the settle uses — not a second one. (A second blur is how the relief a dab
        // deposits and the relief the spatula reads start disagreeing in the last bit.)
        impasto_settle::box_blur(&mut scratch, win.w, win.h, r);
        // Keep the inner tile only: the window's outer ring of width `r` is where the buffer-edge clamp
        // could have lied, and it is exactly the part we grew in order to throw away.
        let off_x = (tile.x - win.x) as usize;
        let off_y = (tile.y - win.y) as usize;
        for row in 0..tile.h as usize {
            let src = (off_y + row) * ww + off_x;
            let dst = (tile.y as usize + row) * (w as usize) + tile.x as usize;
            self.paint.sculpt.blurred[dst..dst + tile.w as usize]
                .copy_from_slice(&scratch[src..src + tile.w as usize]);
        }
    }

    /// Re-render the relief over `rect` from the frozen `pre` and the accumulated intensity.
    ///
    /// **Always from `pre`, never from the last render.** That single rule is what buys idempotence (a
    /// shape editor re-stamping the whole stroke every frame lands on the same canvas), what keeps the
    /// smoothing scale a function of the artist's Radius rather than of their Spacing, and what lets the
    /// knobs stay live after the stroke — one law, three properties.
    pub(super) fn render_sculpt(&mut self, rect: Region) {
        let (Some(layer), (w, h)) = (self.paint.sculpt.layer, self.source_size) else {
            return;
        };
        let n = (w as usize) * (h as usize);
        if self.paint.sculpt.pre.len() != n || self.paint.sculpt.amount.len() != n {
            return; // a stale, differently-sized session: the shape guard, never an index panic
        }
        // Rebuild the memo when the radius moved, when it is not canvas-sized, OR when its per-tile flag
        // vec does not match the canvas's tile grid. That last clause is not paranoia: a rebind can change
        // `w` while leaving `n` identical (a 64×128 sprite and a 128×64 one), and then `tiles_x` is wrong
        // while every length check still passes.
        let r = self.paint.sculpt.radius_px();
        if self.paint.sculpt.blur_radius != r
            || self.paint.sculpt.blurred.len() != n
            || self.paint.sculpt.blur_done.len() != tile_count(w, h)
        {
            self.paint.sculpt.blurred = vec![0.0; n];
            self.paint.sculpt.blur_done = vec![false; tile_count(w, h)];
            self.paint.sculpt.blur_radius = r;
        }
        self.ensure_blur_tiles(rect, r);

        let sharpen = matches!(
            SculptMode::from_u8(self.paint.sculpt.mode),
            SculptMode::Sharpen
        );
        let pre = Arc::clone(&self.paint.sculpt.pre);
        let amount = std::mem::take(&mut self.paint.sculpt.amount);
        let blurred = std::mem::take(&mut self.paint.sculpt.blurred);
        let mut moved: Option<Region> = None;
        {
            let Some(entry) = self.heights.get_mut(&layer) else {
                self.paint.sculpt.amount = amount;
                self.paint.sculpt.blurred = blurred;
                return;
            };
            let target = Arc::make_mut(entry);
            // `!= n`, NOT `!= pre.len()`: `pre` is the session's, and the whole hazard is a session that
            // outlived the canvas it was measured against.
            if target.len() != n {
                self.paint.sculpt.amount = amount;
                self.paint.sculpt.blurred = blurred;
                return;
            }
            for y in rect.y..rect.y + rect.h {
                let row = (y as usize) * (w as usize);
                for x in rect.x..rect.x + rect.w {
                    let i = row + x as usize;
                    let a = amount[i];
                    if a <= 0.0 {
                        continue; // the brush never touched this texel: its relief is still `pre`
                    }
                    // The brush's Strength and Flow are ALREADY in here (they are folded into the dab's
                    // coverage as it accumulates — `ph2d_painter_brush::sculpt`). Scaling by them again
                    // was the bug the mutation run caught: it made the touch quadratic in Strength, which
                    // is the very thing the deposit does by accident and this kernel refuses to inherit.
                    let k = a.clamp(0.0, 1.0);
                    let p = pre[i];
                    // Smooth pulls toward the local average; Sharpen pushes away from it by the same
                    // amount (an unsharp mask). One expression, one sign — Sharpen is not a second
                    // engine, it is this one run backwards, and `k ≤ 1` bounds it at twice the local
                    // detail so it cannot ring away.
                    let toward = if sharpen {
                        p + p - blurred[i]
                    } else {
                        blurred[i]
                    };
                    let next = (p + k * (toward - p)).clamp(-H_CEIL, H_CEIL);
                    if (next - target[i]).abs() > impasto_settle::RELIEF_EPS {
                        let rr = Region { x, y, w: 1, h: 1 };
                        moved = Some(moved.map_or(rr, |acc| super::union_region(acc, rr)));
                    }
                    target[i] = next;
                }
            }
        }
        self.paint.sculpt.amount = amount;
        self.paint.sculpt.blurred = blurred;

        if let Some(m) = moved {
            // Grow by one: the light reads a texel's NEIGHBOURS (the normal is a central difference), so a
            // texel just outside the changed box is lit by a slope that changed inside it. Unlike the
            // deposit's re-derive — which arrives through a blur, so its edge decays below a visible byte —
            // a sculpt edit can land a HARD change right at the boundary (Sharpen does, by definition), so
            // here the margin is not a formality.
            //
            // **`mark_dirty` and NOTHING else.** The obvious `invalidate_composite()` next to it drops the
            // whole composite cache and every adjustment cut-cache, forcing a full recompose of the canvas
            // — and this runs on every pointer move. It was written that way, and the kill criterion
            // measured it at **148 ms/move at 2048²**, 37× over budget, against a 0.0 ms baseline. The
            // relief that changed is already on screen: `mark_dirty` named exactly the texels that moved,
            // and the light re-runs over them. (The deposit learned this same lesson at the same cost —
            // see the closing paragraph of `impasto::sync_relief_flags`.)
            if let Some(g) = grow_region(m, 1, w, h) {
                self.mark_dirty(g);
            }
        }
    }

    /// A Sculpt-card edit (Mode / Radius) — or a brush Strength edit — re-renders the last stroke in place.
    ///
    /// No re-stroking: the intensity is stored, the frozen source is stored, and the relief is a pure
    /// function of the two. This is the same promise the Body card makes, and it is made here for the same
    /// reason — a knob that does nothing to what you are looking at is the failure mode this section keeps
    /// producing.
    ///
    /// No-op unless the parked stroke is on the layer the artist is looking at: dialling Radius after
    /// switching layers must not reach back and re-sculpt a stroke on some other one.
    pub(super) fn refresh_live_sculpt(&mut self) {
        let (Some(layer), Some(rect)) = (self.paint.sculpt.layer, self.paint.sculpt.bbox) else {
            return;
        };
        if !self.impasto_live_edit() // "Adjust Last Stroke" — finished work stays finished
            || self.layers.active() != Some(layer)
        {
            return;
        }
        // Put the window back to the frozen source first: the render only ever WRITES texels the brush
        // touched, so a knob that shrinks the effect (Strength down, Radius down) would otherwise leave the
        // stronger version standing wherever the new one writes nothing. Restore, then re-render.
        //
        // Both planes must describe the CANVAS, not merely each other. `entry.len() == pre.len()` was the
        // guard here, and a same-AREA rebind (64×128 → 128×64) satisfies it while `rect` — measured against
        // the new width — indexes straight off the end.
        let (w, h) = self.source_size;
        let n = (w as usize) * (h as usize);
        let pre = Arc::clone(&self.paint.sculpt.pre);
        if pre.len() != n {
            self.end_sculpt_session(); // a session that no longer describes this canvas is not a session
            return;
        }
        if let Some(entry) = self.heights.get_mut(&layer)
            && entry.len() == n
        {
            let dst = Arc::make_mut(entry);
            impasto_settle::for_each_in(rect, w, |i| dst[i] = pre[i]);
        }
        self.render_sculpt(rect);
        // The restore above can itself be the only change (Strength → 0), and it wrote through `heights`
        // without telling anyone. Dirty the window unconditionally rather than trusting the render to
        // have found something to say.
        if let Some(g) = grow_region(rect, 1, w, h) {
            self.mark_dirty(g);
            self.invalidate_composite();
        }
    }
}
