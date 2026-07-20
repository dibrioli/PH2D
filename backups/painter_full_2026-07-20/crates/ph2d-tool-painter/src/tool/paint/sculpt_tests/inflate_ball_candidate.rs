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

/// **The reach bound deletes only DEAD work — the ball is byte-identical without it.**
///
/// `blob_ball` walks its disc from the centre outwards and stops at `dq >= A(q)²`, where `A(q)` is the
/// largest `amount` anywhere in the texel's box. The claim is not "close enough": a source contributes only
/// where `a_p² > dq`, and no `a_p` in the disc exceeds `A(q)`, so every offset past that point fails the
/// in-ball test the old loop would have run. It is the same maximum over the same candidate set, with the
/// non-candidates never loaded.
///
/// Measured on the kill fixture before the bound existed: **35% of texels had `A(q) = 0`** — an entire
/// 804-offset disc walked to find nothing — and only **32% of 73M taps** could ever have contributed.
///
/// The oracle here is the shipped kernel **with the optimisation removed**, which is the only oracle that
/// answers the question being asked. It checks BOTH outputs: the height (a max, so order cannot matter) and
/// the packed argmax (where the matter comes from — order CAN matter there, on an exact tie, which is why
/// the disc is sorted stably).
///
/// **Mutation that must bleed:** relax `A(q)` to the texel's own `amount` instead of the box max — a bound
/// that looks reasonable and is wrong, because the source that lifts a texel is a NEIGHBOUR. (Verified
/// RED.)
///
/// **A mutation that does NOT bleed, and why that is recorded rather than fixed:** swapping the disc's
/// `sort_by` for `sort_unstable_by` leaves this gate green. That is not the fixture being weak about ties
/// — it has a flat plateau precisely to generate them — it is `sort_unstable_by` happening to produce the
/// same order here (pdqsort is not adversarial on a nearly-sorted key). The tie-break order rests on
/// `slice::sort_by` being **stable by the standard library's contract**, which is a guarantee no fixture
/// can stand in for. Do not "strengthen" this gate by hunting an input where pdqsort differs: that would
/// pin an implementation detail of the sort, not the property.
#[test]
fn the_reach_bound_is_exact_the_ball_is_byte_identical_without_it() {
    const SIZE: u32 = 96;
    let n = (SIZE * SIZE) as usize;
    // `amount` must contain the whole range the bound reasons about, or the gate proves nothing: dead
    // neighbourhoods (where it fires immediately), partial touches (where it fires part-way through the
    // disc), and saturation (where it never fires at all).
    let amount: Vec<f32> = (0..n)
        .map(|i| {
            let (x, y) = ((i as u32 % SIZE) as f32, (i as u32 / SIZE) as f32);
            if x < 20.0 || y < 20.0 {
                0.0 // untouched: A(q) = 0 over a wide band, the 35% case
            } else if x > 70.0 {
                1.0 // saturated: the bound never fires
            } else {
                ((x - 20.0) / 50.0).clamp(0.0, 1.0) * ((y - 20.0) / 60.0).clamp(0.0, 1.0)
            }
        })
        .collect();
    // Varied relief, including a flat plateau — the tie generator. On flat paint with uniform `amount`
    // every source at the same distance lifts by the identical float, so the argmax is decided purely by
    // iteration order, and this is where an unstable sort would show.
    let pre: Vec<f32> = (0..n)
        .map(|i| {
            let (x, y) = ((i as u32 % SIZE) as f32, (i as u32 / SIZE) as f32);
            if (30.0..60.0).contains(&x) && (30.0..60.0).contains(&y) {
                0.5 // the plateau
            } else {
                0.4 * ((x * 0.031).fract() + (y * 0.017).fract())
            }
        })
        .collect();

    for &depth in &[1.0f32, 0.5, -0.75] {
        let unit = 16.0f32;
        let rho = depth.abs() * unit;
        let r = rho.ceil() as i64;
        let cr = Region {
            x: 8,
            y: 8,
            w: SIZE - 16,
            h: SIZE - 16,
        };
        let (fast_h, fast_s) = super::super::sculpt_offset::blob_ball(
            &pre,
            &amount,
            SIZE,
            SIZE,
            cr,
            depth,
            unit,
            depth > 0.0,
        );
        let (slow_h, slow_s) = brute_ball(&pre, &amount, SIZE, cr, depth, unit, r);
        let hdiff = fast_h
            .iter()
            .zip(&slow_h)
            .filter(|(a, b)| a.to_bits() != b.to_bits())
            .count();
        assert_eq!(
            hdiff, 0,
            "depth {depth}: the bounded walk changed {hdiff} heights — it is supposed to skip only \
             offsets that fail the in-ball test anyway"
        );
        let sdiff = fast_s.iter().zip(&slow_s).filter(|(a, b)| a != b).count();
        assert_eq!(
            sdiff, 0,
            "depth {depth}: the bounded walk moved {sdiff} argmax sources. The matter follows this \
             pointer, so a different winner is a different COLOUR arriving — the tie-break order has to \
             survive the sort"
        );
    }
}

