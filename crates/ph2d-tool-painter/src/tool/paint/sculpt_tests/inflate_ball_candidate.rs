//! **The bounded-ball workshop.** The gold-standard border is the TRUE BALL (`√(ρ²−d²)`), proven in
//! [`super::inflate_edge_probes::diag_prove_the_true_ball_model`]; its only open question is SPEED. The
//! shipped kernel is a separable parabolic dilation — `O(N)`, but the parabola has **unbounded support**,
//! and that is the whole of the junction bug ([`super::inflate_junction_probes`]): a source captures the
//! envelope out to `√(H/a)` while it can only serve out to `ρ√2`, so a tall junction hands its Voronoi cell
//! a dead zone, and the boundary of that cell is the white gash.
//!
//! A **bounded** structuring element has `capture == reach` by construction — no dead zone, no gash. The
//! exact ball is bounded but `O(area·ρ²)`. These probes settle, by measurement and not by assertion, the one
//! question that decides the shape of the fix: **is the exact ball's real per-move cost inside the kill
//! criterion?** They mirror the kill harness ([`super::session::sculpt_perf_kill_criterion`]) — 2048²/4096²,
//! brush 100, the widest ball (Depth 1 ⇒ ρ = 16) — and time the KERNEL over the region a move actually
//! touches (the footprint grown by the reach), which is a few hundred texels square, NOT the whole canvas.
//! The "73 ms" this line has quoted was never pinned to a region; this pins it.

use super::super::Region;
use super::super::region::grow_region;
use super::*;

/// The per-move compute region the real render builds: the dab footprint (brush radius) grown twice by the
/// ball's reach, exactly as [`super::super::sculpt_inflate::render_inflate`] grows `rect → kr → cr`.
fn per_move_region(size: u32, brush_r: f32, rho: i64) -> Region {
    let reach = ((2 * rho * rho) as f64).sqrt().ceil() as u32;
    let mid = size / 2;
    let foot = Region {
        x: mid.saturating_sub(brush_r as u32),
        y: mid.saturating_sub(brush_r as u32),
        w: (brush_r as u32) * 2,
        h: (brush_r as u32) * 2,
    };
    let kr = grow_region(foot, reach, size, size).expect("kr");
    grow_region(kr, 2 * reach, size, size).expect("cr")
}

/// A relief the same shape the kill harness uses — cheap high-frequency terrain the ball has to chew.
fn kill_relief(size: u32) -> Vec<f32> {
    let n = (size * size) as usize;
    (0..n)
        .map(|i| {
            let x = (i as u32 % size) as f32;
            let y = (i as u32 / size) as f32;
            0.4 * ((x * 0.031).fract() + (y * 0.017).fract())
        })
        .collect()
}

/// **The exact ball over one region** — `max` over the disc of `pre + Depth·√(1 − d²/ρ²)`. Bounded support,
/// lands at zero on its own. This is the REFERENCE kernel; the probe times it, it is not shipped.
///
/// The two axes are not the same unit: `d` is texels, the lift is loads, so the ball is an ellipsoid in
/// (texel, load) space with horizontal radius `ρ` texels and vertical radius `Depth` loads
/// ([[feedback_geometry_over_mixed_units_needs_the_consumers_conversion]]).
fn exact_ball_region(pre: &[f32], size: u32, cr: Region, depth: f32, unit: f32) -> Vec<f32> {
    let rho = depth * unit;
    let r = rho.ceil() as i64;
    let sz = size as i64;
    let (cw, ch) = (cr.w as usize, cr.h as usize);
    let mut out = vec![0.0f32; cw * ch];
    // The disc offsets, precomputed once: this is the constant the O(area·ρ²) cost is made of.
    let mut disc: Vec<(i64, i64, f32)> = Vec::new();
    for dy in -r..=r {
        for dx in -r..=r {
            let d2 = (dx * dx + dy * dy) as f32;
            if d2 <= rho * rho {
                disc.push((dx, dy, depth * (1.0 - d2 / (rho * rho)).max(0.0).sqrt()));
            }
        }
    }
    for ry in 0..ch {
        let qy = cr.y as i64 + ry as i64;
        for rx in 0..cw {
            let qx = cr.x as i64 + rx as i64;
            let mut best = pre[(qy * sz + qx) as usize];
            for &(dx, dy, lift) in &disc {
                let (px, py) = (qx + dx, qy + dy);
                if px < 0 || py < 0 || px >= sz || py >= sz {
                    continue;
                }
                let v = pre[(py * sz + px) as usize] + lift;
                if v > best {
                    best = v;
                }
            }
            out[ry * cw + rx] = best;
        }
    }
    out
}

