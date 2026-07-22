//! State fingerprint after a deterministic scripted session — the refactor
//! guard: any hot-loop rewrite (drying locals, flow hoists) must leave this
//! EXACT value untouched, because the engine is deterministic end to end and
//! "the JS is the spec" leaves no room for an ulp of drift.
//!
//! The pinned value was produced by the FIRST straight port (pre any perf
//! work) and cross-checked against it after each rewrite via git stash.

mod util;

use ph2d_wet_paint::painter::{Engine, Tool};
use util::drive_stroke;

/// FNV-1a over the f32 bit patterns of the fields that matter. The byte
/// ORDER is the original planar one (all R, then all G, then all B) so the
/// pin survives the interleaved-color layout change — the layout is not the
/// simulation.
fn fingerprint(e: &Engine) -> u64 {
    let g = e.active_grid();
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    fn eat(h: &mut u64, arr: &[f32]) {
        for v in arr {
            for b in v.to_bits().to_le_bytes() {
                *h ^= b as u64;
                *h = h.wrapping_mul(0x1000_0000_01b3);
            }
        }
    }
    fn eat_channel(h: &mut u64, arr: &[[f32; 3]], ch: usize) {
        for v in arr {
            for b in v[ch].to_bits().to_le_bytes() {
                *h ^= b as u64;
                *h = h.wrapping_mul(0x1000_0000_01b3);
            }
        }
    }
    eat(&mut h, &g.film);
    eat(&mut h, &g.susp);
    eat(&mut h, &g.sett);
    eat_channel(&mut h, &g.susp_rgb, 0);
    eat_channel(&mut h, &g.susp_rgb, 1);
    eat_channel(&mut h, &g.susp_rgb, 2);
    eat_channel(&mut h, &g.sett_rgb, 0);
    eat_channel(&mut h, &g.sett_rgb, 1);
    eat_channel(&mut h, &g.sett_rgb, 2);
    eat(&mut h, &g.vel_x);
    eat(&mut h, &g.vel_y);
    h
}

fn scripted_session(wet_lift: Option<f64>) -> Engine {
    // A session that exercises deposit + trail + drying + flow + advect +
    // projection + a wet pass + drip under tilt.
    let mut e = Engine::new(300, 200);
    if let Some(v) = wet_lift {
        e.set_knob(ph2d_wet_paint::tuning::Knob::WetLift, v);
    }
    drive_stroke(&mut e, 40.0, 60.0, 260.0, 90.0, 4.0, 30);
    e.tool = Tool::Wet;
    drive_stroke(&mut e, 40.0, 120.0, 260.0, 120.0, 5.0, 10);
    e.tool = Tool::Paint;
    e.sim.gravity_override = Some([0.0, 1.0]);
    drive_stroke(&mut e, 60.0, 40.0, 240.0, 40.0, 3.0, 80);
    e
}

#[test]
fn scripted_session_fingerprint_is_stable() {
    let fp = fingerprint(&scripted_session(None));
    // Pinned (see history below). If a rewrite moves this number, the
    // rewrite changed the simulation — find out why before touching the pin.
    assert_eq!(fp, PINNED, "session fingerprint drifted: {fp:#018x}");
}

/// Doc 23 §5 gate: `wetLift = 0` IS the pre-doc-23 model, to the byte — the
/// same scripted session reproduces the pin that stood before the active
/// lift landed. This is also the proof that extracting `lift_settled` out
/// of the drying pass was pure code motion (the passive re-wet runs in this
/// session and hashes identically).
#[test]
fn wet_lift_zero_is_the_old_model_to_the_byte() {
    let fp = fingerprint(&scripted_session(Some(0.0)));
    assert_eq!(
        fp, PINNED_PRE_DOC23,
        "wetLift=0 must reproduce the pre-doc-23 session: {fp:#018x}"
    );
}

// Pin history — every move names the SEMANTIC change that justified it:
// 0x1d26_6795_d687_a4c4  first straight port (survived the drying-locals,
//                        flow row-slice and interleaved-color rewrites —
//                        those were layout, not simulation).
// 0x6097_a692_a23d_bd5f  the port-verify parity FIXES (both move the Rust
//                        TOWARD the JS): advect's `film[i] += q00+q10+q01+q11`
//                        sums the q's before adding to the cell, and the
//                        paper lattices are Float32Array (each rng draw
//                        rounds to f32 before the noise sampler reads it).
// 0x99d8_891b_57a7_2abe  doc 23 P1: the Wet tool ACTIVELY lifts settled
//                        pigment through `lift_settled` (wetLift 0.25) —
//                        the session's Wet stroke now dissolves what the
//                        first stroke dried. wetLift=0 still reproduces
//                        the previous pin (gate above): the drift is the
//                        feature, not an accident.
const PINNED: u64 = 0x99d8_891b_57a7_2abe;
const PINNED_PRE_DOC23: u64 = 0x6097_a692_a23d_bd5f;
