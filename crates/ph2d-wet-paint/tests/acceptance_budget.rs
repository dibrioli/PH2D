//! SPEC §18.11 + §18.12 — the BINDING calibration: stroke budgets at two
//! radii, lane structure, and the §7 deposit-integral (with its uniformity
//! clause). These pin the perceived strength of a default stroke; they are
//! the tests that keep the bristle texture honest.

mod util;

use ph2d_wet_paint::brush::{BRISTLE_SIZE, BristleKnobs, build_bristle_texture, radial_falloff};
use ph2d_wet_paint::painter::Engine;
use util::{drive_stroke, sweep_nan};

struct BudgetResult {
    engine: Engine,
    mass: f64,
    water: f64,
    cov: f64,
    upper_share: f64,
    above5: u32,
    max_run: u32,
}

/// The §18.11 scripted stroke at a given brush size; measures budgets and
/// the §18.12 lane metrics.
fn budget_stroke(size_slider: f64, r: i64) -> BudgetResult {
    let mut e = Engine::new(300, 100);
    e.sliders.size = size_slider; // other sliders default (pressure 0.7, water 1.0)
    drive_stroke(&mut e, 20.0, 50.0, 280.0, 50.0, 4.0, 0); // 260 px @ 4 px/frame, sim paused
    let g = e.active_grid();
    let mut mass = 0.0f64;
    let mut water = 0.0f64;
    let mut band = 0u32;
    let mut touched = 0u32;
    for y in 1..=g.h as i64 {
        for x in 1..=g.w as i64 {
            let i = x as usize + y as usize * g.s;
            mass += g.susp[i] as f64 + g.sett[i] as f64;
            water += g.film[i] as f64;
            if (y - 50).abs() <= r && (20..=280).contains(&x) {
                band += 1;
                if g.susp[i] as f64 + g.sett[i] as f64 > 0.0 {
                    touched += 1;
                }
            }
        }
    }
    // Lane structure: suspended mass per row over the central 220 columns.
    let mut rows: Vec<f64> = Vec::new();
    for dy in -r..=r {
        let mut sum = 0.0f64;
        for x in 40..=259usize {
            sum += g.susp[x + (50 + dy) as usize * g.s] as f64;
        }
        rows.push(sum);
    }
    let max_row = rows.iter().cloned().fold(f64::MIN, f64::max);
    let mut upper = 0.0f64;
    let mut lower = 0.0f64;
    let mut above5 = 0u32;
    let mut run = 0u32;
    let mut max_run = 0u32;
    for (k, &row) in rows.iter().enumerate() {
        let dy = k as i64 - r;
        if dy < 0 {
            upper += row;
        } else if dy > 0 {
            lower += row;
        }
        if row > 0.05 * max_row {
            above5 += 1;
        }
        if row < 0.02 * max_row {
            run += 1;
            if run > max_run {
                max_run = run;
            }
        } else {
            run = 0;
        }
    }
    let cov = touched as f64 / band as f64;
    let upper_share = upper / (upper + lower);
    BudgetResult { engine: e, mass, water, cov, upper_share, above5, max_run }
}

#[test]
fn t11_stroke_budget_at_two_radii() {
    let b12 = budget_stroke(10.0 / 33.0, 12); // radius = 2 + slider*33 = 12
    let b25 = budget_stroke(0.7, 25); // the boot radius
    assert!(
        (b12.mass - 235_000.0).abs() <= 235_000.0 * 0.12,
        "r12 mass {} outside 235000 +-12%",
        b12.mass.round()
    );
    assert!(
        (b12.water - 245.0).abs() <= 245.0 * 0.12,
        "r12 water {:.1} outside 245 +-12%",
        b12.water
    );
    assert!(
        b12.cov >= 0.20 && b12.cov <= 0.27,
        "r12 coverage {:.1}% outside 20-27%",
        b12.cov * 100.0
    );
    assert!(
        (b25.mass - 890_000.0).abs() <= 890_000.0 * 0.15,
        "r25 mass {} outside 890000 +-15%",
        b25.mass.round()
    );
    assert!(
        (b25.water - 835.0).abs() <= 835.0 * 0.15,
        "r25 water {:.1} outside 835 +-15%",
        b25.water
    );
    assert!(
        b25.cov >= 0.33 && b25.cov <= 0.45,
        "r25 coverage {:.1}% outside 33-45%",
        b25.cov * 100.0
    );
    println!(
        "    r12: mass {} | water {:.1} | coverage {:.1}%",
        b12.mass.round(),
        b12.water,
        b12.cov * 100.0
    );
    println!(
        "    r25: mass {} | water {:.1} | coverage {:.1}%",
        b25.mass.round(),
        b25.water,
        b25.cov * 100.0
    );
    sweep_nan(b12.engine.active_grid(), "post-budget-r12");
    sweep_nan(b25.engine.active_grid(), "post-budget-r25");
}

