//! Connected-component island separation.
//!
//! Splits an RGBA buffer's disconnected foreground regions (alpha ≥
//! threshold) into individually cropped sub-images, sorted largest
//! first. BFS flood fill with caller's choice of 4- or 8-connectivity.

/// One detected island: its cropped RGBA bytes + bounding-box origin
/// in the source image.
#[derive(Clone, Debug)]
pub struct Island {
    pub rgba: Vec<u8>,
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
    pub pixel_count: u32,
}

/// Options for `separate_islands`. Defaults match the legacy engine.
#[derive(Copy, Clone, Debug)]
pub struct IslandOpts {
    /// Min alpha (0..=255) to count a pixel as foreground.
    pub alpha_threshold: u8,
    /// Minimum island size in pixels — smaller blobs are dropped.
    pub min_pixels: u32,
    /// 8-connected (true) or 4-connected (false) neighborhood.
    pub diagonal: bool,
}

impl Default for IslandOpts {
    fn default() -> Self {
        Self {
            alpha_threshold: 10,
            min_pixels: 4,
            diagonal: true,
        }
    }
}

/// Extract connected foreground regions. Returns islands sorted by
/// pixel count descending.
pub fn separate_islands(rgba: &[u8], w: u32, h: u32, opts: IslandOpts) -> Vec<Island> {
    let (wu, hu) = (w as usize, h as usize);
    let total = wu * hu;
    if total == 0 || rgba.len() < total * 4 {
        return Vec::new();
    }

    // Label array: 0 = unvisited, u32::MAX = background, else = label.
    let mut labels: Vec<u32> = vec![0; total];
    for (i, lab) in labels.iter_mut().enumerate() {
        if rgba[i * 4 + 3] < opts.alpha_threshold {
            *lab = u32::MAX;
        }
    }

    let mut queue: Vec<usize> = Vec::new();
    let dx4: [i32; 4] = [-1, 1, 0, 0];
    let dy4: [i32; 4] = [0, 0, -1, 1];
    let dx8: [i32; 8] = [-1, 1, 0, 0, -1, -1, 1, 1];
    let dy8: [i32; 8] = [0, 0, -1, 1, -1, 1, -1, 1];

    let mut bounds: Vec<(i32, i32, i32, i32, u32)> = Vec::new(); // (minX, minY, maxX, maxY, count)
    let mut label_count: u32 = 0;

    for start in 0..total {
        if labels[start] != 0 {
            continue;
        }
        label_count += 1;
        let label = label_count;
        let sx = (start % wu) as i32;
        let sy = (start / wu) as i32;
        let mut bnd = (sx, sy, sx, sy, 0u32);

        queue.clear();
        queue.push(start);
        labels[start] = label;

        let mut head = 0;
        while head < queue.len() {
            let p = queue[head];
            head += 1;
            let px = (p % wu) as i32;
            let py = (p / wu) as i32;
            bnd.0 = bnd.0.min(px);
            bnd.1 = bnd.1.min(py);
            bnd.2 = bnd.2.max(px);
            bnd.3 = bnd.3.max(py);
            bnd.4 += 1;

            let n = if opts.diagonal { 8 } else { 4 };
            for i in 0..n {
                let (dx, dy) = if opts.diagonal {
                    (dx8[i], dy8[i])
                } else {
                    (dx4[i], dy4[i])
                };
                let nx = px + dx;
                let ny = py + dy;
                if nx < 0 || nx >= w as i32 || ny < 0 || ny >= h as i32 {
                    continue;
                }
                let np = ny as usize * wu + nx as usize;
                if labels[np] != 0 {
                    continue;
                }
                labels[np] = label;
                queue.push(np);
            }
        }
        bounds.push(bnd);
    }

    // Build cropped output canvases.
    let mut islands: Vec<Island> = Vec::with_capacity(bounds.len());
    for (idx, b) in bounds.iter().enumerate() {
        if b.4 < opts.min_pixels {
            continue;
        }
        let label = (idx + 1) as u32;
        let min_x = b.0 as usize;
        let min_y = b.1 as usize;
        let max_x = b.2 as usize;
        let max_y = b.3 as usize;
        let iw = (max_x - min_x + 1) as u32;
        let ih = (max_y - min_y + 1) as u32;
        let mut buf = vec![0u8; (iw * ih * 4) as usize];

        for y in min_y..=max_y {
            for x in min_x..=max_x {
                let src_pos = y * wu + x;
                if labels[src_pos] != label {
                    continue;
                }
                let src_idx = src_pos * 4;
                let dst_idx = ((y - min_y) * iw as usize + (x - min_x)) * 4;
                buf[dst_idx] = rgba[src_idx];
                buf[dst_idx + 1] = rgba[src_idx + 1];
                buf[dst_idx + 2] = rgba[src_idx + 2];
                buf[dst_idx + 3] = rgba[src_idx + 3];
            }
        }

        islands.push(Island {
            rgba: buf,
            x: min_x as u32,
            y: min_y as u32,
            w: iw,
            h: ih,
            pixel_count: b.4,
        });
    }

    islands.sort_by_key(|x| std::cmp::Reverse(x.pixel_count));
    islands
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a fully opaque RGBA buffer of solid color.
    fn opaque(w: u32, h: u32, r: u8, g: u8, b: u8) -> Vec<u8> {
        let mut buf = vec![0u8; (w * h * 4) as usize];
        for i in 0..(w * h) as usize {
            buf[i * 4] = r;
            buf[i * 4 + 1] = g;
            buf[i * 4 + 2] = b;
            buf[i * 4 + 3] = 255;
        }
        buf
    }

    fn set_alpha(buf: &mut [u8], w: u32, x: u32, y: u32, a: u8) {
        let idx = ((y * w + x) * 4 + 3) as usize;
        buf[idx] = a;
    }

    #[test]
    fn fully_transparent_input_yields_no_islands() {
        let buf = vec![0u8; 8 * 8 * 4];
        let r = separate_islands(&buf, 8, 8, IslandOpts::default());
        assert!(r.is_empty());
    }

    #[test]
    fn single_blob_returns_one_island() {
        let (w, h) = (8u32, 8u32);
        let mut buf = vec![0u8; (w * h * 4) as usize];
        // 4×4 opaque red blob in top-left.
        for y in 0..4 {
            for x in 0..4 {
                let i = (y * w + x) as usize;
                buf[i * 4] = 220;
                buf[i * 4 + 3] = 255;
            }
        }
        let r = separate_islands(&buf, w, h, IslandOpts::default());
        assert_eq!(r.len(), 1);
        assert_eq!(r[0].x, 0);
        assert_eq!(r[0].y, 0);
        assert_eq!(r[0].w, 4);
        assert_eq!(r[0].h, 4);
        assert_eq!(r[0].pixel_count, 16);
    }

    #[test]
    fn two_disconnected_blobs_split() {
        // 16×8, opaque squares at (0..4, 0..4) and (10..14, 4..8).
        let (w, h) = (16u32, 8u32);
        let mut buf = vec![0u8; (w * h * 4) as usize];
        for y in 0..4 {
            for x in 0..4 {
                let i = (y * w + x) as usize;
                buf[i * 4] = 255;
                buf[i * 4 + 3] = 255;
            }
        }
        for y in 4..8 {
            for x in 10..14 {
                let i = (y * w + x) as usize;
                buf[i * 4 + 2] = 255;
                buf[i * 4 + 3] = 255;
            }
        }
        let r = separate_islands(&buf, w, h, IslandOpts::default());
        assert_eq!(r.len(), 2);
        // Both should be 4×4.
        assert_eq!(r[0].w, 4);
        assert_eq!(r[0].h, 4);
        assert_eq!(r[1].w, 4);
        assert_eq!(r[1].h, 4);
    }

    #[test]
    fn diagonal_off_keeps_neighbors_apart() {
        // Two pixels touching diagonally only.
        let (w, h) = (3u32, 3u32);
        let mut buf = opaque(w, h, 0, 0, 0);
        // Make all transparent first.
        for i in 0..(w * h) as usize {
            buf[i * 4 + 3] = 0;
        }
        // Set (0,0) and (1,1) opaque.
        set_alpha(&mut buf, w, 0, 0, 255);
        set_alpha(&mut buf, w, 1, 1, 255);

        let r4 = separate_islands(
            &buf,
            w,
            h,
            IslandOpts {
                diagonal: false,
                min_pixels: 1,
                ..Default::default()
            },
        );
        assert_eq!(r4.len(), 2, "4-conn should keep diagonal pair apart");

        let r8 = separate_islands(
            &buf,
            w,
            h,
            IslandOpts {
                diagonal: true,
                min_pixels: 1,
                ..Default::default()
            },
        );
        assert_eq!(r8.len(), 1, "8-conn should join diagonal pair");
    }

    #[test]
    fn min_pixels_drops_tiny_specks() {
        let (w, h) = (8u32, 8u32);
        let mut buf = vec![0u8; (w * h * 4) as usize];
        // 2×2 big enough.
        for y in 0..2 {
            for x in 0..2 {
                let i = (y * w + x) as usize;
                buf[i * 4 + 3] = 255;
            }
        }
        // Single speck pixel — should be dropped.
        set_alpha(&mut buf, w, 6, 6, 255);

        let r = separate_islands(
            &buf,
            w,
            h,
            IslandOpts {
                min_pixels: 2,
                diagonal: true,
                alpha_threshold: 10,
            },
        );
        assert_eq!(r.len(), 1);
        assert_eq!(r[0].pixel_count, 4);
    }

    #[test]
    fn results_sorted_largest_first() {
        let (w, h) = (16u32, 8u32);
        let mut buf = vec![0u8; (w * h * 4) as usize];
        // Big blob 6×6 at top-left.
        for y in 0..6 {
            for x in 0..6 {
                let i = (y * w + x) as usize;
                buf[i * 4 + 3] = 255;
            }
        }
        // Small blob 2×2 at bottom-right.
        for y in 6..8 {
            for x in 12..14 {
                let i = (y * w + x) as usize;
                buf[i * 4 + 3] = 255;
            }
        }
        let r = separate_islands(&buf, w, h, IslandOpts::default());
        assert_eq!(r.len(), 2);
        assert!(r[0].pixel_count > r[1].pixel_count);
    }
}
