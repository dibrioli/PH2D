//! [`WatercolorParams`] — the per-brush watercolor tuning exposed to the artist
//! (ADR-0079). The serializable brush-facing DTO for the 19 solver controls (8 gated
//! diffusion-advection, 3 deposition, 4 shallow-water, 1 capillary, 1 sharpness, 1 lift,
//! 1 capillary-branching), kept SEPARATE from the solver-internal
//! [`crate::diffusion::DiffusionParams`] so the brush-file contract is insulated from solver
//! churn — [`WatercolorParams::to_diffusion`] maps 1:1.
//!
//! [`WatercolorParams::CONTROLS`] is the single source of truth for the UI: label +
//! physical `[min,max]` range per control, indexed `0..19`. The Brush Studio panel
//! iterates it to lay out the "Watercolor" subsection; the tool maps a slider's
//! NodeId → index and writes via [`WatercolorParams::set_normalized`]. Keeping it here
//! (not duplicated in the panel + tool) means one place defines what a control is.

use crate::diffusion::DiffusionParams;
use serde::{Deserialize, Serialize};

/// A single watercolor control's display label + physical value range (the slider maps
/// `0..1` ↔ `[min, max]`). The order of [`WatercolorParams::CONTROLS`] is the contract
/// index used by the panel + tool — APPEND only.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct WatercolorControl {
    pub label: &'static str,
    pub min: f32,
    pub max: f32,
}

/// Per-brush watercolor tuning (ADR-0079) — the 19 solver controls the artist drives via
/// the Brush Studio "Watercolor" subsection. Serializable like every other brush param so
/// the brush-file/MCP/replay carry it. [`Self::to_diffusion`] projects onto the solver's
/// [`DiffusionParams`]. Cap ≤ 20 (20 used — +`lift` ADR-0081, +`capillary_branching` ADR-0082,
/// +`water` water-brush/rewetting;
/// ADR-0079-amendment-1) — gate `architecture_painter_contract_surface`.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub struct WatercolorParams {
    // ── Gated diffusion-advection (the base wash) ──
    pub diffusivity: f32,
    pub evaporation: f32,
    pub downhill: f32,
    pub flow_outward: f32,
    pub w_lo: f32,
    pub w_hi: f32,
    pub perm_valley: f32,
    pub perm_crest: f32,
    // ── Pigment deposition (ADR-0078 S3) ──
    pub deposition: f32,
    pub deposition_dry: f32,
    pub granulation: f32,
    // ── Shallow-water velocity layer (ADR-0078 S3d) ──
    pub velocity: f32,
    pub viscosity: f32,
    pub drag: f32,
    pub pressure: f32,
    // ── Capillary fringe (ADR-0078 S5) ──
    pub capillary: f32,
    // ── Advection sharpness (ADR-0078 S5c — BFECC/MacCormack) ──
    pub sharpness: f32,
    // ── Lift (ADR-0081) — re-wetting re-mobilizes non-staining deposited pigment ──
    pub lift: f32,
    // ── Branched capillary fringe (ADR-0082) — fiber-channeled conductance suppression ──
    pub capillary_branching: f32,
    /// **Water brush / rewetting** ∈ [0,1] — scales each dab's PIGMENT deposit by `1 − water`
    /// (water deposit stays full). 1 = pure water: pre-wet, soften, bloom, lift — without
    /// depositing colour. Tool-side (dab emission), no [`DiffusionParams`] twin. Default 0
    /// = the validated paint look bit-for-bit.
    pub water: f32,
    // === 20/20 used (cap ≤ 20) — raise needs an ADR ===
}

