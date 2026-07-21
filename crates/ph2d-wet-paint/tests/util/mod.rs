//! Shared drivers for the SPEC §18 acceptance suite — the Rust port of the
//! reference `test/smoke.mjs` harness.

use ph2d_wet_paint::grid::Grid;
use ph2d_wet_paint::painter::Engine;

/// Drive a straight scripted stroke at a fixed px/frame speed (40 Hz clock),
/// exactly like the reference harness: one pointer sample per step, sim
/// stepped only when the engine says it should (paused while down except for
/// blow), then the release tail, then `sim_after` extra steps.
pub fn drive_stroke(
    engine: &mut Engine,
    x0: f64,
    y0: f64,
    x1: f64,
    y1: f64,
    px_per_frame: f64,
    sim_after: usize,
) {
    let dx = x1 - x0;
    let dy = y1 - y0;
    let len = (dx * dx + dy * dy).sqrt();
    let (ux, uy) = if len > 0.0 {
        (dx / len, dy / len)
    } else {
        (0.0, 0.0)
    };
    engine.pointer_down(x0, y0, None);
    let mut travelled = 0.0;
    let mut x = x0;
    let mut y = y0;
    while travelled < len {
        travelled = (travelled + px_per_frame).min(len);
        x = x0 + ux * travelled;
        y = y0 + uy * travelled;
        engine.pointer_frame(x, y);
        if engine.sim_should_run() {
            engine.step_simulation();
        }
    }
    engine.pointer_up();
    while engine.release_frame(x, y) { /* release tail ticks at 40 Hz */ }
    for _ in 0..sim_after {
        engine.step_simulation();
    }
}

pub fn has_nan(arr: &[f32]) -> bool {
    arr.iter().any(|v| v.is_nan())
}

/// NaN sweep over every float field the reference harness checks.
pub fn sweep_nan(g: &Grid, label: &str) {
    let fields: [(&str, &[f32]); 5] = [
        ("film", &g.film),
        ("susp", &g.susp),
        ("sett", &g.sett),
        ("vel_x", &g.vel_x),
        ("vel_y", &g.vel_y),
    ];
    for (name, arr) in fields {
        assert!(!has_nan(arr), "{label}: NaN found in {name}");
    }
    for (name, arr) in [("susp_rgb", &g.susp_rgb), ("sett_rgb", &g.sett_rgb)] {
        let nan = arr.iter().any(|c| c.iter().any(|v| v.is_nan()));
        assert!(!nan, "{label}: NaN found in {name}");
    }
}
