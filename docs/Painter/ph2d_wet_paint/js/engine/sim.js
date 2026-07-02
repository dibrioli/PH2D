// Simulation orchestrator (spec section 5): fixed 40 Hz steps with per-pass
// cadences, plus the adaptive drying cadence.
//
// Per step (frame counter n, incremented first):
//   n % 2 == 0        rebuild the active region; if no fluid remains, idle
//   n % dryEvery == 0 drying / settle / re-wet
//   n % 4 == 0        flow-field build WITH the absorbency brake
//   other frames      cheap velocity smoothing, never braked (+ diffusion ext)
//   every frame       advection + gravity injection, then the drain boundaries
//   n % 3 == 0        pressure projection, then velocity boundaries again
//
// The caller (shell or test) owns the accumulator and the "paused while the
// pointer is down" rule; this module only advances single fixed steps.

import { makeColorMixer } from './colorops.js';
import {
  rebuildActiveRegion, buildFlowField, smoothVelocity, diffusionPass,
  advect, project, applyBoundaries,
} from './solver.js';
import { dryingPass, fastDry } from './drying.js';

export const SIM_HZ = 40;

export function createSim(grid, tuning) {
  return {
    grid,
    tuning,
    frame: 0,
    // Adaptive drying cadence, starts calm.
    dryEvery: 6,
    evapScale: 0.001,
    rewetBase: 0.0001,
    // Gravity: the tilt dial provides direction + magnitude; boots ON
    // pointing straight down at the gravity knob's magnitude.
    tiltOn: true,
    tiltDirX: 0,
    tiltDirY: 1,
    tiltScale: 1, // dial radius fraction relative to the knob value
    gravityOverride: null, // tests may pin an exact vector
    // Experimental color routing (K-M pigment mixing checkbox).
    kmMixing: false,
    // Force-skip every section-17 extension code path (neutrality test).
    extBypass: false,
  };
}

/** The effective gravity vector for this step. */
export function simGravity(sim) {
  if (sim.gravityOverride) return sim.gravityOverride;
  if (!sim.tiltOn) return { x: 0, y: 0 };
  const mag = sim.tuning.get('gravity') * sim.tiltScale;
  return { x: sim.tiltDirX * mag, y: sim.tiltDirY * mag };
}

/**
 * Snapshot every knob the solver reads into a flat params object, and attach
 * the color mixer. Done once per step so inner loops never touch the registry.
 */
export function gatherParams(sim) {
  const P = Object.assign({}, sim.tuning.values);
  P.mixColor = makeColorMixer(sim.kmMixing);
  P.kmMixing = sim.kmMixing; // the advection color mean also routes through K-M
  return P;
}

/**
 * Advance one fixed 40 Hz step. Returns false when the sim is idle (no fluid).
 */
export function simStep(sim) {
  const g = sim.grid;
  if (!g.hasFluid) return false;
  const P = gatherParams(sim);
  const grav = simGravity(sim);
  sim.frame++;
  const n = sim.frame;

  if (n % 2 === 0) {
    rebuildActiveRegion(g);
    if (!g.hasFluid) return false;
  }
  if (n % sim.dryEvery === 0) {
    // The evaporation / re-wet knobs are straight multipliers on the
    // cadence-adaptive scales.
    dryingPass(g, P, sim.evapScale * P.evaporation, sim.rewetBase * P.rewet, sim.extBypass);
  }
  if (n % 4 === 0) {
    buildFlowField(g, P, grav.x, grav.y, sim.extBypass);
  } else {
    smoothVelocity(g, P);
    if (!sim.extBypass && P.extDiffusion > 0) diffusionPass(g, P);
  }
  const vmax = advect(g, P, grav.x, grav.y);
  applyBoundaries(g, false);
  if (n % 3 === 0) {
    project(g, P);
    applyBoundaries(g, true);
  }
  // Adaptive drying cadence for the NEXT frames, from this advection's max
  // velocity component: calm water dries faster per pass, flowing water is
  // dried gently but often.
  if (vmax < 0.5) {
    sim.dryEvery = 6; sim.evapScale = 0.001; sim.rewetBase = 0.0001;
  } else {
    sim.dryEvery = 3; sim.evapScale = 0.00025; sim.rewetBase = 0.000025;
  }
  return true;
}

/** Fast dry action (spec section 12), routed here so it shares params. */
export function simFastDry(sim) {
  if (!sim.grid.hasFluid) return;
  const P = gatherParams(sim);
  fastDry(sim.grid, P, sim.rewetBase * P.rewet, sim.extBypass);
}
