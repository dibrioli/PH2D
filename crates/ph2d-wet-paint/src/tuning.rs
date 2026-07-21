//! Live tuning registry (port of `tuning.js`, SPEC §16): one module owning
//! every calibration knob the engine reads at runtime.
//!
//! The engine reads knobs through the typed [`Knob`] enum (an array index —
//! cheaper than the JS Map, same semantics); the panel reads the [`KNOB_DEFS`]
//! table for labels/ranges/defaults and writes through [`Tuning::set_by_key`]
//! (NaN falls back to the default, exactly like the JS free numeric input).
//!
//! The `Hidden` group holds the gated extensions of SPEC §17: implemented,
//! default-neutral, no visible sliders.

/// Every calibration knob. Discriminants are indices into [`KNOB_DEFS`] and
/// the value array — the order below IS the table order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(usize)]
pub enum Knob {
    PigmentPerDab,
    PaperGate,
    Felt,
    BristleCount,
    Drag,
    Pickup,
    Intensity,
    BristleStrength,
    BristleSize,
    Spacing,
    TipClean,
    BlendForce,
    GateSaturation,
    WaterPerDab,
    WaterCap,
    Evaporation,
    Rewet,
    Retention,
    EdgeDarkening,
    BaseEvaporation,
    Leveling,
    Capillary,
    Brake,
    Gravity,
    LevelClamp,
    Viscosity,
    MaxVelocity,
    Projection,
    BrakeReach,
    CapillaryGate,
    Eraser,
    Dryer,
    Blow,
    Smear,
    PaperContrast,
    PaperFibres,
    PaperGrooves,
    VisualGrain,
    Emboss,
    PaperVisibility,
    ExtDiffusion,
    ExtBackrun,
    ExtFingering,
    ExtRivulets,
    ExtGranulation,
    ExtStaining,
    ExtDryBrush,
    ExtWetSoften,
    ExtRakeLight,
    ExtValleyGran,
    ExtWetSheen,
    ExtGrainDither,
    ExtEdgeTint,
}

pub const KNOB_COUNT: usize = 53;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KnobGroup {
    Paint,
    Water,
    Physics,
    Tools,
    Paper,
    /// SPEC §17 gated extensions: no visible sliders, neutral defaults.
    Hidden,
}

/// What a knob write invalidates (the facade acts on it).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rebuild {
    /// Rebuild the bristle stamp texture.
    Brush,
    /// Re-bake the paper tile (on slider release).
    Paper,
    /// Appearance only: repaint.
    Render,
}

pub struct KnobDef {
    pub knob: Knob,
    pub group: KnobGroup,
    pub key: &'static str,
    pub min: f64,
    pub max: f64,
    pub step: f64,
    pub default: f64,
    pub rebuild: Option<Rebuild>,
}

const fn k(
    knob: Knob,
    group: KnobGroup,
    key: &'static str,
    min: f64,
    max: f64,
    step: f64,
    default: f64,
    rebuild: Option<Rebuild>,
) -> KnobDef {
    KnobDef { knob, group, key, min, max, step, default, rebuild }
}

