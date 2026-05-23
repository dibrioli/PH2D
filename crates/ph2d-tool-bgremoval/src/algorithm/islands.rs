//! Connected-component extraction on the bgremoval output's alpha matte.
//!
//! Runs ONCE per Apply when [`crate::params::BgRemovalParams::separate_islands`]
//! is set, after the main pipeline has written the final RGBA into
//! [`crate::scratch::BgRemovalScratch::output_rgba`]. Each non-background
//! component above the `min_pixels` threshold is cropped into its bounding
//! box and emitted as an [`IslandPayload`] (RGBA8 sub-image + position +
//! pixel count). Sorted by descending pixel count so the host can keep the
//! biggest island in the source sprite and spawn the rest beside it
//! (legacy parity).
//!
//! Algorithm: classical BFS-on-pixels using a `Vec<u32>` queue and a
//! `Vec<i32>` label buffer (0 = unvisited, -1 = background, > 0 = island
//! id). 8-connected (legacy parity — picks up anti-aliased fringes that
//! touch only diagonally). The label + queue buffers live on
//! [`BgRemovalScratch`] so re-runs reuse the allocations (HR-3 in
//! steady state across snapshot-stable Applies).
//!
//! Per-island RGBA payloads ARE allocated fresh — they leave the tool
//! to become new sprites, so they cannot share a recycled buffer. Apply
//! is not a hot path (1× per user click).

use crate::params::ISLAND_ALPHA_THRESHOLD;
use crate::scratch::BgRemovalScratch;

/// One emitted island after CCL. `rgba` is a fresh RGBA8 sub-image
/// (`w × h`) holding only that island's pixels — pixels belonging to
/// other components inside the bounding box are zeroed (fully
/// transparent). `(x, y)` is the bounding box origin in the original
/// output's pixel space (top-left, Y-down convention to match the
/// `output_rgba` layout the pipeline writes).
#[derive(Clone, Debug)]
pub struct IslandPayload {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
    pub pixel_count: u32,
    pub rgba: Vec<u8>,
}