#[test]
fn t11b_deposit_integral_and_uniformity() {
    // §7 deposit integral + its uniformity clause: over well-spread tile
    // positions the integral's mean holds and its minimum never collapses —
    // a brush of ANY radius finds the same strong-tip density anywhere.
    let tex = build_bristle_texture(BristleKnobs::default());
    let integral_at = |cx: i32, cy: i32| -> (f64, u32) {
        let mut integral = 0.0f64;
        let mut clearing = 0u32;
        let bmask = BRISTLE_SIZE as i32 - 1;
        for dy in -12..=12i32 {
            for dx in -12..=12i32 {
                let d2 = (dx * dx + dy * dy) as f64 / 144.0;
                if d2 >= 1.0 {
                    continue;
                }
                let f = radial_falloff(d2.sqrt(), 9.0);
                let t = tex
                    [(((cy + dy) & bmask) as usize) * BRISTLE_SIZE + (((cx + dx) & bmask) as usize)]
                    as f64;
                let v = (f * t * 2.415).min(1.0) - 0.3;
                if v > 0.0 {
                    integral += v;
                    clearing += 1;
                }
            }
        }
        (integral, clearing)
    };
    let (origin_integral, origin_clearing) = integral_at(0, 0);
    assert!(
        (origin_integral - 4.0).abs() <= 4.0 * 0.15,
        "origin deposit integral {origin_integral:.2} outside 4.0 +-15%"
    );
    assert!(
        (13..=23).contains(&origin_clearing),
        "origin clearing texels {origin_clearing} outside 13-23"
    );
    let mut g_sum = 0.0f64;
    let mut g_min = f64::INFINITY;
    for cy in [0, 43, 85] {
        for cx in [0, 43, 85] {
            let (v, _) = integral_at(cx, cy);
            g_sum += v;
            if v < g_min {
                g_min = v;
            }
        }
    }
    let g_mean = g_sum / 9.0;
    assert!(
        (g_mean - 4.0).abs() <= 4.0 * 0.25,
        "grid integral mean {g_mean:.2} outside 4.0 +-25%"
    );
    assert!(g_min >= 1.5, "grid integral min {g_min:.2} < 1.5 (uniformity)");
    println!(
        "    integral: origin {origin_integral:.2} ({origin_clearing} texels) | 3x3 grid mean {g_mean:.2} min {g_min:.2}"
    );
}

#[test]
fn t12_lane_structure_at_two_radii() {
    let b12 = budget_stroke(10.0 / 33.0, 12);
    let b25 = budget_stroke(0.7, 25);
    assert!(
        b12.upper_share >= 0.35 && b12.upper_share <= 0.65,
        "r12 upper half-band carries {:.1}% of off-path mass (need 35-65%)",
        b12.upper_share * 100.0
    );
    assert!(
        b25.upper_share >= 0.35 && b25.upper_share <= 0.65,
        "r25 upper half-band carries {:.1}% of off-path mass (need 35-65%)",
        b25.upper_share * 100.0
    );
    assert!(b12.above5 >= 7, "r12 rows >5% of max: {} of 25 (need >= 7)", b12.above5);
    assert!(b25.above5 >= 20, "r25 rows >5% of max: {} of 51 (need >= 20)", b25.above5);
    assert!(b12.max_run <= 6, "r12 longest <2% run: {} (need <= 6)", b12.max_run);
    assert!(b25.max_run <= 12, "r25 longest <2% run: {} (need <= 12)", b25.max_run);
    println!(
        "    r12: upper {:.1}% | rows>5% {}/25 | max gap {}",
        b12.upper_share * 100.0,
        b12.above5,
        b12.max_run
    );
    println!(
        "    r25: upper {:.1}% | rows>5% {}/51 | max gap {}",
        b25.upper_share * 100.0,
        b25.above5,
        b25.max_run
    );
}