/// The full registry, in [`Knob`] discriminant order (defaults from SPEC §16).
pub const KNOB_DEFS: [KnobDef; KNOB_COUNT] = [
    k(Knob::PigmentPerDab, KnobGroup::Paint, "pigmentPerDab", 0.0, 2000.0, 10.0, 600.0, None),
    k(Knob::PaperGate, KnobGroup::Paint, "paperGate", 0.0, 1.5, 0.01, 0.6, None),
    k(Knob::Felt, KnobGroup::Paint, "felt", 0.0, 0.2, 0.001, 0.01, Some(Rebuild::Brush)),
    k(Knob::BristleCount, KnobGroup::Paint, "bristleCount", 0.0, 4000.0, 10.0, 950.0, Some(Rebuild::Brush)),
    k(Knob::Drag, KnobGroup::Paint, "drag", 0.0, 1.0, 0.01, 0.16, None),
    k(Knob::Pickup, KnobGroup::Paint, "pickup", 0.0, 0.2, 0.001, 0.005, None),
    k(Knob::Intensity, KnobGroup::Paint, "intensity", 0.0, 4.0, 0.05, 1.0, None),
    k(Knob::BristleStrength, KnobGroup::Paint, "bristleStrength", 0.0, 4.0, 0.05, 1.0, Some(Rebuild::Brush)),
    k(Knob::BristleSize, KnobGroup::Paint, "bristleSize", 0.2, 4.0, 0.05, 1.0, Some(Rebuild::Brush)),
    k(Knob::Spacing, KnobGroup::Paint, "spacing", 0.5, 8.0, 0.5, 2.0, None),
    k(Knob::TipClean, KnobGroup::Paint, "tipClean", 0.0, 0.05, 0.0005, 0.001, None),
    k(Knob::BlendForce, KnobGroup::Paint, "blendForce", 0.0, 1.0, 0.01, 0.8, None),
    k(Knob::GateSaturation, KnobGroup::Paint, "gateSaturation", 100.0, 6000.0, 50.0, 2000.0, None),
    k(Knob::WaterPerDab, KnobGroup::Water, "waterPerDab", 0.0, 4.0, 0.05, 1.0, None),
    k(Knob::WaterCap, KnobGroup::Water, "waterCap", 1.0, 30.0, 0.5, 8.0, None),
    k(Knob::Evaporation, KnobGroup::Water, "evaporation", 0.0, 8.0, 0.05, 1.0, None),
    k(Knob::Rewet, KnobGroup::Water, "rewet", 0.0, 8.0, 0.05, 1.0, None),
    k(Knob::Retention, KnobGroup::Water, "retention", 0.9, 1.0, 0.0005, 0.995, None),
    k(Knob::EdgeDarkening, KnobGroup::Water, "edgeDarkening", 0.0, 200.0, 1.0, 50.0, None),
    k(Knob::BaseEvaporation, KnobGroup::Water, "baseEvaporation", 0.0, 20.0, 0.1, 2.0, None),
    k(Knob::Leveling, KnobGroup::Physics, "leveling", 0.0, 2.0, 0.01, 0.5, None),
    k(Knob::Capillary, KnobGroup::Physics, "capillary", 0.0, 1.0, 0.01, 0.2, None),
    k(Knob::Brake, KnobGroup::Physics, "brake", 0.0, 4.0, 0.05, 1.5, None),
    k(Knob::Gravity, KnobGroup::Physics, "gravity", 0.0, 0.05, 0.0005, 0.005, None),
    k(Knob::LevelClamp, KnobGroup::Physics, "levelClamp", 0.0, 1.0, 0.01, 0.2, None),
    k(Knob::Viscosity, KnobGroup::Physics, "viscosity", 0.0, 2.0, 0.01, 0.2, None),
    k(Knob::MaxVelocity, KnobGroup::Physics, "maxVelocity", 0.2, 3.0, 0.05, 1.0, None),
    k(Knob::Projection, KnobGroup::Physics, "projection", 0.0, 1.5, 0.01, 0.7, None),
    k(Knob::BrakeReach, KnobGroup::Physics, "brakeReach", 1.0, 12.0, 0.5, 4.0, None),
    k(Knob::CapillaryGate, KnobGroup::Physics, "capillaryGate", 100.0, 6000.0, 50.0, 2000.0, None),
    k(Knob::Eraser, KnobGroup::Tools, "eraser", 0.0, 1.0, 0.01, 0.4, None),
    k(Knob::Dryer, KnobGroup::Tools, "dryer", 0.0, 1.0, 0.01, 0.32, None),
    k(Knob::Blow, KnobGroup::Tools, "blow", 0.0, 1.0, 0.01, 0.2, None),
    k(Knob::Smear, KnobGroup::Tools, "smear", 0.0, 1.0, 0.01, 0.8, None),
    k(Knob::PaperContrast, KnobGroup::Paper, "paperContrast", 0.2, 2.5, 0.05, 1.0, Some(Rebuild::Paper)),
    k(Knob::PaperFibres, KnobGroup::Paper, "paperFibres", 0.0, 3.0, 0.05, 1.0, Some(Rebuild::Paper)),
    k(Knob::PaperGrooves, KnobGroup::Paper, "paperGrooves", 0.0, 3.0, 0.05, 1.0, Some(Rebuild::Paper)),
    k(Knob::VisualGrain, KnobGroup::Paper, "visualGrain", 0.0, 3.0, 0.05, 1.0, Some(Rebuild::Render)),
    k(Knob::Emboss, KnobGroup::Paper, "emboss", 0.0, 4.0, 0.05, 1.0, Some(Rebuild::Render)),
    // Render-only master (paper group header): fades the tooth in the
    // APPEARANCE only — the physics never sees it.
    k(Knob::PaperVisibility, KnobGroup::Paper, "paperVisibility", 0.0, 1.5, 0.01, 1.0, Some(Rebuild::Render)),
    // ---- gated extensions (SPEC §17): neutral defaults, no sliders ----
    k(Knob::ExtDiffusion, KnobGroup::Hidden, "extDiffusion", 0.0, 0.25, 0.005, 0.0, None),
    k(Knob::ExtBackrun, KnobGroup::Hidden, "extBackrun", 0.0, 2.0, 0.05, 0.0, None),
    k(Knob::ExtFingering, KnobGroup::Hidden, "extFingering", 0.0, 1.0, 0.01, 0.0, None),
    k(Knob::ExtRivulets, KnobGroup::Hidden, "extRivulets", 4.0, 60.0, 1.0, 15.0, None),
    k(Knob::ExtGranulation, KnobGroup::Hidden, "extGranulation", 0.0, 1.0, 0.01, 0.0, None),
    k(Knob::ExtStaining, KnobGroup::Hidden, "extStaining", 0.0, 1.0, 0.01, 0.5, None),
    k(Knob::ExtDryBrush, KnobGroup::Hidden, "extDryBrush", 0.0, 1.0, 0.01, 0.0, None),
    k(Knob::ExtWetSoften, KnobGroup::Hidden, "extWetSoften", 0.0, 1.0, 0.01, 0.0, None),
    k(Knob::ExtRakeLight, KnobGroup::Hidden, "extRakeLight", 0.0, 3.0, 0.05, 0.0, Some(Rebuild::Render)),
    k(Knob::ExtValleyGran, KnobGroup::Hidden, "extValleyGran", 0.0, 1.0, 0.01, 0.0, Some(Rebuild::Render)),
    k(Knob::ExtWetSheen, KnobGroup::Hidden, "extWetSheen", 0.0, 2.0, 0.05, 0.0, Some(Rebuild::Render)),
    k(Knob::ExtGrainDither, KnobGroup::Hidden, "extGrainDither", 0.0, 8.0, 0.1, 0.0, Some(Rebuild::Render)),
    k(Knob::ExtEdgeTint, KnobGroup::Hidden, "extEdgeTint", 0.0, 1.0, 0.01, 0.5, Some(Rebuild::Render)),
];

