//! **The bounded-ball workshop** — the probes that DECIDED, and then SHIPPED, the TRUE BALL
//! (`super::super::sculpt_offset::blob_ball`). The parabola it replaced had **unbounded support**: a source
//! captured the envelope out to `√(H/a)` while it could only serve out to `ρ√2`, so a tall junction handed
//! its Voronoi cell a dead zone whose boundary is the white gash ([`super::inflate_junction_probes`]). A
//! **bounded** ball has `capture == reach` by construction — no dead zone, no gash — and these probes settled,
//! by measurement, the one question that decided the kernel: the exact ball is `O(area·ρ²)` (44 ms/move
//! serial) but embarrassingly parallel, and the shipped `blob_ball` (parallel over rows + the disc-list
//! reformulation) lands the widest case UNDER the kill. They mirror the kill harness
//! ([`super::session::sculpt_perf_kill_criterion`]) — 2048²/4096², brush 100, the widest ball (Depth 1 ⇒
//! ρ = 16) — and time the KERNEL over the region a move actually touches (the footprint grown by the reach).

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
    println!(
        "             region      | serial exact O(area·ρ²) | SHIPPED (parallel blob_ball) | speedup"
    );
    for size in [2048u32, 4096] {
        let pre = kill_relief(size);
        let amount = vec![1.0f32; (size * size) as usize]; // Filter Layer fills `amount` uniformly
        let cr = per_move_region(size, 100.0, rho);
        let texels = (cr.w * cr.h) as f64;

        let ex = || {
            let t0 = Instant::now();
            let _ = exact_ball_region(&pre, size, cr, depth, unit);
            t0.elapsed().as_secs_f64() * 1000.0
        };
        let _ = ex();
        let exact_ms = (0..3).map(|_| ex()).fold(f64::MAX, f64::min);

        // The SHIPPED kernel: `sculpt_offset::blob_ball`, the exact bounded ball parallelised over rows.
        let ship = || {
            let t0 = Instant::now();
            let _ = super::super::sculpt_offset::blob_ball(
                &pre, &amount, size, size, cr, depth, unit, true,
            );
            t0.elapsed().as_secs_f64() * 1000.0
        };
        let _ = ship();
        let ship_ms = (0..3).map(|_| ship()).fold(f64::MAX, f64::min);

        println!(
            "  @{size}: {}×{} ({:.0}k) | {exact_ms:9.2} ms          | {ship_ms:10.2} ms                 | {:.0}×",
            cr.w,
            cr.h,
            texels / 1000.0,
            exact_ms / ship_ms
        );
    }
    println!(
        "\nkill criterion: ≤4 ms/move target, KILL at 8 (this is the kernel alone; the frame adds composite+light)"
    );
}
