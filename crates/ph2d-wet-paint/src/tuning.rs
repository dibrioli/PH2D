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

mod defs;

pub use defs::{KNOB_DEFS, KnobDef};

/// The boot value of every knob, in [`Knob`] discriminant order — `const` so
/// a host's authored-state `DEFAULT` (a `const` item) can carry the exact
/// engine boot (f64 equality gates depend on it).
#[must_use]
pub const fn knob_defaults() -> [f64; KNOB_COUNT] {
    let mut values = [0.0; KNOB_COUNT];
    let mut i = 0;
    while i < KNOB_COUNT {
        values[i] = KNOB_DEFS[i].default;
        i += 1;
    }
    values
}

/// Live knob values. Writes report what they invalidated; the facade reacts
/// (lazy brush rebuild, paper re-bake on release, repaint).
#[derive(Clone)]
pub struct Tuning {
    values: [f64; KNOB_COUNT],
}

impl Default for Tuning {
    fn default() -> Self {
        Tuning {
            values: knob_defaults(),
        }
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

    /// Reset every knob of a group. Returns the defs that actually changed —
    /// the caller MUST act on their `rebuild` kinds exactly as it would for
    /// single `set` calls (the JS fires onChange per knob; swallowing these
    /// left stale brush textures / unbaked paper — port-verify finding).
    #[must_use = "act on the rebuild kinds of the changed knobs"]
    pub fn reset_group(&mut self, group: KnobGroup) -> Vec<&'static KnobDef> {
        let mut changed = Vec::new();
        for d in KNOB_DEFS.iter() {
            if d.group == group
                && let Some(def) = self.set(d.knob, d.default)
            {
                changed.push(def);
            }
        }
        changed
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