/// The exact bounded ball over a whole small canvas, returning the dilated height AND the packed source
/// offset (argmax) — everything the matter advection needs. Slow (`O(N·ρ²)`), for a TEST fixture only.
fn exact_ball_full(pre: &[f32], size: u32, depth: f32, unit: f32) -> (Vec<f32>, Vec<u32>) {
    let rho = depth * unit;
    let r = rho.ceil() as i64;
    let sz = size as i64;
    let n = (size * size) as usize;
    let (mut hbuf, mut sbuf) = (vec![0.0f32; n], vec![0u32; n]);
    let mut disc: Vec<(i64, i64, f32)> = Vec::new();
    for dy in -r..=r {
        for dx in -r..=r {
            let d2 = (dx * dx + dy * dy) as f32;
            if d2 <= rho * rho {
                disc.push((dx, dy, depth * (1.0 - d2 / (rho * rho)).max(0.0).sqrt()));
            }
        }
    }
    for qy in 0..sz {
        for qx in 0..sz {
            let qi = (qy * sz + qx) as usize;
            let (mut best, mut bsx, mut bsy) = (pre[qi], 0i64, 0i64);
            for &(dx, dy, lift) in &disc {
                let (px, py) = (qx + dx, qy + dy);
                if px < 0 || py < 0 || px >= sz || py >= sz {
                    continue;
                }
                let v = pre[(py * sz + px) as usize] + lift;
                if v > best {
                    best = v;
                    bsx = dx;
                    bsy = dy;
                }
            }
            hbuf[qi] = best;
            sbuf[qi] = super::super::sculpt_offset::pack_src(bsx, bsy);
        }
    }
    (hbuf, sbuf)
}

/// The **coverage** the matter advection produces from a height/argmax pair — the SAME rule as
/// [`super::super::sculpt_inflate::render_inflate`]: `cov = max(pre_cover, pre_cover[src] · t)`, where `t` is
/// the ball's fraction at this texel (its own profile — no taper, the true ball lands at zero by itself).
fn advect_cover(sbuf: &[u32], pre_cover: &[u8], size: u32, depth: f32, unit: f32) -> Vec<u8> {
    let rho = depth * unit;
    let sz = size as i64;
    let mut cov = pre_cover.to_vec();
    for qy in 0..sz {
        for qx in 0..sz {
            let qi = (qy * sz + qx) as usize;
            let (dx, dy) = super::super::sculpt_offset::unpack_src(sbuf[qi]);
            if dx == 0 && dy == 0 {
                continue;
            }
            let (sx, sy) = (qx + dx, qy + dy);
            if sx < 0 || sy < 0 || sx >= sz || sy >= sz {
                continue;
            }
            let d2 = (dx * dx + dy * dy) as f32;
            let t = (1.0 - d2 / (rho * rho)).max(0.0).sqrt(); // the true ball's own profile fraction
            let v = (f32::from(pre_cover[(sy * sz + sx) as usize]) * t) as u8;
            if v > cov[qi] {
                cov[qi] = v;
            }
        }
    }
    cov
}

/// DIAGNOSTIC — **does the BOUNDED true ball remove the cross's gash, or does the weak-at-rim arrival
/// reproduce it?** The decisive experiment my committed root-cause rests on. If the exact ball gashes too,
/// boundedness is not the whole fix and the coverage guard / taper is the real mechanism.
#[test]
#[ignore]
fn diag_does_the_bounded_ball_fix_the_cross() {
    use super::inflate_junction_probes as jx;
    let (mut cur, layer) = jx::the_cross_pub();
    let pre = heights_of(&cur, layer);
    let pre_cover = super::inflate_edge::covers_of(&cur, layer);
    let size = super::inflate_edge::SIZE;
    let unit = super::super::impasto_light::DEPTH_UNIT_PX;
    let depth = 1.0f32;

    // (1) the shipped kernel, via the real product path.
    jx::inflate_cross(&mut cur);
    let cov_cur = super::inflate_edge::covers_of(&cur, layer);

    // (2) the exact bounded ball, same ground.
    let (_hb, sb) = exact_ball_full(&pre, size, depth, unit);
    let cov_ball = advect_cover(&sb, &pre_cover, size, depth, unit);

    let glyph = |v: u8| match v {
        200..=255 => '#',
        128..=199 => '+',
        50..=127 => '-',
        1..=49 => '.',
        0 => ' ',
    };
    let cx = (size / 2) as usize;
    let cy = (size / 2) as usize;
    for (label, cov) in [
        ("PRE", &pre_cover),
        ("SHIPPED parabola", &cov_cur),
        ("EXACT bounded ball", &cov_ball),
    ] {
        println!("\n--- {label} --- armpit quadrant [cx..cx+52, cy..cy+42]");
        for y in cy..cy + 42 {
            let row: String = (cx..cx + 52)
                .map(|x| glyph(cov[y * size as usize + x]))
                .collect();
            println!("{y:3} |{row}|");
        }
    }
}