/// Extract every connected foreground component from `rgba` (RGBA8,
/// straight alpha, `w * h * 4` bytes), filter by `min_pixels`, and
/// append one [`IslandPayload`] per surviving component to `out` (sorted
/// by descending pixel count). Uses `scratch.labels` + `scratch.island_queue`
/// for working memory; both are grown to fit on entry.
///
/// `out` is `.clear()`-ed before populating so the caller can reuse the
/// same Vec across runs (the Vec's capacity grows but never shrinks).
///
/// # Panics
/// Panics if `rgba.len() != (w * h * 4) as usize`.
pub fn extract(
    rgba: &[u8],
    w: u32,
    h: u32,
    min_pixels: u32,
    scratch: &mut BgRemovalScratch,
    out: &mut Vec<IslandPayload>,
) {
    let expected = (w as usize) * (h as usize) * 4;
    assert_eq!(
        rgba.len(),
        expected,
        "rgba slice length must equal w*h*4 (was {} expected {})",
        rgba.len(),
        expected
    );
    out.clear();

    let n = (w as usize) * (h as usize);
    if n == 0 {
        return;
    }

    // Grow label + queue buffers to fit. Both reset to zero / empty so a
    // smaller image after a bigger one starts clean (resize-smaller
    // truncates but leaves capacity).
    scratch.labels.clear();
    scratch.labels.resize(n, 0);
    scratch.island_queue.clear();
    if scratch.island_queue.capacity() < n {
        scratch
            .island_queue
            .reserve(n - scratch.island_queue.capacity());
    }

    let wu = w as usize;
    let hu = h as usize;

    // First pass: mark background pixels as -1 so the BFS skips them
    // without a per-step alpha re-read. Pixels above the threshold stay
    // at 0 (unvisited).
    for i in 0..n {
        let alpha = rgba[i * 4 + 3];
        if alpha < ISLAND_ALPHA_THRESHOLD {
            scratch.labels[i] = -1;
        }
    }

    // Pre-compute neighbour offsets for 8-connected BFS. Bounds-checked
    // explicitly inside the loop (no wrap arithmetic on u32 / usize).
    const NEIGHBORS: [(i32, i32); 8] = [
        (-1, -1),
        (0, -1),
        (1, -1),
        (-1, 0),
        (1, 0),
        (-1, 1),
        (0, 1),
        (1, 1),
    ];

    let mut next_label: i32 = 0;

    // Per-island bounds, accumulated during BFS. Cleared between runs;
    // the Vec is kept on the stack since the count is bounded by total
    // pixels and we typically expect handfuls.
    let mut bounds: Vec<ComponentBounds> = Vec::new();

    for py in 0..hu {
        for px in 0..wu {
            let seed = py * wu + px;
            if scratch.labels[seed] != 0 {
                continue;
            }
            next_label += 1;
            let label = next_label;
            scratch.labels[seed] = label;
            scratch.island_queue.push(seed as u32);

            let mut b = ComponentBounds {
                min_x: px as u32,
                min_y: py as u32,
                max_x: px as u32,
                max_y: py as u32,
                count: 0,
            };

            // BFS — read head pointer, push to tail. Vec acts as the
            // queue; we drain by index rather than pop_front to avoid
            // shifting (the Vec is reused across components, just
            // .clear()-ed at the end of each BFS — except we keep
            // appending until empty for *this* component, then start
            // fresh for the next).
            let mut head = 0usize;
            while head < scratch.island_queue.len() {
                let p = scratch.island_queue[head] as usize;
                head += 1;
                let qx = (p % wu) as i32;
                let qy = (p / wu) as i32;
                b.count += 1;
                if (qx as u32) < b.min_x {
                    b.min_x = qx as u32;
                }
                if (qx as u32) > b.max_x {
                    b.max_x = qx as u32;
                }
                if (qy as u32) < b.min_y {
                    b.min_y = qy as u32;
                }
                if (qy as u32) > b.max_y {
                    b.max_y = qy as u32;
                }

                for (dx, dy) in NEIGHBORS {
                    let nx = qx + dx;
                    let ny = qy + dy;
                    if nx < 0 || ny < 0 || nx >= w as i32 || ny >= h as i32 {
                        continue;
                    }
                    let np = (ny as usize) * wu + (nx as usize);
                    if scratch.labels[np] != 0 {
                        continue;
                    }
                    scratch.labels[np] = label;
                    scratch.island_queue.push(np as u32);
                }
            }

            scratch.island_queue.clear();
            bounds.push(b);
        }
    }

    // Second pass: extract surviving components into fresh RGBA crops.
    // Iterating per component keeps the inner pixel test (label match)
    // simple — the alternative would be a single-pass per-pixel scatter
    // into N output buffers, which complicates ownership.
    for (idx, b) in bounds.iter().enumerate() {
        if b.count < min_pixels {
            continue;
        }
        let label = (idx as i32) + 1;
        let iw = b.max_x - b.min_x + 1;
        let ih = b.max_y - b.min_y + 1;
        let mut payload = vec![0u8; (iw as usize) * (ih as usize) * 4];
        for y in b.min_y..=b.max_y {
            for x in b.min_x..=b.max_x {
                let src_p = (y as usize) * wu + (x as usize);
                if scratch.labels[src_p] != label {
                    continue;
                }
                let src_i = src_p * 4;
                let dst_i =
                    (((y - b.min_y) as usize) * (iw as usize) + ((x - b.min_x) as usize)) * 4;
                payload[dst_i] = rgba[src_i];
                payload[dst_i + 1] = rgba[src_i + 1];
                payload[dst_i + 2] = rgba[src_i + 2];
                payload[dst_i + 3] = rgba[src_i + 3];
            }
        }
        out.push(IslandPayload {
            x: b.min_x,
            y: b.min_y,
            w: iw,
            h: ih,
            pixel_count: b.count,
            rgba: payload,
        });
    }

    // Legacy parity: biggest island first so the host can keep it as
    // the original sprite and spawn the rest beside it.
    out.sort_by_key(|island| std::cmp::Reverse(island.pixel_count));
}

#[derive(Copy, Clone, Debug)]
struct ComponentBounds {
    min_x: u32,
    min_y: u32,
    max_x: u32,
    max_y: u32,
    count: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a flat RGBA buffer where every pixel of value `1` in `mask`
    /// is opaque white and every `0` is fully transparent. `mask` is
    /// row-major, `w` wide.
    fn rgba_from_mask(mask: &[u8], w: u32, h: u32) -> Vec<u8> {
        assert_eq!(mask.len(), (w as usize) * (h as usize));
        let mut out = vec![0u8; mask.len() * 4];
        for (i, &m) in mask.iter().enumerate() {
            if m != 0 {
                out[i * 4] = 255;
                out[i * 4 + 1] = 255;
                out[i * 4 + 2] = 255;
                out[i * 4 + 3] = 255;
            }
        }
        out
    }

    #[test]
    fn extracts_two_disjoint_islands() {
        // 5x5: two single-pixel islands at opposite corners.
        #[rustfmt::skip]
        let mask: [u8; 25] = [
            1, 0, 0, 0, 0,
            0, 0, 0, 0, 0,
            0, 0, 0, 0, 0,
            0, 0, 0, 0, 0,
            0, 0, 0, 0, 1,
        ];
        let rgba = rgba_from_mask(&mask, 5, 5);
        let mut scratch = BgRemovalScratch::default();
        let mut out = Vec::new();
        extract(&rgba, 5, 5, 1, &mut scratch, &mut out);
        assert_eq!(out.len(), 2, "expected two islands, got {}", out.len());
        // Both single-pixel; bounding boxes 1×1.
        for island in &out {
            assert_eq!(island.w, 1);
            assert_eq!(island.h, 1);
            assert_eq!(island.pixel_count, 1);
            assert_eq!(island.rgba, vec![255, 255, 255, 255]);
        }
        // First should be the lexicographically-first scan position
        // (top-left island) because both have the same pixel count and
        // sort is stable.
        assert_eq!(out[0].x, 0);
        assert_eq!(out[0].y, 0);
        assert_eq!(out[1].x, 4);
        assert_eq!(out[1].y, 4);
    }

