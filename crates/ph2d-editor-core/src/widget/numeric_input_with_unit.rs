//! [`NumericInputWithUnit`] — a [`NumberInput`](super::number_input)
//! with a trailing unit chip (`px` / `m` / `deg` / `rad` / `%`).
//!
//! Sprite Inspector v2 W6 (spec §15.7, T6.2). Several Inspector fields
//! carry a physical unit (Transform position in `px`/`m`, rotation in
//! `deg`/`rad`, opacity in `%`). This widget renders the unit suffix as
//! a non-editable chip on the right edge of the field and parses a
//! typed suffix back into a value + unit so users can type `90deg`.

use super::number_input::{NumberInput, paint_number_input_with_buffer};
use super::text_input::TextInputState;
use crate::paint::{fill_rounded_rect, paint_text, resolve};
use crate::zones::Rect;
use ph2d_a11y::Node;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Radius, Spacing, Theme, TypeToken};
use ph2d_vector::VectorScene;

/// Physical unit displayed (and parsed) by [`NumericInputWithUnit`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Unit {
    /// Pixels.
    Px,
    /// Meters.
    Meters,
    /// Degrees.
    Degrees,
    /// Radians.
    Radians,
    /// Percent.
    Percent,
}

impl Unit {
    /// Canonical suffix string, also what the parser matches on.
    pub const fn suffix(self) -> &'static str {
        match self {
            Unit::Px => "px",
            Unit::Meters => "m",
            Unit::Degrees => "deg",
            Unit::Radians => "rad",
            Unit::Percent => "%",
        }
    }

    /// Longest-match parse of a trailing unit suffix. Returns the unit
    /// whose suffix the (lowercased, trimmed) string ends with, longest
    /// first so `rad` wins over a hypothetical shorter match.
    pub fn parse_suffix(s: &str) -> Option<Unit> {
        let s = s.trim().to_ascii_lowercase();
        // Order longest-first to avoid a short suffix shadowing a longer
        // one that shares its tail.
        const ALL: [Unit; 5] = [
            Unit::Degrees,
            Unit::Radians,
            Unit::Meters,
            Unit::Px,
            Unit::Percent,
        ];
        ALL.into_iter().find(|u| s.ends_with(u.suffix()))
    }
}

/// Parse a typed field like `"90deg"` / `"12.5 px"` / `"50%"` into its
/// numeric value and (optional) unit. Returns `None` only when the
/// numeric portion fails to parse. A bare number (`"42"`) yields
/// `(42.0, None)`.
pub fn parse(input: &str) -> Option<(f64, Option<Unit>)> {
    let trimmed = input.trim();
    let unit = Unit::parse_suffix(trimmed);
    let num_part = match unit {
        Some(u) => &trimmed[..trimmed.len() - u.suffix().len()],
        None => trimmed,
    };
    num_part.trim().parse::<f64>().ok().map(|v| (v, unit))
}

#[derive(Clone, Debug)]
pub struct NumericInputWithUnit {
    pub input: NumberInput,
    pub unit: Unit,
}

impl NumericInputWithUnit {
    pub fn new(input: NumberInput, unit: Unit) -> Self {
        Self { input, unit }
    }

    pub fn state(mut self, state: TextInputState) -> Self {
        self.input.state = state;
        self
    }

    /// Width reserved on the right edge for the unit chip.
    fn chip_w(&self) -> f32 {
        // Two suffixes are wide enough to need extra room (`deg`/`rad`);
        // the rest fit in the base chip. Keep a single canonical width
        // so adjacent fields align.
        Spacing::Xl3.px() // LITERAL-PX-OK: unit chip column width (canonical, aligns adjacent fields)
    }

    /// Rect occupied by the editable number field (host minus the chip).
    pub fn input_rect(&self, host: Rect) -> Rect {
        let chip = self.chip_w() + Spacing::Xs.px();
        Rect::new(host.x, host.y, (host.w - chip).max(0.0), host.h)
    }

    /// Rect occupied by the unit chip.
    pub fn unit_rect(&self, host: Rect) -> Rect {
        let chip = self.chip_w();
        Rect::new(host.x + host.w - chip, host.y, chip, host.h)
    }