impl Default for WatercolorParams {
    /// The **validated watercolor preset** — deposition + shallow-water ON with the tuned
    /// values (the live look the Enio ratified), NOT the all-off `DiffusionParams::default`.
    /// So flipping `fluid_enabled` on gives a good wash immediately; the sliders start here.
    /// Mirrors `ph2d_painter_fluid::WATERCOLOR_*` (kept in sync; that crate can't be a dep
    /// here, so the numbers are duplicated with this note).
    fn default() -> Self {
        let d = DiffusionParams::default(); // the 8 base values (deposition/velocity = 0)
        Self {
            diffusivity: d.diffusivity,
            evaporation: d.evaporation,
            // Downhill (paper-slope channeling) is OFF in the preset — it imprints the paper
            // grain as mottling (ADR-0079); the artist opts in via the "Downhill" slider.
            downhill: 0.0,
            flow_outward: d.flow_outward,
            w_lo: d.w_lo,
            w_hi: d.w_hi,
            perm_valley: d.perm_valley,
            perm_crest: d.perm_crest,
            // Deposition preset (ADR-0078 S3c WATERCOLOR_DEPOSITION_*).
            deposition: 0.012,
            deposition_dry: 0.10,
            granulation: 1.4,
            // Shallow-water preset (ADR-0078 S3d WATERCOLOR_* after the 2026-06-08 tuning).
            velocity: 1.3,
            viscosity: 0.18,
            drag: 0.1,
            pressure: 0.3,
            // Capillary fringe preset (ADR-0078 S5) — a clearly visible soft wet-edge ON by
            // default so the watercolor brush shows the signature *coloured* feathery fringe out
            // of the box (the water wick carries a thread of pigment). The artist tunes it
            // 0..0.24 via the "Capillary" slider (0 = the harder wet-gate edge).
            capillary: 0.15,
            // Sharpness preset (ADR-0078 S5c — BFECC/MacCormack correction). **Watercolor v2
            // (ADR-0085): default OFF (0.0)** — `sharpness > 0` TRIPLES the most expensive sim
            // pass (the 32-channel velocity advect: forward + reverse + correct), the dominant
            // per-cell cost while painting, for only a modest crispness gain (validated visually
            // 2026-06-11). The artist opts back in via the "Sharpness" slider. 0 = soft
            // first-order advect; 1 = max MacCormack.
            sharpness: 0.0,
            // Lift OFF by default (ADR-0081) — the lift pass is dormant; the artist opts in via
            // the "Lift" slider (re-wetting then reactivates dried non-staining pigment).
            lift: 0.0,
            // Branched capillary OFF by default (ADR-0082) — opt-in; 0 = the smooth isotropic
            // fringe bit-for-bit. The artist raises "Branching" for the lobed/dendritic fringe.
            capillary_branching: 0.0,
            // Water brush OFF by default — a paint brush deposits its full pigment load.
            water: 0.0,
        }
    }
}