    #[test]
    fn diagonal_neighbors_are_one_island_8connected() {
        // 3x3: two pixels touching only diagonally. 8-conn ⇒ single
        // island; 4-conn would split into two.
        #[rustfmt::skip]
        let mask: [u8; 9] = [
            1, 0, 0,
            0, 1, 0,
            0, 0, 0,
        ];
        let rgba = rgba_from_mask(&mask, 3, 3);
        let mut scratch = BgRemovalScratch::default();
        let mut out = Vec::new();
        extract(&rgba, 3, 3, 1, &mut scratch, &mut out);
        assert_eq!(out.len(), 1, "diagonal pair must merge in 8-conn");
        assert_eq!(out[0].pixel_count, 2);
        // Bounding box covers both pixels.
        assert_eq!(out[0].x, 0);
        assert_eq!(out[0].y, 0);
        assert_eq!(out[0].w, 2);
        assert_eq!(out[0].h, 2);
    }

    #[test]
    fn min_pixels_filters_noise() {
        // 4x5: one 9-pixel block top, one stray bottom-right pixel
        // separated by a full empty row so the 8-conn BFS cannot
        // bridge them. min_pixels=4 ⇒ stray is dropped, block kept.
        #[rustfmt::skip]
        let mask: [u8; 20] = [
            1, 1, 1, 0,
            1, 1, 1, 0,
            1, 1, 1, 0,
            0, 0, 0, 0,
            0, 0, 0, 1,
        ];
        let rgba = rgba_from_mask(&mask, 4, 5);
        let mut scratch = BgRemovalScratch::default();
        let mut out = Vec::new();
        extract(&rgba, 4, 5, 4, &mut scratch, &mut out);
        assert_eq!(out.len(), 1, "stray pixel must be filtered");
        assert_eq!(out[0].pixel_count, 9);
    }

    #[test]
    fn fully_transparent_input_emits_nothing() {
        let rgba = vec![0u8; 8 * 8 * 4];
        let mut scratch = BgRemovalScratch::default();
        let mut out = Vec::new();
        extract(&rgba, 8, 8, 1, &mut scratch, &mut out);
        assert!(out.is_empty());
    }

    #[test]
    fn islands_are_sorted_by_descending_pixel_count() {
        // 5x5: a 3-pixel island top-left, a 5-pixel island bottom-right.
        #[rustfmt::skip]
        let mask: [u8; 25] = [
            1, 1, 0, 0, 0,
            1, 0, 0, 0, 0,
            0, 0, 0, 0, 0,
            0, 0, 0, 1, 1,
            0, 0, 1, 1, 1,
        ];
        let rgba = rgba_from_mask(&mask, 5, 5);
        let mut scratch = BgRemovalScratch::default();
        let mut out = Vec::new();
        extract(&rgba, 5, 5, 1, &mut scratch, &mut out);
        assert_eq!(out.len(), 2);
        assert_eq!(out[0].pixel_count, 5, "biggest first");
        assert_eq!(out[1].pixel_count, 3);
    }

    #[test]
    fn alpha_below_threshold_counts_as_background() {
        // 2x2: one pixel just below threshold, three above. All marked
        // as opaque white in RGB but alpha differs.
        let mut rgba = vec![
            255u8,
            255,
            255,
            255, // top-left opaque
            255,
            255,
            255,
            (ISLAND_ALPHA_THRESHOLD - 1), // top-right fringe
            255,
            255,
            255,
            255,
            255,
            255,
            255,
            255,
        ];
        let _ = &mut rgba; // suppress mut warning if optimized
        let mut scratch = BgRemovalScratch::default();
        let mut out = Vec::new();
        extract(&rgba, 2, 2, 1, &mut scratch, &mut out);
        // Three opaque pixels remain connected (L-shape, 8-conn).
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].pixel_count, 3);
    }

    #[test]
    fn reuses_scratch_buffers_across_runs() {
        // Two different-sized inputs back-to-back. Labels capacity
        // shouldn't shrink between runs (HR-3 spirit).
        let rgba_big = vec![255u8; 16 * 16 * 4];
        let rgba_small = vec![255u8; 4 * 4 * 4];
        let mut scratch = BgRemovalScratch::default();
        let mut out = Vec::new();
        extract(&rgba_big, 16, 16, 1, &mut scratch, &mut out);
        let cap_after_big = scratch.labels.capacity();
        extract(&rgba_small, 4, 4, 1, &mut scratch, &mut out);
        assert!(
            scratch.labels.capacity() >= cap_after_big,
            "labels capacity must not shrink (was {}, now {})",
            cap_after_big,
            scratch.labels.capacity()
        );
    }
}
