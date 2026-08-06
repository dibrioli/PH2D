//! **What a param's number IS** — the unit vocabulary (doc 88, Wave A).
//!
//! Side-metadata on [`super::NodeRegistry`], additive and default-empty, exactly
//! like [`ParamHardMax`](super::ParamHardMax) / `ParamGate` / `Coupling`: a param
//! with no entry is [`ParamUnit::None`], which is what every param already means
//! today, so nothing moves by default. Lives in its own sibling module rather than
//! inside `ui.rs` because it is a NEW concept and the registry is extended
//! concurrently by other lines (ADR-0107 — foundational is designed for isolation).
//!
//! # The one law
//!
//! A unit declares **what the number is**, never **how it is shown**. `Px` and
//! `Meters` are two faces of one quantity and are therefore NOT declarable by a
//! node: a node that declared `Px` would be opting out of the unit the artist
//! chose in `ProjectSettings::display_unit`, which is a knob the artist cannot
//! reach. The display face is resolved at the panel boundary, once.
//!
//! # Why the store stays METERS
//!
//! A Motion length param is already in world meters — `RenderInstance::world_pos`
//! is the same world space every sprite lives in and `RenderInstance::size` is
//! documented as *local meters*. It stays that way, and only display/parse
//! converts, for a reason that is not taste: **the cook must not depend on a
//! project setting**. If `gap_x` were stored in pixels, the lowering would have to
//! divide by `pixels_per_meter` to produce a world position, and the cook's
//! fingerprint would become a function of a number the artist edits in a Settings
//! menu — taking CPU×GPU parity and cross-machine determinism with it. It is the
//! same discipline `ph2d-timeline` already runs (it stores radians while the app
//! authors degrees) and the same one [`ParamWidget::Angle`](super::ParamWidget)
//! states for itself.

use crate::ui::ParamWidget;

/// The unit of a declared param's `f32`.
///
/// Ordered from "no unit" outward; [`Self::None`] is the neutral default so an
/// un-annotated param behaves exactly as it does today.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum ParamUnit {
    /// Genuinely dimensionless — a weight, a seed-ish index, a raw coefficient.
    /// Also the honest answer for the `value.*` family, whose magnitude has **no
    /// unit of its own**: `value.lfo`'s amplitude means whatever the column the
    /// artist wires it into means (metres on `P`, degrees on `rot`, nothing on
    /// `tint`). A unit is a property of the FLOW there, not of the node, and a
    /// visible gap is worth more than a wrong number.
    #[default]
    None,
    /// A world DISTANCE, stored in **metres** (see the module header). The only
    /// unit the display boundary converts.
    Length,
    /// An angle in **degrees** — the app's one authored-angle unit.
    Angle,
    /// A duration in **seconds**.
    Seconds,
    /// A whole count of things (particles, rows, copies).
    Count,
    /// A 0..1 fraction or a plain multiplier.
    Ratio,
    /// **The unit is a function of another param's value** — the node declares a
    /// `channel` param and the magnitude means metres on Position, degrees on
    /// Rotation, a scale factor on Size.
    ///
    /// This variant exists because the alternative is measurably worse: the shell
    /// already carries a hand-written `match` over `(type_name, param)` to widen
    /// such a param's range on the Rotation channel, and its own doc-comment
    /// records the bug that forced it. Declaring `FromChannel` is what keeps a
    /// length conversion from being applied to degrees — the failure that turns a
    /// `±90` into a `±9000`.
    FromChannel,
}

impl ParamUnit {
    /// Whether the display boundary converts this unit through
    /// `pixels_per_meter`. Only [`Self::Length`] does — everything else is stored
    /// in the unit it is shown in.
    #[must_use]
    pub fn converts(self) -> bool {
        matches!(self, ParamUnit::Length)
    }

    /// The unit suffix shown next to the number, for the units that have a fixed
    /// one. [`Self::Length`] returns `None` because its face is chosen by the
    /// project's display unit, and [`Self::FromChannel`] because its face is
    /// chosen by a sibling param — both are resolved by the caller that knows.
    #[must_use]
    pub fn fixed_suffix(self) -> Option<&'static str> {
        match self {
            ParamUnit::Angle => Some("deg"),
            ParamUnit::Seconds => Some("s"),
            ParamUnit::Length | ParamUnit::FromChannel | ParamUnit::None => None,
            ParamUnit::Count | ParamUnit::Ratio => None,
        }
    }
}

/// A declared unit for one param of some node type.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ParamUnitDecl {
    /// The `ParamSpec::name` this annotates.
    pub param: &'static str,
    pub unit: ParamUnit,
}