/// The shipped kernel with the reach bound removed: every offset in the disc, in raster order. The oracle
/// for the gate above — deliberately not a re-derivation of the ball's maths, just the same maths without
/// the skip.
fn brute_ball(
    pre: &[f32],
    amount: &[f32],
    size: u32,
    cr: Region,
    depth: f32,
    unit: f32,
    r: i64,
) -> (Vec<f32>, Vec<u32>) {
    let rho = depth.abs() * unit;
    let (mag, inv_rho2) = (depth.abs(), 1.0 / (rho * rho));
    let sz = i64::from(size);
    let (cw, ch) = (cr.w as usize, cr.h as usize);
    let dilate = depth > 0.0;
    let mut disc: Vec<(i64, i64, f32)> = Vec::new();
    for dy in -r..=r {
        for dx in -r..=r {
            let dq = (dx * dx + dy * dy) as f32 * inv_rho2;
            if dq <= 1.0 {
                disc.push((dx, dy, dq));
            }
        }
    }
    let (mut hbuf, mut sbuf) = (vec![0.0f32; cw * ch], vec![0u32; cw * ch]);
    for ry in 0..ch {
        let qy = i64::from(cr.y) + ry as i64;
        for rx in 0..cw {
            let qx = i64::from(cr.x) + rx as i64;
            let (mut best, mut bdx, mut bdy) = (pre[(qy * sz + qx) as usize], 0i64, 0i64);
            for &(dx, dy, dq) in &disc {
                let (px, py) = (qx + dx, qy + dy);
                if px < 0 || px >= sz || py < 0 || py >= sz {
                    continue;
                }
                let pi = (py * sz + px) as usize;
                let a_p = amount[pi].clamp(0.0, 1.0);
                let arg = a_p * a_p - dq;
                if arg <= 0.0 {
                    continue;
                }
                let lift = mag * arg.sqrt();
                let v = if dilate {
                    pre[pi] + lift
                } else {
                    pre[pi] - lift
                };
                if if dilate { v > best } else { v < best } {
                    best = v;
                    bdx = dx;
                    bdy = dy;
                }
            }
            hbuf[ry * cw + rx] = best;
            sbuf[ry * cw + rx] = super::super::sculpt_offset::pack_src(bdx, bdy);
        }
    }
    (hbuf, sbuf)
}

/// **The reach bound actually SKIPS work — counted, not timed.**
///
/// Its sibling above proves the bound changes no pixel; this one proves it is worth having. Deliberately a
/// COUNT and not a stopwatch: the shipped wall-clock probes in this file take a min-of-3 and still swing by
/// 3x on a loaded machine (measured 2026-07-18: the UNCHANGED kernel read 10 ms where this file's own docs
/// record 3), so a millisecond bar here would pin the state of the box's thermals. How many disc offsets
/// the walk is allowed to visit is a property of the arithmetic and the same on every machine, forever.
///
/// Two facts, and the first is the one that pays:
///  - an untouched neighbourhood admits **zero** offsets. `A(q) = 0` means no source anywhere in the disc
///    can satisfy `a_p² > dq`, so the whole 800-offset walk collapses on its first comparison. On the kill
///    fixture that was **35% of all texels**, each of them previously walked in full to find nothing.
///  - over a realistic tapering stroke the admitted fraction is well under half — because `amount` falls
///    off from the stroke's spine, and a source that only half-touched the canvas only reaches half as far.
///
/// **Mutation that must bleed:** make `box_max` return `1.0` everywhere (the bound that is always true and
/// never useful) — the dead neighbourhood then admits the whole disc.
#[test]
fn the_reach_bound_admits_only_the_offsets_that_could_contribute() {
    const SIZE: u32 = 128;
    let n = (SIZE * SIZE) as usize;
    let (depth, unit) = (1.0f32, 16.0f32);
    let rho = depth * unit;
    let r = rho.ceil() as i64;
    // A stroke, not a fill: `amount` peaks on the spine and tapers to nothing, which is what a dab's
    // falloff lays down and what the bound reads. (A uniform fill — Filter Layer — is the case where the
    // bound never fires at all, and it is measured NOT to regress by `diag_exact_ball_per_move_cost`.)
    let amount: Vec<f32> = (0..n)
        .map(|i| {
            let (x, y) = ((i as u32 % SIZE) as f32, (i as u32 / SIZE) as f32);
            let d = ((y - 64.0) / 30.0).abs().hypot(((x - 64.0) / 45.0).abs());
            (1.0 - d).clamp(0.0, 1.0)
        })
        .collect();
    let cr = Region {
        x: 0,
        y: 0,
        w: SIZE,
        h: SIZE,
    };
    let abuf = super::super::sculpt_offset::box_max(&amount, SIZE, SIZE, cr, r);

    // The disc the walk would visit without the bound.
    let inv_rho2 = 1.0 / (rho * rho);
    let disc: Vec<f32> = (-r..=r)
        .flat_map(|dy| (-r..=r).map(move |dx| (dx * dx + dy * dy) as f32 * inv_rho2))
        .filter(|dq| *dq <= 1.0)
        .collect();
    let full = disc.len();
    assert!(
        full > 700,
        "precondition: the widest ball is a wide disc ({full})"
    );

    let (mut admitted, mut total, mut dead_texels, mut dead_admitted) = (0u64, 0u64, 0u64, 0u64);
    for (i, a) in abuf.iter().enumerate() {
        let a2 = a * a;
        let adm = disc.iter().filter(|dq| **dq < a2).count() as u64;
        admitted += adm;
        total += full as u64;
        if *a == 0.0 {
            dead_texels += 1;
            dead_admitted += adm;
        }
        let _ = i;
    }
    assert!(
        dead_texels > 1_000,
        "the fixture must CONTAIN untouched ground for this to mean anything (only {dead_texels})"
    );
    assert_eq!(
        dead_admitted, 0,
        "an untouched neighbourhood must admit not one offset — that is the 35% of the kill fixture \
         whose entire disc was being walked to find nothing"
    );
    let frac = admitted as f64 / total as f64;
    assert!(
        frac < 0.5,
        "the bound admitted {:.1}% of the disc over a tapering stroke — it is supposed to shrink the \
         walk with the falloff, and a bound that admits nearly everything is a bound that is not firing",
        frac * 100.0
    );
    println!(
        "reach bound admits {:.1}% of {full} offsets (dead texels: {dead_texels})",
        frac * 100.0
    );
}
