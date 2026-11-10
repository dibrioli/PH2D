//! ORÁCULO `#[cfg(test)]` (B29a, B52.R26): `accumulate_rgba_batch.rs` VERBATIM do HEAD 937a504585 (só os comentários tirados e os
//! caminhos apontados para os irmãos-oráculo). A paridade ao bit do esqueleto de bandas mede-se contra isto.
#![allow(dead_code, clippy::all)]

use super::ColorStampMask;
use super::accumulate_batch::FusedDab;
use super::oraculo_batch::union_rect;
use crate::dab::{DirtyRect, encode};

#[must_use]
pub fn accumulate_color_stamps_rgba_batch(
    accs: &mut [Vec<u8>],
    width: u32,
    height: u32,
    dabs: &[FusedDab],
    stamps: &[ColorStampMask],
) -> Option<DirtyRect> {
    let n = stamps.len();
    if width == 0 || height == 0 || n == 0 || dabs.is_empty() {
        return None;
    }
    let len4 = (width as usize) * (height as usize) * 4;
    if accs.len() != n || accs.iter().any(|a| a.len() != len4) {
        return None;
    }
    let size = stamps[0].size;
    if size == 0 || stamps.iter().any(|s| s.size != size) {
        return None; // a mixed-size batch would index `stamps[i].data` with stamps[0]'s stride
    }
    struct Box_ {
        di: usize,
        x0: i64,
        y0: i64,
        x1: i64,
        y1: i64,
        cx: f32,
        cy: f32,
        inv_radius: f32,
        coverage: f32,
        k: usize,
        size: u32,
    }
    let reach = stamps[0].reach;
    let mut boxes: Vec<Box_> = Vec::with_capacity(dabs.len());
    for (di, d) in dabs.iter().enumerate() {
        let coverage = d.coverage.clamp(0.0, 1.0);
        if coverage <= 0.0 || d.radius <= 0.0 {
            continue;
        }
        let (cx, cy) = (d.center[0], d.center[1]);
        let r = d.radius * reach;
        let x0 = (cx - r).floor().max(0.0) as i64;
        let y0 = (cy - r).floor().max(0.0) as i64;
        let x1 = ((cx + r).ceil() as i64 + 1).min(width as i64);
        let y1 = ((cy + r).ceil() as i64 + 1).min(height as i64);
        if x0 >= x1 || y0 >= y1 {
            continue;
        }
        boxes.push(Box_ {
            di,
            x0,
            y0,
            x1,
            y1,
            cx,
            cy,
            inv_radius: 1.0 / r,
            coverage,
            k: stamps[0].indice_do_nivel(2.0 * r),
            size: stamps[0].nivel_para(2.0 * r).size,
        });
    }
    if boxes.is_empty() {
        return None;
    }
    let gy0 = boxes.iter().map(|b| b.y0).min().expect("non-empty") as usize;
    let gy1 = boxes.iter().map(|b| b.y1).max().expect("non-empty") as usize;
    let rows = gy1 - gy0;
    let area: usize = boxes
        .iter()
        .map(|b| ((b.x1 - b.x0) * (b.y1 - b.y0)) as usize)
        .sum();
    let threads = crate::dab::band_count(area, rows, crate::dab::PARALLEL_MIN_AREA);
    let wus = width as usize;
    let rpb = rows.div_ceil(threads);
    let nbands = rows.div_ceil(rpb);
    let mut bands: Vec<Vec<&mut [u8]>> = (0..nbands).map(|_| Vec::with_capacity(n)).collect();
    for map in accs.iter_mut() {
        let mut rest = &mut map[gy0 * wus * 4..gy1 * wus * 4];
        for band in bands.iter_mut() {
            let take = (rpb * wus * 4).min(rest.len());
            let (head, tail) = rest.split_at_mut(take);
            band.push(head);
            rest = tail;
        }
    }
    let boxes = &boxes;
    let touched_by_band: Vec<Vec<bool>> = std::thread::scope(|scope| {
        let handles: Vec<_> = bands
            .into_iter()
            .enumerate()
            .map(|(bi, mut maps)| {
                scope.spawn(move || {
                    let band_y0 = (gy0 + bi * rpb) as i64;
                    let band_y1 = band_y0 + (maps[0].len() / (wus * 4)) as i64;
                    let mut touched = vec![false; dabs.len()];
                    for b in boxes {
                        let y0 = b.y0.max(band_y0);
                        let y1 = b.y1.min(band_y1);
                        for py in y0..y1 {
                            let v = ((py as f32 + 0.5) - b.cy) * b.inv_radius;
                            let Some((ty0, ty1, ty)) = crate::tip::axis_taps(v, b.size) else {
                                continue;
                            };
                            let sz = b.size as usize;
                            let (yr0, yr1) = (ty0 * sz, ty1 * sz);
                            let row = ((py - band_y0) as usize) * wus;
                            for px in b.x0..b.x1 {
                                let u = ((px as f32 + 0.5) - b.cx) * b.inv_radius;
                                let Some((xc0, xc1, tx)) = crate::tip::axis_taps(u, b.size) else {
                                    continue;
                                };
                                let o00 = (yr0 + xc0) * 4;
                                let o10 = (yr0 + xc1) * 4;
                                let o01 = (yr1 + xc0) * 4;
                                let o11 = (yr1 + xc1) * 4;
                                let o = (row + px as usize) * 4;
                                for (i, stamp) in stamps.iter().enumerate() {
                                    let d = &stamp.nivel(b.k).data;
                                    let ch = |c: usize| {
                                        let f = |off: usize| f32::from(d[off + c]) / 255.0;
                                        let top = f(o00) + (f(o10) - f(o00)) * tx;
                                        let bot = f(o01) + (f(o11) - f(o01)) * tx;
                                        top + (bot - top) * ty
                                    };
                                    let sa = ch(3);
                                    let a = sa * b.coverage;
                                    if a <= 0.0 {
                                        continue;
                                    }
                                    let acc = &mut maps[i];
                                    let inv = 1.0 - a;
                                    acc[o] = encode(
                                        ch(0) * b.coverage + f32::from(acc[o]) / 255.0 * inv,
                                    );
                                    acc[o + 1] = encode(
                                        ch(1) * b.coverage + f32::from(acc[o + 1]) / 255.0 * inv,
                                    );
                                    acc[o + 2] = encode(
                                        ch(2) * b.coverage + f32::from(acc[o + 2]) / 255.0 * inv,
                                    );
                                    acc[o + 3] = encode(a + f32::from(acc[o + 3]) / 255.0 * inv);
                                    touched[b.di] = true;
                                }
                            }
                        }
                    }
                    touched
                })
            })
            .collect();
        handles
            .into_iter()
            .map(|h| h.join().expect("rgba accumulate band panicked"))
            .collect()
    });
    let mut union: Option<DirtyRect> = None;
    for b in boxes {
        if touched_by_band.iter().any(|t| t[b.di]) {
            let r = DirtyRect {
                x: b.x0 as u32,
                y: b.y0 as u32,
                w: (b.x1 - b.x0) as u32,
                h: (b.y1 - b.y0) as u32,
            };
            union = Some(union.map_or(r, |u| union_rect(u, r)));
        }
    }
    union
}
