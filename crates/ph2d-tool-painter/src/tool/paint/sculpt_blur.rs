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

use super::impasto_ceiling::H_MAX;
use super::region::grow_region;
use super::sculpt::{MemoKey, SculptFamily, SculptMode};
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
    /// Fill every tile `rect` overlaps that this session has not filled yet, into the memo.
    ///
    /// Each tile is computed at most ONCE per stroke (that is the point), and each one is bit-for-bit what
    /// a whole-canvas run of the same kernel would have written there (see the module docs). Only the blur
    /// (Smooth / Sharpen) is memoised now — Inflate became the Blob, whose ball radius rides the live
    /// `amount`, so it is not a stroke-constant and recomputes each render in [`Self::render_inflate`].
    ///
    /// There is deliberately **no early return on a zero reach**. A zero-radius blur is the identity, and it
    /// must still be *written* — bailing out would leave the memo at
    /// `0.0`, and every verb in this family lerps TOWARD the memo, so the relief would be pulled to zero:
    /// the paint flattened away, silently, by a knob at the bottom of its track. The same fail-closed
    /// reasoning as the unknown-tile branch below.
    pub(super) fn ensure_memo_tiles(&mut self, rect: Region, key: MemoKey) {
        let (w, h) = self.source_size;
        if w == 0 || h == 0 || key == MemoKey::None {
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
                match self.paint.sculpt.memo_done.get(ti) {
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
                self.fill_one_tile(tile, key);
                self.paint.sculpt.memo_done[ti] = true;
            }
        }
    }

    /// Fill ONE tile: lift the read window (the tile grown by the kernel's [`MemoKey::reach`], clipped to
    /// the canvas) out of the frozen `pre`, run the kernel over it, and keep the inner tile.
    ///
    /// The window growth is the ONE number both kernels' byte-identity arguments rest on, so it is asked of
    /// the key rather than of the caller — a blur that grew by the offset's radius (or the reverse) would be
    /// wrong only near a tile seam, which is the hardest possible place to notice.
    fn fill_one_tile(&mut self, tile: Region, key: MemoKey) {
        let (w, h) = self.source_size;
        let Some(win) = grow_region(tile, key.reach(), w, h) else {
            return;
        };
        let (ww, wh) = (win.w as usize, win.h as usize);
        let mut scratch: Vec<f32> = Vec::with_capacity(ww * wh);
        let pre = &self.paint.sculpt.pre;
        for row in 0..wh {
            let start = (win.y as usize + row) * (w as usize) + win.x as usize;
            scratch.extend_from_slice(&pre[start..start + ww]);
        }
        // The tile, in the WINDOW's coordinates — what each kernel is asked to produce.
        let inner = Region {
            x: tile.x - win.x,
            y: tile.y - win.y,
            w: tile.w,
            h: tile.h,
        };
        let cells = (tile.w as usize) * (tile.h as usize);
        let mut out: Vec<f32> = vec![0.0; cells];
        match key {
            MemoKey::None => return,
            MemoKey::Blur(r) => {
                // The SAME box blur the settle uses — not a second one. (A second blur is how the relief a
                // dab deposits and the relief the spatula reads start disagreeing in the last bit.)
                impasto_settle::box_blur(&mut scratch, win.w, win.h, r);
                for row in 0..tile.h as usize {
                    let src = (inner.y as usize + row) * ww + inner.x as usize;
                    let dst = row * (tile.w as usize);
                    out[dst..dst + tile.w as usize]
                        .copy_from_slice(&scratch[src..src + tile.w as usize]);
                }
            }
        }
        // Keep the inner tile only: the window's outer ring is where the buffer edge could have lied, and it
        // is exactly the part we grew in order to throw away. (The source offsets are RELATIVE, so they are
        // window-independent and survive the crop unchanged — which is the whole reason they are stored as
        // an offset rather than as an index.)
        for row in 0..tile.h as usize {
            let dst = (tile.y as usize + row) * (w as usize) + tile.x as usize;
            let src = row * (tile.w as usize);
            self.paint.sculpt.memo[dst..dst + tile.w as usize]
                .copy_from_slice(&out[src..src + tile.w as usize]);
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
        let mode = self.paint.sculpt.mode_enum();
        // Inflate is the Blob: its own render, from `pre` + the live `amount` field, not memoised. It fattens
        // the form BEYOND the touched texels (a big central ball pushes the boundary out), so it owns its own
        // write region and matter advection — see `render_inflate`.
        if mode == SculptMode::Inflate {
            self.render_inflate(rect);
            return;
        }
        // The memo belongs to the MEMO family, and the guard is not tidiness: without it the plane and
        // height verbs would fall into the "not canvas-sized" branch below on every single render and
        // allocate a memo they never read — 4 B/px of pure waste, and the end of the session's 12 B/px
        // promise.
        //
        // Rebuild when the KEY moved (kernel or radius — Smooth → Inflate keeps the buffer's length and
        // changes every value in it, so a radius-only check would serve a blur to the offset), when it is
        // not canvas-sized, OR when its per-tile flag vec does not match the canvas's tile grid. That last
        // clause is not paranoia: a rebind can change `w` while leaving `n` identical (a 64×128 sprite and a
        // 128×64 one), and then `tiles_x` is wrong while every length check still passes.
        let key = self.paint.sculpt.memo_key();
        if mode.family() == SculptFamily::Memo {
            if self.paint.sculpt.memo_key != key
                || self.paint.sculpt.memo.len() != n
                || self.paint.sculpt.memo_done.len() != tile_count(w, h)
            {
                self.paint.sculpt.memo = vec![0.0; n];
                self.paint.sculpt.memo_done = vec![false; tile_count(w, h)];
                self.paint.sculpt.memo_key = key;
            }
            self.ensure_memo_tiles(rect, key);
        }

        let offset = self.paint.sculpt.plane_offset();
        let depth = self.paint.sculpt.depth();
        let pre = Arc::clone(&self.paint.sculpt.pre);
        let amount = std::mem::take(&mut self.paint.sculpt.amount);
        let plane_sum = std::mem::take(&mut self.paint.sculpt.plane_sum);
        let memo = std::mem::take(&mut self.paint.sculpt.memo);
        let mut moved: Option<Region> = None;
        {
            let Some(entry) = self.heights.get_mut(&layer) else {
                self.paint.sculpt.amount = amount;
                self.paint.sculpt.plane_sum = plane_sum;
                self.paint.sculpt.memo = memo;
                return;
            };
            let target = Arc::make_mut(entry);
            // `!= n`, NOT `!= pre.len()`: `pre` is the session's, and the whole hazard is a session that
            // outlived the canvas it was measured against. The family's own target must describe the canvas
            // too — a family switch sizes it (`ensure_family_target`), and if that has not happened yet
            // there is nothing to render, not something to guess at.
            let family_ready = match mode.family() {
                SculptFamily::Memo => memo.len() == n,
                SculptFamily::Plane => plane_sum.len() == n,
                SculptFamily::Height => true, // no buffer: the target is `pre` and a knob
            };
            if target.len() != n || !family_ready {
                self.paint.sculpt.amount = amount;
                self.paint.sculpt.plane_sum = plane_sum;
                self.paint.sculpt.memo = memo;
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
                    // ── The target: where this texel is being pulled TO. ────────────────────────────
                    //
                    // Smooth pulls toward the local average; Sharpen pushes away from it by the same amount
                    // (an unsharp mask), and `k ≤ 1` bounds it at twice the local detail so it cannot ring
                    // away. The plane verbs pull toward the coverage-weighted mean of every plane that
                    // touched the texel — `plane_sum / amount`, the division being what makes the target
                    // independent of Strength and Flow (they scale both sides). `+ offset` is the rigid
                    // shift that gives Scrape and Fill their bite; it is added HERE, not baked into
                    // `plane_sum`, which is exactly why the Offset slider is live on an open shape.
                    //
                    // Layer's target is a CONSTANT (`pre + Depth`), and that constant is what bounds it:
                    // `k ≤ 1`, so however long the artist dwells the coat never passes one Depth. Inflate's
                    // is the same, scaled by the surface's own `n_z` — see `inflate_nz`.
                    let toward = match mode {
                        SculptMode::Smooth => memo[i],
                        SculptMode::Sharpen => p + p - memo[i],
                        SculptMode::Flatten
                        | SculptMode::Scrape
                        | SculptMode::Fill
                        | SculptMode::Chisel => plane_sum[i] / a + offset,
                        SculptMode::Layer => p + depth,
                        // The relief offset by a ball of radius `Depth` — dilated, or eroded when Depth is
                        // negative. The `+ Depth` on a flat is already IN it (the ball's cap), so this is
                        // not `p + …`: the memo IS the target. See `super::sculpt_offset`.
                        SculptMode::Inflate => memo[i],
                    };
                    // ── The verb: which SIGN of the travel is allowed through. ──────────────────────
                    //
                    // Scrape is a spatula, not a press: it takes off what stands above the plane and leaves
                    // the valleys alone. Fill is its mirror. Clamping the DELTA (rather than the result)
                    // keeps `k` doing its one job — how far along the travel we go — so a half-Strength
                    // Scrape removes half the excess instead of scraping to half the plane's height, which
                    // would depend on where the origin of the height field happens to sit.
                    let delta = toward - p;
                    let delta = match mode {
                        SculptMode::Scrape | SculptMode::Chisel => delta.min(0.0),
                        SculptMode::Fill => delta.max(0.0),
                        _ => delta,
                    };
                    // The SANITY bound, not the ceiling. The ceiling is a display transform now
                    // (`impasto_ceiling::soft_ceiling`) — clamping the stored relief here is what turned a sculpted
                    // mound into a dead flat mesa two strokes in, because every value above it became the
                    // same value and a constant has no slope for the light to read.
                    let next = (p + k * delta).clamp(-H_MAX, H_MAX);
                    if (next - target[i]).abs() > impasto_settle::RELIEF_EPS {
                        let rr = Region { x, y, w: 1, h: 1 };
                        moved = Some(moved.map_or(rr, |acc| super::union_region(acc, rr)));
                    }
                    target[i] = next;
                }
            }
        }
        self.paint.sculpt.amount = amount;
        self.paint.sculpt.plane_sum = plane_sum;
        self.paint.sculpt.memo = memo;

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

    /// **Inflate — the Blob** (Enio 2026-07-14, third smoke: *"não é inflate … puxa as faces na direção das
    /// normais usando para intensidade o falloff … estude o Blob do Blender"*).
    ///
    /// A morphological offset by a ball whose **radius follows the falloff**: at the brush centre the ball is
    /// full (`|Depth|·DEPTH_UNIT_PX` px), tapering to zero at the edge. That is the whole correction. The
    /// constant-radius ball this replaced fattened the form but left a **flat top** (a max filter plateaus
    /// where the ball fits), and it was faded in by `amount` afterward — so the centre read as a flat Layer
    /// raise with an inflate rim, the *"mistura de inflate com layer"*. With the radius IN the falloff, the
    /// centre is the sphere's own curvature (a rounded peak), the rim fattens, and the edge tapers to
    /// nothing — a rounded blob, which is what a height field can offer of Blender's Blob.
    ///
    /// ## Why it is not memoised
    ///
    /// The ball radius is `|Depth|·amount(source)`, and `amount` grows across the stroke — so the offset is
    /// not a stroke-constant and the tile memo (which caches a function of the frozen `pre` alone) cannot
    /// hold it. It recomputes each render, over the batch's rect grown by the ball's reach. That is `O(ρ²)`
    /// per texel, but a source with zero coverage contributes nothing (the disc is mostly empty near the
    /// rim), so the real cost is well under the worst case.
    ///
    /// ## The form fattens BEYOND the brush, so the window is grown
    ///
    /// A big-radius source near the centre pushes the boundary out past where the brush touched — that IS the
    /// fattening — so this writes to `rect` grown by the ball's reach, not just the touched texels. And the
    /// **matter follows** (coverage / material / colour), the way it must for a form that grows onto bare
    /// canvas (the round-2 lesson): the paint dilates with the relief.
    pub(super) fn render_inflate(&mut self, rect: Region) {
        let (Some(layer), (w, h)) = (self.paint.sculpt.layer, self.source_size) else {
            return;
        };
        let n = (w as usize) * (h as usize);
        if self.paint.sculpt.pre.len() != n || self.paint.sculpt.amount.len() != n {
            return;
        }
        let depth = self.paint.sculpt.depth();
        let unit = super::impasto_light::DEPTH_UNIT_PX;
        let rho = (depth.abs() * unit).ceil() as i64;
        if rho == 0 {
            return; // Depth 0 is the identity — nothing to offset
        }
        let smooth = self.paint.sculpt.inflate_smooth_px();
        let dilate = depth > 0.0;
        // The parabola reaches ~√2·ρ before it drops below its neighbours — so the compute region is `rect`
        // grown by `2ρ` (a safe bound on that reach) PLUS the blur's reach; the write region is `rect + 2ρ`.
        let reach = 2 * rho as u32;
        let Some(cr) = grow_region(rect, reach + smooth, w, h) else {
            return;
        };
        let Some(kr) = grow_region(rect, reach, w, h) else {
            return;
        };
        let pre = Arc::clone(&self.paint.sculpt.pre);
        let amount = Arc::clone(&self.paint.sculpt.amount);
        let (cw, ch) = (cr.w as usize, cr.h as usize);
        let (wu, wi, hi) = (w as usize, i64::from(w), i64::from(h));
        // The parabola's PEAK field over the region: `pre ± |Depth|·amount`, the falloff as the height budget.
        // (`+` inflates, `−` deflates.) The curvature `a` matches the sphere of radius `|Depth|·unit` at its
        // apex, so the separable parabolic dilation is a rounded, fattening dome — the exact spherical ball's
        // `O(area·ρ²)` collapsed to `O(area)`, which is the difference between 73 ms/move and shippable.
        let mut g = vec![0.0f32; cw * ch];
        for ry in 0..ch {
            let gy = (cr.y as usize + ry) * wu + cr.x as usize;
            for rx in 0..cw {
                let a = amount[gy + rx].clamp(0.0, 1.0);
                let peak = depth.abs() * a; // in loads
                g[ry * cw + rx] = if dilate {
                    pre[gy + rx] + peak
                } else {
                    pre[gy + rx] - peak
                };
            }
        }
        let a_curv = 1.0 / (2.0 * depth.abs() * unit * unit); // 1/(2·|Depth|·unit²)
        let (mut hbuf, mut sbuf) = super::sculpt_offset::blob_dilate(&g, cr.w, cr.h, a_curv, dilate);
        // The ball has a RADIUS. The parabola is a ball only within ρ√2 of its source — that is exactly where
        // it has fallen by the full |Depth| (the ball's own height), and a real ball ends there. Past it the
        // parabola keeps lifting neighbours out of nothing, and on THICK paint that runaway skirt reaches a
        // hundred texels and gets clipped by the rectangular write region into a hard shelf: Enio's 3rd smoke,
        // *"funciona mas com um falloff de influência retangular bizarra"*. The composed argmax already carries
        // how far the winning matter travelled, so bounding `dx²+dy²` is a CIRCULAR reach cap (never a square)
        // for one comparison per texel. At ρ√2 the lift dome has already reached `pre`, so on a flat this clamp
        // touches nothing — it only ever cuts the runaway tail. Clamped texels fall back to `pre` and carry no
        // matter (`src = 0`), and the fall-back happens BEFORE the smooth blur so its shoulder stays clean.
        let reach2 = 2 * rho * rho;
        let r_edge = (reach2 as f64).sqrt();
        for i in 0..(cw * ch) {
            let (dx, dy) = super::sculpt_offset::unpack_src(sbuf[i]);
            let d2 = dx * dx + dy * dy;
            if d2 <= reach2 {
                continue; // the ball genuinely reaches this source — the exact result stands
            }
            // The winning source is past ρ√2 — the parabola's runaway. Re-clamp it ONTO the ball's rim and
            // read the height THERE: the true bounded offset is a dome that stops at ρ√2, not the unbounded
            // parabola clipped to the rectangular write region. This is symmetric, which a blunt fall-to-`pre`
            // is not: a dilation of bare canvas beside a tall wall lands on a low rim source (→ `pre`, no
            // spurious lift), while an erosion of a THIN ridge still reaches its shoulder inside ρ√2 (the rim
            // source is that shoulder, and the dig survives) — the form being eroded surrounds the texel, so
            // `pre` would have been the one height it must NOT keep.
            let (rx, ry) = ((i % cw) as i64, (i / cw) as i64);
            let gi = (cr.y as usize + ry as usize) * wu + cr.x as usize + rx as usize;
            let s = r_edge / (d2 as f64).sqrt();
            let (cdx, cdy) = ((dx as f64 * s).round() as i64, (dy as f64 * s).round() as i64);
            let (sx, sy) = (rx + cdx, ry + cdy);
            let rim = if sx >= 0 && sy >= 0 && (sx as usize) < cw && (sy as usize) < ch {
                g[(sy as usize) * cw + sx as usize]
            } else {
                pre[gi]
            };
            let drop = a_curv * (cdx * cdx + cdy * cdy) as f32;
            if dilate {
                let v = rim - drop;
                hbuf[i] = v.max(pre[gi]);
                // Advect only if the rim source actually lifted this texel (else it fell back to `pre`).
                sbuf[i] = if v > pre[gi] {
                    super::sculpt_offset::pack_src(cdx, cdy)
                } else {
                    0
                };
            } else {
                hbuf[i] = (rim + drop).min(pre[gi]);
                sbuf[i] = 0; // erosion advects nothing — the matter loop skips a less-painted source anyway
            }
        }
        if smooth > 0 {
            impasto_settle::box_blur(&mut hbuf, cr.w, cr.h, smooth);
        }

        // ── Write the height + advect the matter, over the write region `kr`. ───────────────────────────
        let pre_cover = Arc::clone(&self.paint.sculpt.pre_cover);
        let pre_mats = Arc::clone(&self.paint.sculpt.pre_mats);
        let pre_rgba = Arc::clone(&self.paint.sculpt.pre_rgba);
        let matter_ok = pre_cover.len() == n
            && pre_mats.len() == n
            && pre_rgba.len() == n * 4
            && self.canvas_rgba.len() == n * 4;
        let mut moved: Option<Region> = None;
        {
            let Some(entry) = self.heights.get_mut(&layer) else {
                return;
            };
            if entry.len() != n {
                return;
            }
            let target = Arc::make_mut(entry);
            for y in kr.y..kr.y + kr.h {
                for x in kr.x..kr.x + kr.w {
                    let ci = ((y - cr.y) as usize) * cw + (x - cr.x) as usize;
                    let gi = (y as usize) * wu + (x as usize);
                    let next = hbuf[ci].clamp(-H_MAX, H_MAX);
                    if (next - target[gi]).abs() > impasto_settle::RELIEF_EPS {
                        let rr = Region { x, y, w: 1, h: 1 };
                        moved = Some(moved.map_or(rr, |acc| super::union_region(acc, rr)));
                    }
                    target[gi] = next;
                }
            }
        }
        // The MATTER dilates with the form: where the ball carried paint from a source, that source's
        // coverage / material / colour extend to here — a form that grows onto bare canvas takes its paint
        // with it (round-2 lesson). `max` on the coverage: the paint extends, it does not thin.
        if matter_ok
            && let (Some(cov_e), Some(mat_e)) =
                (self.covers.get_mut(&layer), self.mats.get_mut(&layer))
            && cov_e.len() == n
            && mat_e.len() == n
        {
            let cov = Arc::make_mut(cov_e);
            let mat = Arc::make_mut(mat_e);
            let rgba = Arc::make_mut(&mut self.canvas_rgba);
            for y in kr.y..kr.y + kr.h {
                for x in kr.x..kr.x + kr.w {
                    let ci = ((y - cr.y) as usize) * cw + (x - cr.x) as usize;
                    let (dx, dy) = super::sculpt_offset::unpack_src(sbuf[ci]);
                    if dx == 0 && dy == 0 {
                        continue; // the matter here is its own — nothing dilated onto it
                    }
                    let (sx, sy) = (i64::from(x) + dx, i64::from(y) + dy);
                    if sx < 0 || sy < 0 || sx >= wi || sy >= hi {
                        continue;
                    }
                    let si = (sy as usize) * wu + (sx as usize);
                    let gi = (y as usize) * wu + (x as usize);
                    if pre_cover[si] <= cov[gi] {
                        continue; // the source is no more painted than here — leave it
                    }
                    cov[gi] = pre_cover[si];
                    mat[gi] = pre_mats[si];
                    rgba[gi * 4..gi * 4 + 4].copy_from_slice(&pre_rgba[si * 4..si * 4 + 4]);
                }
            }
            self.invalidate_composite();
        }

        if let Some(m) = moved {
            if let Some(g) = grow_region(m, 1, w, h) {
                self.mark_dirty(g);
            }
        } else if matter_ok {
            // The matter can change with no height crossing `RELIEF_EPS` (a flat blob fattening sideways),
            // and that write went straight to `canvas_rgba`. Dirty the write region so it uploads.
            self.mark_dirty(kr);
        }
    }

    /// A Sculpt-card edit (Mode / Radius) re-renders the gesture that is **still being authored** — and
    /// nothing else.
    ///
    /// The `Some(layer)` guard below is the whole rule, and it reads stronger than it looks: the session
    /// dies the moment a gesture is committed (`end_sculpt_session`), so *a live session IS an uncommitted
    /// gesture*. In practice that leaves exactly one case where these knobs move pixels — an **open shape
    /// editor**, whose stroke is a preview with an Apply button precisely because it is not canvas yet.
    /// Turn Radius there and the curve you are looking at re-renders; that is the shape editor's entire
    /// promise, and it would be a lie for this one card to opt out of it.
    ///
    /// A **finished** stroke is not touched. It used to be — the session was parked at pen-up and rode the
    /// Body card's "Adjust Last Stroke" checkbox — and Enio's smoke killed it in one sentence: picking
    /// **Sharpen**, in order to sharpen somewhere *else*, converted the Smooth he had just made into its
    /// opposite. See [`super::sculpt::SculptState`] for why the deposit's live-edit does not transfer to an
    /// operation: paint is a substance and has properties you can keep tuning; a smoothing is a verb that
    /// already happened.
    ///
    /// Still layer-checked: an open shape lives on ONE layer, and dialling Radius after switching away must
    /// not reach back through the layer stack to re-sculpt it.
    pub(super) fn refresh_live_sculpt(&mut self) {
        let (Some(layer), Some(rect)) = (self.paint.sculpt.layer, self.paint.sculpt.bbox) else {
            return; // no session ⇒ nothing uncommitted ⇒ the knobs arm the next stroke and stop there
        };
        if self.layers.active() != Some(layer) {
            return;
        }
        // Inflate FATTENS beyond the touched texels, and a PREVIOUS render at a larger Depth reached
        // further than the current one — so the restore has to cover the widest the Blob can ever reach
        // (`2·⌈MAX·unit⌉ + MAX_SMOOTH`), or lowering Depth would leave a ghost ring of the old fattening. It
        // is a per-knob-edit cost, not per-move, so the generous bound is cheap.
        let rect = if self.paint.sculpt.mode_enum() == SculptMode::Inflate {
            let pad = 2
                * (super::sculpt::SCULPT_DEPTH_MAX * super::impasto_light::DEPTH_UNIT_PX).ceil()
                    as u32
                + super::sculpt::INFLATE_SMOOTH_MAX_PX as u32;
            grow_region(rect, pad, self.source_size.0, self.source_size.1).unwrap_or(rect)
        } else {
            rect
        };
        // Put the window back to the frozen source first: the render only ever WRITES texels the brush
        // touched, so a knob that shrinks the effect (Strength down, Radius down) would otherwise leave the
        // stronger version standing wherever the new one writes nothing. Restore, then re-render.
        //
        // Both planes must describe the CANVAS, not merely each other. `entry.len() == pre.len()` was the
        // guard here, and a same-AREA rebind (64×128 → 128×64) satisfies it while `rect` — measured against
        // the new width — indexes straight off the end.
        let (w, h) = self.source_size;
        let n = (w as usize) * (h as usize);
        if self.paint.sculpt.pre.len() != n {
            self.end_sculpt_session(); // a session that no longer describes this canvas is not a session
            return;
        }
        // ALL FOUR planes, through the one restore — the relief AND the matter. Inflate writes coverage,
        // material and pixels, so a restore that only put the height back would leave the paint it moved
        // standing wherever the new setting moves less of it. (And it restores unconditionally: switching
        // Inflate → Smooth must give the paint back, and asking the CURRENT verb what to undo asks the
        // wrong verb.)
        self.restore_sculpt_window(layer, rect);
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