    /// Build the a11y node. Signature mirrors the widget template
    /// (`x, y, w, h`); the unit is folded into the inner field's label
    /// so a screen reader announces "<label> (deg)".
    pub fn build_a11y(&self, x: f64, y: f64, w: f64, h: f64) -> Node {
        let mut labeled = self.input.clone();
        labeled.label = if self.input.label.is_empty() {
            self.unit.suffix().to_string()
        } else {
            format!("{} ({})", self.input.label, self.unit.suffix())
        };
        let host = Rect::new(x as f32, y as f32, w as f32, h as f32);
        let r = self.input_rect(host);
        labeled.build_a11y(r.x as f64, r.y as f64, r.w as f64, r.h as f64)
    }
}

#[allow(clippy::too_many_arguments)]
pub fn paint_numeric_input_with_unit(
    widget: &NumericInputWithUnit,
    buffer: Option<&str>,
    caret: usize,
    selection_anchor: Option<usize>,
    host: Rect,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) {
    let field = widget.input_rect(host);
    paint_number_input_with_buffer(
        &widget.input,
        buffer,
        caret,
        selection_anchor,
        field,
        scene,
        text_system,
        theme,
    );
    let chip = widget.unit_rect(host);
    let disabled = widget.input.state == TextInputState::Disabled;
    fill_rounded_rect(
        scene,
        chip,
        Radius::Sm.px(),
        resolve(ColorToken::Bg2, theme),
    );
    let font = TypeToken::Sm.px();
    let color = if disabled {
        ColorToken::TextDisabled
    } else {
        ColorToken::Text2
    };
    paint_text(
        text_system,
        scene,
        widget.unit.suffix(),
        chip.x + Spacing::Xs.px(),
        chip.y + (chip.h - font) * 0.5,
        font,
        chip.w - Spacing::Xs.px() * 2.0,
        resolve(color, theme),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_a11y::NodeId;

    #[test]
    fn suffix_round_trips() {
        for u in [
            Unit::Px,
            Unit::Meters,
            Unit::Degrees,
            Unit::Radians,
            Unit::Percent,
        ] {
            assert_eq!(Unit::parse_suffix(u.suffix()), Some(u));
        }
    }

    #[test]
    fn parse_value_and_unit() {
        assert_eq!(parse("90deg"), Some((90.0, Some(Unit::Degrees))));
        assert_eq!(parse("12.5 px"), Some((12.5, Some(Unit::Px))));
        assert_eq!(parse("50%"), Some((50.0, Some(Unit::Percent))));
        assert_eq!(parse("3.14rad"), Some((3.14, Some(Unit::Radians))));
        assert_eq!(parse("7m"), Some((7.0, Some(Unit::Meters))));
    }

    #[test]
    fn parse_bare_number_has_no_unit() {
        assert_eq!(parse("42"), Some((42.0, None)));
    }

    #[test]
    fn parse_rejects_non_numeric() {
        assert_eq!(parse("abc"), None);
        assert_eq!(parse("deg"), None);
    }

    #[test]
    fn state_propagates_to_input() {
        let w =
            NumericInputWithUnit::new(NumberInput::new(NodeId(1), "Rotation", 90.0), Unit::Degrees)
                .state(TextInputState::Disabled);
        assert_eq!(w.input.state, TextInputState::Disabled);
    }

    #[test]
    fn input_and_chip_partition_host_without_overlap() {
        let widget =
            NumericInputWithUnit::new(NumberInput::new(NodeId(1), "Rotation", 90.0), Unit::Degrees);
        let host = Rect::new(0.0, 0.0, 200.0, 28.0);
        let field = widget.input_rect(host);
        let chip = widget.unit_rect(host);
        assert!(field.x + field.w <= chip.x + 0.01);
        assert!(chip.x + chip.w <= host.x + host.w + 0.01);
    }

    fn smoke(unit: Unit, state: TextInputState, theme: Theme) {
        let mut scene = VectorScene::new();
        let mut text = TextSystem::without_system_fonts();
        let widget =
            NumericInputWithUnit::new(NumberInput::new(NodeId(1), "Field", 1.0), unit).state(state);
        paint_numeric_input_with_unit(
            &widget,
            None,
            0,
            None,
            Rect::new(0.0, 0.0, 200.0, 28.0),
            &mut scene,
            &mut text,
            theme,
        );
        let _ = widget.build_a11y(0.0, 0.0, 200.0, 28.0);
    }

    #[test]
    fn paint_smoke_all_units() {
        for u in [
            Unit::Px,
            Unit::Meters,
            Unit::Degrees,
            Unit::Radians,
            Unit::Percent,
        ] {
            smoke(u, TextInputState::Normal, Theme::Forge);
        }
    }

    #[test]
    fn paint_smoke_disabled() {
        smoke(Unit::Degrees, TextInputState::Disabled, Theme::Sunstone);
    }
}