/// A param whose typed entry reaches BELOW its slider — the floor twin of
/// [`ParamHardMax`](super::ParamHardMax), and its exact mould (a separate table
/// rather than a field on `ParamUiHint`, because that hint is a struct literal at
/// hundreds of sites and a new field is hundreds of edits to express one
/// exception).
///
/// The ceiling shipped alone; the floor did not exist at all, so until now the box
/// could never go under the slider's `min`. That asymmetry is what stopped an
/// artist typing `0.001` into a param whose useful drag starts at `0.01`.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ParamHardMin {
    /// The `ParamSpec::name` this widens downward.
    pub param: &'static str,
    /// The typed-entry floor. Must be ≤ the hint's `min` to mean anything.
    pub min: f32,
}

/// **The one door to "what unit is this param?"** — the widget answers first, the
/// declared table second.
///
/// The order is not an optimisation, it is the invariant: a `ParamWidget` that
/// already fixes the unit (an `Angle` is degrees, a `Seed` is a count, an `Enum`
/// is an index) cannot be contradicted by a table entry, so those params never
/// need one and can never be mis-declared. Only a plain `Slider` carries an open
/// question, and only it consults `declared`.
///
/// ⚠️ A `Length` on a whole-number widget is REFUSED (falls back to `Count`):
/// scaling does not commute with rounding, so a `step` of 1 would become a step of
/// 100 under a pixel display and the chip would walk a hundred at a time.
#[must_use]
pub fn unit_of(widget: ParamWidget, declared: Option<ParamUnit>) -> ParamUnit {
    match widget {
        ParamWidget::Angle => ParamUnit::Angle,
        ParamWidget::IntSlider | ParamWidget::Seed => ParamUnit::Count,
        ParamWidget::Toggle | ParamWidget::Enum { .. } => ParamUnit::None,
        // Non-numeric widgets carry no number to have a unit.
        ParamWidget::Color { .. }
        | ParamWidget::Channels { .. }
        | ParamWidget::Source
        | ParamWidget::Text
        | ParamWidget::Curve
        | ParamWidget::Gradient
        | ParamWidget::Palette => ParamUnit::None,
        ParamWidget::Slider => declared.unwrap_or_default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The neutral default is what every un-annotated param already means.
    #[test]
    fn an_undeclared_slider_is_unitless() {
        assert_eq!(unit_of(ParamWidget::Slider, None), ParamUnit::None);
        assert_eq!(ParamUnit::default(), ParamUnit::None);
        assert!(!ParamUnit::None.converts());
    }

    /// The widget answers FIRST — a table entry cannot contradict a widget that
    /// already fixes the unit. Without this, one mis-declared entry could tell the
    /// panel to scale an angle by `pixels_per_meter`.
    #[test]
    fn the_widget_wins_over_a_contradicting_declaration() {
        for w in [
            ParamWidget::Angle,
            ParamWidget::IntSlider,
            ParamWidget::Seed,
            ParamWidget::Toggle,
            ParamWidget::Enum { labels: &["a"] },
        ] {
            let got = unit_of(w, Some(ParamUnit::Length));
            assert_ne!(
                got,
                ParamUnit::Length,
                "{w:?} fixes its own unit; a declared Length must not reach it"
            );
            assert!(!got.converts(), "{w:?} must never be converted");
        }
    }

    /// `Angle` is the precedent this whole vocabulary follows: the widget already
    /// said "degrees", so the table never has to.
    #[test]
    fn an_angle_widget_is_degrees_without_being_declared() {
        assert_eq!(unit_of(ParamWidget::Angle, None), ParamUnit::Angle);
        assert_eq!(ParamUnit::Angle.fixed_suffix(), Some("deg"));
    }

    /// Only a Length crosses `pixels_per_meter`. Everything else is stored in the
    /// unit it is shown in, so converting it would be inventing a scale.
    #[test]
    fn length_is_the_only_unit_that_converts() {
        for u in [
            ParamUnit::None,
            ParamUnit::Angle,
            ParamUnit::Seconds,
            ParamUnit::Count,
            ParamUnit::Ratio,
            ParamUnit::FromChannel,
        ] {
            assert!(
                !u.converts(),
                "{u:?} must not be scaled by pixels_per_meter"
            );
        }
        assert!(ParamUnit::Length.converts());
    }

    /// A Length has NO fixed suffix on purpose — its face (`px` / `m`) is the
    /// project's choice, not the node's. A node that could pin it would be opting
    /// out of the artist's setting.
    #[test]
    fn a_length_has_no_node_chosen_face() {
        assert_eq!(ParamUnit::Length.fixed_suffix(), None);
        assert_eq!(ParamUnit::FromChannel.fixed_suffix(), None);
    }

    /// A slider DOES consult the table — the half that makes the feature exist.
    #[test]
    fn a_declared_slider_takes_its_unit() {
        assert_eq!(
            unit_of(ParamWidget::Slider, Some(ParamUnit::Length)),
            ParamUnit::Length
        );
        assert_eq!(
            unit_of(ParamWidget::Slider, Some(ParamUnit::Seconds)),
            ParamUnit::Seconds
        );
    }
}
