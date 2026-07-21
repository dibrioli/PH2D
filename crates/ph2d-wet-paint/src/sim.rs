//! Simulation orchestrator (port of `sim.js`, SPEC §5): fixed 40 Hz steps
//! with per-pass cadences, plus the adaptive drying cadence.
//!
//! Per step (frame counter n, incremented first):
//!   n % 2 == 0        rebuild the active region; if no fluid remains, idle
//!   n % dry_every == 0 drying / settle / re-wet
//!   n % 4 == 0        flow-field build WITH the absorbency brake
//!   other frames      cheap velocity smoothing, never braked (+ diffusion ext)
//!   every frame       advection + gravity injection, then the drain boundaries
//!   n % 3 == 0        pressure projection, then velocity boundaries again
//!
//! The caller (facade or test) owns the accumulator and the "paused while the
//! pointer is down" rule; this module only advances single fixed steps.

use crate::colorops::ColorMix;
use crate::drying::{drying_pass, fast_dry};
use crate::grid::Grid;
use crate::solver::{
    advect, apply_boundaries, build_flow_field, diffusion_pass, project, rebuild_active_region,
    smooth_velocity,
};
use crate::tuning::{KNOB_COUNT, Knob, Tuning};

pub const SIM_HZ: f64 = 40.0;

/// Snapshot of every knob the solver reads, taken once per step so inner
/// loops never touch the registry (the JS `gatherParams`), plus the color
/// mixer routing.
pub struct Params {
    values: [f64; KNOB_COUNT],
    pub mix: ColorMix,
    /// The advection's incoming-color mean also routes through K–M when ON.
    pub km_mixing: bool,
}

impl Params {
    #[inline]
    pub fn k(&self, knob: Knob) -> f64 {
        self.values[knob as usize]
    }
}

/// Sim state that lives OUTSIDE the grid: cadence, tilt, experimental flags.
/// (The JS holds the grid reference here; in Rust the caller passes the grid
/// to [`sim_step`] — the facade owns which layer's grid simulates.)
pub struct Sim {
    pub frame: u64,
    // Adaptive drying cadence, starts calm.
    pub dry_every: u64,
    pub evap_scale: f64,
    pub rewet_base: f64,
    // Gravity: the tilt dial provides direction + magnitude; boots ON
    // pointing straight down at the gravity knob's magnitude.
    pub tilt_on: bool,
    pub tilt_dir_x: f64,
    pub tilt_dir_y: f64,
    /// Dial radius fraction relative to the knob value.
    pub tilt_scale: f64,
    /// Tests may pin an exact vector.
    pub gravity_override: Option<[f64; 2]>,
    /// Experimental color routing (K–M pigment mixing checkbox).
    pub km_mixing: bool,
    /// Force-skip every SPEC §17 extension code path (neutrality test §18.10).
    pub ext_bypass: bool,
}

impl Default for Sim {
    fn default() -> Self {
        Sim {
            frame: 0,
            dry_every: 6,
            evap_scale: 0.001,
            rewet_base: 0.0001,
            tilt_on: true,
            tilt_dir_x: 0.0,
            tilt_dir_y: 1.0,
            tilt_scale: 1.0,
            gravity_override: None,
            km_mixing: false,
            ext_bypass: false,
        }
    }
}

impl Sim {
    /// The effective gravity vector for this step.
    pub fn gravity(&self, tuning: &Tuning) -> [f64; 2] {
        if let Some(g) = self.gravity_override {
            return g;
        }
        if !self.tilt_on {
            return [0.0, 0.0];
        }
        let mag = tuning.get(Knob::Gravity) * self.tilt_scale;
        [self.tilt_dir_x * mag, self.tilt_dir_y * mag]
    }

    /// Snapshot every knob + the color routing (the JS `gatherParams`).
    pub fn gather_params(&self, tuning: &Tuning) -> Params {
        Params {
            values: tuning.snapshot(),
            mix: if self.km_mixing {
                ColorMix::Km
            } else {
                ColorMix::Plain
            },
            km_mixing: self.km_mixing,
        }
    }
}

/// Advance one fixed 40 Hz step. Returns false when the sim is idle (no fluid).
pub fn sim_step(sim: &mut Sim, g: &mut Grid, tuning: &Tuning) -> bool {
    if !g.has_fluid {
        return false;
    }
    let p = sim.gather_params(tuning);
    let grav = sim.gravity(tuning);
    sim.frame += 1;
    let n = sim.frame;

    if n % 2 == 0 {
        rebuild_active_region(g);
        if !g.has_fluid {
            return false;
        }
    }
    if n % sim.dry_every == 0 {
        // The evaporation / re-wet knobs are straight multipliers on the
        // cadence-adaptive scales.
        drying_pass(
            g,
            &p,
            sim.evap_scale * p.k(Knob::Evaporation),
            sim.rewet_base * p.k(Knob::Rewet),
            sim.ext_bypass,
        );
    }
    if n % 4 == 0 {
        build_flow_field(g, &p, grav[0], grav[1], sim.ext_bypass);
    } else {
        smooth_velocity(g, &p);
        if !sim.ext_bypass && p.k(Knob::ExtDiffusion) > 0.0 {
            diffusion_pass(g, &p);
        }
    }
    let vmax = advect(g, &p, grav[0], grav[1]);
    apply_boundaries(g, false);
    if n % 3 == 0 {
        project(g, &p);
        apply_boundaries(g, true);
    }
    // Adaptive drying cadence for the NEXT frames, from this advection's max
    // velocity component: calm water dries faster per pass, flowing water is
    // dried gently but often.
    if vmax < 0.5 {
        sim.dry_every = 6;
        sim.evap_scale = 0.001;
        sim.rewet_base = 0.0001;
    } else {
        sim.dry_every = 3;
        sim.evap_scale = 0.00025;
        sim.rewet_base = 0.000025;
    }
    true
}

/// Fast dry action (SPEC §12), routed here so it shares params.
pub fn sim_fast_dry(sim: &mut Sim, g: &mut Grid, tuning: &Tuning) {
    if !g.has_fluid {
        return;
    }
    let p = sim.gather_params(tuning);
    fast_dry(g, &p, sim.rewet_base * p.k(Knob::Rewet), sim.ext_bypass);
}