impl WatercolorParams {
    /// The control descriptors (label + range), indexed `0..19` — the single source the
    /// Brush Studio panel + the tool's slider→param mapping both read. **APPEND only**
    /// (the index is the panel/tool contract). Ranges bound each slider's physical value;
    /// the preset defaults all fall inside them.
    pub const CONTROLS: [WatercolorControl; 20] = [
        // CFL-bounded (diffusivity/viscosity ≤ 0.24) keep their max; the rest were widened
        // (2026-06-08 Enio: several too subtle) so each slider has visible headroom.
        WatercolorControl {
            label: "Diffusivity",
            min: 0.0,
            max: 0.24,
        },
        WatercolorControl {
            label: "Evaporation",
            min: 0.0,
            max: 0.08,
        },
        WatercolorControl {
            label: "Downhill",
            min: 0.0,
            max: 0.3,
        },
        WatercolorControl {
            label: "Bleed",
            min: 0.0,
            max: 2.0,
        },
        WatercolorControl {
            label: "Wet Gate Lo",
            min: 0.0,
            max: 0.5,
        },
        WatercolorControl {
            label: "Wet Gate Hi",
            min: 0.0,
            max: 1.0,
        },
        WatercolorControl {
            label: "Perm Valley",
            min: 0.0,
            max: 1.0,
        },
        WatercolorControl {
            label: "Perm Crest",
            min: 0.0,
            max: 1.0,
        },
        WatercolorControl {
            label: "Deposition",
            min: 0.0,
            max: 0.3,
        },
        WatercolorControl {
            label: "Edge Darkening",
            min: 0.0,
            max: 1.0,
        },
        WatercolorControl {
            label: "Granulation",
            min: 0.0,
            max: 8.0,
        },
        WatercolorControl {
            label: "Flow Velocity",
            min: 0.0,
            max: 3.0,
        },
        WatercolorControl {
            label: "Viscosity",
            min: 0.0,
            max: 0.24,
        },
        WatercolorControl {
            label: "Drag",
            min: 0.0,
            max: 1.0,
        },
        WatercolorControl {
            label: "Backrun",
            min: 0.0,
            max: 2.0,
        },
        // CFL-bounded (≤ 0.24, like Diffusivity/Viscosity) — the outward water-wick rate that
        // sets how far the soft fringe creeps past the painted area (ADR-0078 S5).
        WatercolorControl {
            label: "Capillary",
            min: 0.0,
            max: 0.24,
        },
        // BFECC/MacCormack 2nd-order advection (ADR-0078 S5c): 0 = soft first-order upwind
        // (smeary), 1 = sharp error-compensated transport (crisp flow + backruns). >1 OVER-
        // corrects (more aggressive anti-diffusion). Capped at 2.5 — beyond that the
        // over-correction starts to posterize into artefacts (Enio 2026-06-09); the limiter
        // keeps it stable, but 2.5 is the usable ceiling. 0 = the shipped look bit-for-bit.
        WatercolorControl {
            label: "Sharpness",
            min: 0.0,
            max: 2.5,
        },
        // Lift (ADR-0081/0084): re-wetting re-mobilizes dried/backdrop pigment (paper-reveal).
        // 0 = off (the validated look bit-for-bit). The rate applies PER SUBSTEP and saturates
        // geometrically (`lifted_frac += rate·(1−lifted_frac)` over many substeps per stroke), so
        // the perceptual range is tiny: Enio's smoke (2026-06-09) found 0.025 already gives the
        // FULL effect — the slider maps onto exactly that usable band instead of wasting 97.5%
        // of its travel past saturation.
        WatercolorControl {
            label: "Lift",
            min: 0.0,
            max: 0.025,
        },
        // Branched capillary fringe (ADR-0082): fiber-channeled suppression of the capillary
        // conductance → a lobed/dendritic fringe instead of the smooth ring. 0 = off (the
        // isotropic capillary bit-for-bit); 1 = max suppression in the paper valleys. Opt-in.
        WatercolorControl {
            label: "Branching",
            min: 0.0,
            max: 1.0,
        },
        // Water brush / rewetting (the wet-on-wet workflow staple): scales the PIGMENT each dab
        // deposits by `1 − water` while the WATER deposit stays full. 0 = normal paint (the
        // validated look bit-for-bit); 1 = PURE WATER — pre-wet the paper for soft wet-on-wet
        // blooms, soften/bleed existing wet edges, and (with Lift > 0) lift dry paint without
        // depositing any colour. Continuous: a damp 0.5 brush paints a half-load dilute wash.
        // Acts at dab emission (the tool), NOT in the solver — no DiffusionParams twin.
        WatercolorControl {
            label: "Water",
            min: 0.0,
            max: 1.0,
        },
    ];

    /// Number of artist-facing controls (= `CONTROLS.len()`).
    pub const COUNT: usize = Self::CONTROLS.len();

    /// Read control `i`'s raw physical value (panics if `i >= COUNT`).
    #[must_use]
    pub fn get(&self, i: usize) -> f32 {
        match i {
            0 => self.diffusivity,
            1 => self.evaporation,
            2 => self.downhill,
            3 => self.flow_outward,
            4 => self.w_lo,
            5 => self.w_hi,
            6 => self.perm_valley,
            7 => self.perm_crest,
            8 => self.deposition,
            9 => self.deposition_dry,
            10 => self.granulation,
            11 => self.velocity,
            12 => self.viscosity,
            13 => self.drag,
            14 => self.pressure,
            15 => self.capillary,
            16 => self.sharpness,
            17 => self.lift,
            18 => self.capillary_branching,
            19 => self.water,
            _ => panic!("watercolor control index {i} out of range"),
        }
    }