/// Live knob values. Writes report what they invalidated; the facade reacts
/// (lazy brush rebuild, paper re-bake on release, repaint).
#[derive(Clone)]
pub struct Tuning {
    values: [f64; KNOB_COUNT],
}

impl Default for Tuning {
    fn default() -> Self {
        let mut values = [0.0; KNOB_COUNT];
        for (i, d) in KNOB_DEFS.iter().enumerate() {
            values[i] = d.default;
        }
        Tuning { values }
    }
}

impl Tuning {
    #[inline]
    pub fn get(&self, knob: Knob) -> f64 {
        self.values[knob as usize]
    }

    /// Copy of the whole value array (the per-step params snapshot).
    #[inline]
    pub fn snapshot(&self) -> [f64; KNOB_COUNT] {
        self.values
    }

    pub fn def_of_key(key: &str) -> Option<&'static KnobDef> {
        KNOB_DEFS.iter().find(|d| d.key == key)
    }

    /// Set a knob. NaN (a garbled numeric input) falls back to the default.
    /// Returns the def when the stored value actually changed.
    pub fn set(&mut self, knob: Knob, v: f64) -> Option<&'static KnobDef> {
        let d = &KNOB_DEFS[knob as usize];
        let nv = if v.is_nan() { d.default } else { v };
        if self.values[knob as usize] == nv {
            return None;
        }
        self.values[knob as usize] = nv;
        Some(d)
    }

    /// Panel-side write by registry key. Unknown keys are ignored.
    pub fn set_by_key(&mut self, key: &str, v: f64) -> Option<&'static KnobDef> {
        let d = Self::def_of_key(key)?;
        self.set(d.knob, v)
    }

    pub fn reset(&mut self, knob: Knob) -> Option<&'static KnobDef> {
        self.set(knob, KNOB_DEFS[knob as usize].default)
    }

    pub fn reset_group(&mut self, group: KnobGroup) {
        for d in KNOB_DEFS.iter() {
            if d.group == group {
                self.set(d.knob, d.default);
            }
        }
    }

    pub fn is_default(&self, knob: Knob) -> bool {
        self.values[knob as usize] == KNOB_DEFS[knob as usize].default
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_table_order_is_the_enum_order() {
        for (i, d) in KNOB_DEFS.iter().enumerate() {
            assert_eq!(d.knob as usize, i, "KNOB_DEFS[{i}] holds {:?}", d.knob);
        }
    }

    #[test]
    fn keys_are_unique() {
        for (i, a) in KNOB_DEFS.iter().enumerate() {
            for b in KNOB_DEFS.iter().skip(i + 1) {
                assert_ne!(a.key, b.key);
            }
        }
    }
}
