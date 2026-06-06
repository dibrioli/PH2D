//! Variable-font axes (ADR-0066 §2.2): the OT axis tag, the [`VariableFontAxis`]
//! trait, and a concrete [`FontAxis`].
//!
//! An axis is a named, bounded `f32` parameter (`weight`, `width`, `slant`,
//! `optical-size`, custom). Exposing axes as graph parameters is what makes
//! typography *animatable* (ADR-0066 §2.4); this module is the data the rest of
//! the crate (and the animation graph) drives.

use core::fmt;

/// An OpenType 4-byte axis tag, e.g. `wght`. Tags are ASCII; the registered
/// axes have associated constants.
#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct AxisTag([u8; 4]);

impl AxisTag {
    /// Weight (`wght`, typically 1..1000, default 400).
    pub const WEIGHT: AxisTag = AxisTag(*b"wght");
    /// Width (`wdth`, percent of normal, default 100).
    pub const WIDTH: AxisTag = AxisTag(*b"wdth");
    /// Slant (`slnt`, degrees, default 0).
    pub const SLANT: AxisTag = AxisTag(*b"slnt");
    /// Optical size (`opsz`, points, default ≈ text size).
    pub const OPTICAL_SIZE: AxisTag = AxisTag(*b"opsz");
    /// Italic (`ital`, 0..1).
    pub const ITALIC: AxisTag = AxisTag(*b"ital");
    /// Grade (`GRAD`, weight without changing advance).
    pub const GRADE: AxisTag = AxisTag(*b"GRAD");

    /// Build a tag from 4 bytes (the raw OT tag).
    pub const fn new(tag: [u8; 4]) -> Self {
        Self(tag)
    }

    /// The raw 4 bytes.
    pub const fn to_bytes(self) -> [u8; 4] {
        self.0
    }

    /// The tag as a `&str` (OT tags are ASCII; non-UTF-8 falls back to `????`).
    pub fn as_str(&self) -> &str {
        core::str::from_utf8(&self.0).unwrap_or("????")
    }
}

impl fmt::Debug for AxisTag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "AxisTag({})", self.as_str())
    }
}

impl fmt::Display for AxisTag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Setting an axis outside its `[min, max]` range.
#[derive(Clone, Debug, PartialEq)]
pub struct AxisOutOfRangeError {
    pub tag: AxisTag,
    pub value: f32,
    pub min: f32,
    pub max: f32,
}

impl fmt::Display for AxisOutOfRangeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "axis {} value {} out of range [{}, {}]",
            self.tag, self.value, self.min, self.max
        )
    }
}

impl std::error::Error for AxisOutOfRangeError {}

/// A variable-font design axis (ADR-0066 §2.2).
pub trait VariableFontAxis {
    /// Human-readable name (`"Weight"`, `"Optical Size"`, …).
    fn name(&self) -> &str;
    /// The OT 4-byte tag.
    fn tag(&self) -> AxisTag;
    fn min(&self) -> f32;
    fn max(&self) -> f32;
    fn default(&self) -> f32;
    /// The currently-set value.
    fn current(&self) -> f32;
    /// Set the value; errors (without mutating) if outside `[min, max]`.
    fn set(&mut self, value: f32) -> Result<(), AxisOutOfRangeError>;
}

/// A concrete variable-font axis (the value the engine animates).
#[derive(Clone, Debug, PartialEq)]
pub struct FontAxis {
    name: String,
    tag: AxisTag,
    min: f32,
    max: f32,
    default: f32,
    current: f32,
}

impl FontAxis {
    /// New axis starting at its default. `min`/`max` are ordered defensively;
    /// `default` clamps into range.
    pub fn new(name: impl Into<String>, tag: AxisTag, min: f32, default: f32, max: f32) -> Self {
        let (lo, hi) = if min <= max { (min, max) } else { (max, min) };
        let default = default.clamp(lo, hi);
        Self {
            name: name.into(),
            tag,
            min: lo,
            max: hi,
            default,
            current: default,
        }
    }

    /// Set the value clamped into `[min, max]`, returning what was applied. The
    /// animation path uses this (an axis driven past its range pins, never
    /// fails); [`VariableFontAxis::set`] is the strict, erroring setter.
    pub fn set_clamped(&mut self, value: f32) -> f32 {
        self.current = value.clamp(self.min, self.max);
        self.current
    }

    /// The value normalized to `[-1, 1]` around the default (the f2dot14 space OT
    /// variations interpolate in: default→0, min→-1, max→+1).
    pub fn normalized(&self) -> f32 {
        let v = self.current;
        if v < self.default {
            let span = (self.default - self.min).max(f32::EPSILON);
            ((v - self.default) / span).max(-1.0)
        } else if v > self.default {
            let span = (self.max - self.default).max(f32::EPSILON);
            ((v - self.default) / span).min(1.0)
        } else {
            0.0
        }
    }
}

impl VariableFontAxis for FontAxis {
    fn name(&self) -> &str {
        &self.name
    }
    fn tag(&self) -> AxisTag {
        self.tag
    }
    fn min(&self) -> f32 {
        self.min
    }
    fn max(&self) -> f32 {
        self.max
    }
    fn default(&self) -> f32 {
        self.default
    }
    fn current(&self) -> f32 {
        self.current
    }
    fn set(&mut self, value: f32) -> Result<(), AxisOutOfRangeError> {
        if value < self.min || value > self.max {
            return Err(AxisOutOfRangeError {
                tag: self.tag,
                value,
                min: self.min,
                max: self.max,
            });
        }
        self.current = value;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tag_round_trips_and_prints() {
        assert_eq!(AxisTag::WEIGHT.as_str(), "wght");
        assert_eq!(AxisTag::new(*b"opsz"), AxisTag::OPTICAL_SIZE);
        assert_eq!(format!("{}", AxisTag::SLANT), "slnt");
    }

    #[test]
    fn set_enforces_range() {
        let mut a = FontAxis::new("Weight", AxisTag::WEIGHT, 100.0, 400.0, 900.0);
        assert!(a.set(700.0).is_ok());
        assert_eq!(a.current(), 700.0);
        let err = a.set(1200.0).unwrap_err();
        assert_eq!(err.max, 900.0);
        assert_eq!(a.current(), 700.0, "failed set must not mutate");
    }

    #[test]
    fn clamp_setter_pins() {
        let mut a = FontAxis::new("Weight", AxisTag::WEIGHT, 100.0, 400.0, 900.0);
        assert_eq!(a.set_clamped(2000.0), 900.0);
        assert_eq!(a.set_clamped(-50.0), 100.0);
    }

    #[test]
    fn normalized_maps_default_to_zero() {
        let mut a = FontAxis::new("Weight", AxisTag::WEIGHT, 100.0, 400.0, 900.0);
        assert_eq!(a.normalized(), 0.0);
        a.set_clamped(900.0);
        assert!((a.normalized() - 1.0).abs() < 1e-6);
        a.set_clamped(100.0);
        assert!((a.normalized() + 1.0).abs() < 1e-6);
    }
}