    /// Write control `i`'s raw physical value (clamped to its range; panics if out of range).
    pub fn set(&mut self, i: usize, v: f32) {
        let c = &Self::CONTROLS[i];
        let v = v.clamp(c.min, c.max);
        match i {
            0 => self.diffusivity = v,
            1 => self.evaporation = v,
            2 => self.downhill = v,
            3 => self.flow_outward = v,
            4 => self.w_lo = v,
            5 => self.w_hi = v,
            6 => self.perm_valley = v,
            7 => self.perm_crest = v,
            8 => self.deposition = v,
            9 => self.deposition_dry = v,
            10 => self.granulation = v,
            11 => self.velocity = v,
            12 => self.viscosity = v,
            13 => self.drag = v,
            14 => self.pressure = v,
            15 => self.capillary = v,
            16 => self.sharpness = v,
            17 => self.lift = v,
            18 => self.capillary_branching = v,
            19 => self.water = v,
            _ => panic!("watercolor control index {i} out of range"),
        }
    }

    /// Control `i`'s value normalized to `0..1` over its `[min, max]` range (for the slider
    /// position).
    #[must_use]
    pub fn normalized(&self, i: usize) -> f32 {
        let c = &Self::CONTROLS[i];
        ((self.get(i) - c.min) / (c.max - c.min)).clamp(0.0, 1.0)
    }

    /// Set control `i` from a normalized `0..1` slider value (mapped onto `[min, max]`).
    pub fn set_normalized(&mut self, i: usize, v01: f32) {
        let c = &Self::CONTROLS[i];
        self.set(i, c.min + v01.clamp(0.0, 1.0) * (c.max - c.min));
    }

    /// Project onto the solver-internal [`DiffusionParams`] (1:1 over the 19 controls) —
    /// the single conversion point the live path uses to drive the solver.
    #[must_use]
    pub fn to_diffusion(&self) -> DiffusionParams {
        DiffusionParams {
            diffusivity: self.diffusivity,
            evaporation: self.evaporation,
            downhill: self.downhill,
            flow_outward: self.flow_outward,
            w_lo: self.w_lo,
            w_hi: self.w_hi,
            perm_valley: self.perm_valley,
            perm_crest: self.perm_crest,
            deposition: self.deposition,
            deposition_dry: self.deposition_dry,
            granulation: self.granulation,
            velocity: self.velocity,
            viscosity: self.viscosity,
            drag: self.drag,
            pressure: self.pressure,
            capillary: self.capillary,
            // Pigment mobility is a physical constant (paper filtering), not a brush control.
            capillary_mobility: crate::diffusion::CAPILLARY_PIGMENT_MOBILITY,
            sharpness: self.sharpness,
            lift: self.lift,
            capillary_branching: self.capillary_branching,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preset_defaults_lie_inside_their_ranges() {
        let p = WatercolorParams::default();
        for i in 0..WatercolorParams::COUNT {
            let n = p.normalized(i);
            assert!(
                (0.0..=1.0).contains(&n),
                "control {} ({}) preset out of range: {}",
                i,
                WatercolorParams::CONTROLS[i].label,
                p.get(i)
            );
        }
    }

    #[test]
    fn normalized_set_roundtrips() {
        let mut p = WatercolorParams::default();
        for i in 0..WatercolorParams::COUNT {
            p.set_normalized(i, 0.5);
            let c = &WatercolorParams::CONTROLS[i];
            let expect = c.min + 0.5 * (c.max - c.min);
            assert!((p.get(i) - expect).abs() < 1e-6, "control {i} roundtrip");
            assert!(
                (p.normalized(i) - 0.5).abs() < 1e-6,
                "control {i} normalize"
            );
        }
    }

    #[test]
    fn to_diffusion_preserves_all_controls() {
        let p = WatercolorParams {
            diffusivity: 0.1,
            granulation: 2.0,
            pressure: 0.7,
            ..Default::default()
        };
        let d = p.to_diffusion();
        assert_eq!(d.diffusivity, 0.1);
        assert_eq!(d.granulation, 2.0);
        assert_eq!(d.pressure, 0.7);
    }
}