/// The exact bounded ball over a region, **parallelised over output rows** — disjoint rows, no reduction,
/// no RNG, byte-identical to the serial version: exactly the property ADR-0109 used to admit rayon for the
/// watercolor composite. This is the candidate SHIPPABLE kernel; the probe below times it.
fn exact_ball_region_par(pre: &[f32], size: u32, cr: Region, depth: f32, unit: f32) -> Vec<f32> {
    use rayon::prelude::*;
    let rho = depth * unit;
    let r = rho.ceil() as i64;
    let sz = size as i64;
    let (cw, ch) = (cr.w as usize, cr.h as usize);
    let mut disc: Vec<(i64, i64, f32)> = Vec::new();
    for dy in -r..=r {
        for dx in -r..=r {
            let d2 = (dx * dx + dy * dy) as f32;
            if d2 <= rho * rho {
                disc.push((dx, dy, depth * (1.0 - d2 / (rho * rho)).max(0.0).sqrt()));
            }
        }
    }
    let mut out = vec![0.0f32; cw * ch];
    out.par_chunks_mut(cw).enumerate().for_each(|(ry, row)| {
        let qy = cr.y as i64 + ry as i64;
        for (rx, cell) in row.iter_mut().enumerate() {
            let qx = cr.x as i64 + rx as i64;
            let mut best = pre[(qy * sz + qx) as usize];
            for &(dx, dy, lift) in &disc {
                let (px, py) = (qx + dx, qy + dy);
                if px < 0 || py < 0 || px >= sz || py >= sz {
                    continue;
                }
                let v = pre[(py * sz + px) as usize] + lift;
                if v > best {
                    best = v;
                }
            }
            *cell = best;
        }
    });
    out
}

/// DIAGNOSTIC — **the decisive number: exact ball vs separable parabola, per-move, over the real region.**
///
/// If the exact ball clears the kill criterion here, the "algorithm owed" evaporates and the fix is to ship
/// the exact ball. If it does not, this quantifies by how much, and a bounded FAST kernel is owed.
#[test]
#[ignore = "perf measurement — run with --release --ignored"]
fn diag_exact_ball_per_move_cost() {
    use std::time::Instant;
    let unit = super::super::impasto_light::DEPTH_UNIT_PX;
    let depth = 1.0f32;
    let rho = (depth * unit).ceil() as i64;
    println!("ball radius ρ = {rho} texels   (Depth {depth}, the widest)");
    println!("             region      | separable O(N) | exact ball O(area·ρ²) | ratio");
    for size in [2048u32, 4096] {
        let pre = kill_relief(size);
        let cr = per_move_region(size, 100.0, rho);
        let texels = (cr.w * cr.h) as f64;

        // Separable: build the peak field and run the shipped engine over `cr`.
        let a = 1.0 / (2.0 * depth * unit * unit);
        let g: Vec<f32> = {
            let mut v = vec![0.0f32; (cr.w * cr.h) as usize];
            for ry in 0..cr.h {
                for rx in 0..cr.w {
                    let gi = ((cr.y + ry) * size + cr.x + rx) as usize;
                    v[(ry * cr.w + rx) as usize] = pre[gi] + depth;
                }
            }
            v
        };
        let sep = |g: &[f32]| {
            let t0 = Instant::now();
            let _ = super::super::sculpt_offset::blob_dilate(g, cr.w, cr.h, a, true);
            t0.elapsed().as_secs_f64() * 1000.0
        };
        // Warm + best-of-3 (the region is small; a single run is jittery).
        let _ = sep(&g);
        let sep_ms = (0..3).map(|_| sep(&g)).fold(f64::MAX, f64::min);

        let ex = || {
            let t0 = Instant::now();
            let _ = exact_ball_region(&pre, size, cr, depth, unit);
            t0.elapsed().as_secs_f64() * 1000.0
        };
        let _ = ex();
        let exact_ms = (0..3).map(|_| ex()).fold(f64::MAX, f64::min);

        let par = || {
            let t0 = Instant::now();
            let _ = exact_ball_region_par(&pre, size, cr, depth, unit);
            t0.elapsed().as_secs_f64() * 1000.0
        };
        let _ = par();
        let par_ms = (0..3).map(|_| par()).fold(f64::MAX, f64::min);

        println!(
            "  @{size}: {}×{} ({:.0}k) | sep {sep_ms:6.2} | exact {exact_ms:7.2} ({:.0}×) | PARALLEL {par_ms:6.2} ms",
            cr.w,
            cr.h,
            texels / 1000.0,
            exact_ms / sep_ms
        );
    }
    println!(
        "\nkill criterion: ≤4 ms/move target, KILL at 8 (this is the kernel alone; the frame adds composite+light)"
    );
}
