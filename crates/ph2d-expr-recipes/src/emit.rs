//! **Turning knobs into text** — the three helpers every recipe funnels through.
//!
//! Each is a single door on purpose: a recipe that formats its own number, or
//! decides its own parenthesisation, is a recipe that gets it wrong once and
//! nobody notices until a stack composes in the wrong order.

use crate::knob::{Knob, KnobValue};

/// What a recipe is handed when it emits.
pub struct EmitCtx<'a> {
    /// The row's knob values, positionally matched to [`Recipe::knobs`].
    ///
    /// [`Recipe::knobs`]: crate::Recipe::knobs
    pub knobs: &'a [KnobValue],
    /// The recipe's knob DEFINITIONS, same order — what [`EmitCtx::lit`] clamps
    /// against.
    pub defs: &'a [Knob],
    /// The formula accumulated by the rows ABOVE this one (`"value"` at the top).
    pub inner: &'a str,
    /// The clock expression in force here (`"time"` unless a [`RowKind::Time`] row
    /// changed it).
    ///
    /// [`RowKind::Time`]: crate::RowKind::Time
    pub clock: &'a str,
}

impl EmitCtx<'_> {
    /// Knob `i` as a formatted numeric literal.
    #[must_use]
    pub fn n(&self, i: usize) -> String {
        fmt_num(self.knobs.get(i).map_or(0.0, KnobValue::as_num))
    }

    /// Knob `i` as a raw number.
    #[must_use]
    pub fn num(&self, i: usize) -> f32 {
        self.knobs.get(i).map_or(0.0, KnobValue::as_num)
    }

    /// Knob `i` as a literal **clamped to its declared range**.
    ///
    /// ⚠️ For [`KnobKind::Literal`] knobs, and they are the ONE kind whose range is
    /// a validity claim rather than a drag range: the parser refuses `wiggle`
    /// octaves below 1 outright (*"octaves must be a number >= 1"*), so a typed 0
    /// would produce text that does not parse — a blank formula bar the instant
    /// the artist selects the field and presses Delete. Number knobs are NOT
    /// clamped on purpose: typing past the slider is the artist's business.
    ///
    /// [`KnobKind::Literal`]: crate::KnobKind::Literal
    #[must_use]
    pub fn lit(&self, i: usize) -> String {
        let v = self.num(i);
        let (lo, hi) = self.defs.get(i).map_or((v, v), |k| k.range);
        fmt_num(v.clamp(lo.min(hi), hi.max(lo)))
    }

    /// Knob `i` as a `Name.prop` link.
    ///
    /// ⚠️ An **unfilled** link emits `0`, not an empty string. The formula bar and
    /// the preview render on every keystroke, long before the pick-whip has been
    /// used; a row that emitted `.x` would blank the whole modal the moment a
    /// `Follow` row was added. Gated.
    #[must_use]
    pub fn link(&self, i: usize) -> String {
        match self.knobs.get(i) {
            Some(v @ KnobValue::Link(_)) if !v.as_text().is_empty() => v.as_text().to_string(),
            _ => "0".to_string(),
        }
    }

    /// A free-text knob, trimmed; empty when unset.
    #[must_use]
    pub fn text(&self, i: usize) -> &str {
        self.knobs.get(i).map_or("", KnobValue::as_text)
    }

    /// Knob `i` as a **non-zero** literal — `1` when it is zero.
    ///
    /// ⚠️ For knobs that land in a DENOMINATOR. A `Quantize` step of 0 or a
    /// `Stepped Time` rate of 0 would emit a division by zero: the formula still
    /// parses, and the property silently becomes `inf`. The knob range keeps this
    /// out of the artist's way; this keeps it out of the *arithmetic*.
    #[must_use]
    pub fn nz(&self, i: usize) -> String {
        let v = self.num(i);
        fmt_num(if v == 0.0 { 1.0 } else { v })
    }

    /// The HIGH end of an input range, nudged so `hi - lo` is never zero.
    ///
    /// A remap divides by `(hi - lo)`. A zero-width input range is a legitimate
    /// thing to type on the way to typing something else, so it must not produce
    /// `inf` for a frame — the range widens by one instead, and the artist sees the
    /// low output.
    #[must_use]
    pub fn span_hi(&self, lo: usize, hi: usize) -> String {
        let (a, b) = (self.num(lo), self.num(hi));
        fmt_num(if a == b { a + 1.0 } else { b })
    }

    /// The accumulated formula, parenthesised when it needs to be (see [`paren`]).
    #[must_use]
    pub fn tight(&self) -> String {
        paren(self.inner)
    }
}

/// A numeric literal the lexer accepts.
///
/// ⚠️ Non-finite becomes `0`: `inf` and `NaN` would lex as **identifiers**, so a
/// knob dragged to the edge of an `f32` would silently turn into an attribute
/// lookup that resolves to zero somewhere else entirely.
#[must_use]
pub fn fmt_num(v: f32) -> String {
    if !v.is_finite() {
        return "0".to_string();
    }
    // Rust's `Display` for f32 is shortest-round-trip and never scientific for the
    // magnitudes a knob reaches, so `2.0 -> "2"`, `0.5 -> "0.5"`, `-10.0 -> "-10"`.
    // A bare negative is safe in EVERY position: the grammar's `mul` takes `unary`
    // on its right, so `value*-2` parses (measured, not assumed).
    format!("{v}")
}

/// Wrap `s` in parentheses unless it is already an atom.
///
/// ⚠️ This is the one thing that makes a STACK correct rather than merely each row.
/// `Multiply/Add` emits `{inner}*{m} + {o}`; with `inner = "value + wiggle(2, 30)"`
/// the unparenthesised text means `value + (wiggle(2,30)*m) + o` — a different
/// animation, that parses. The pairwise-composition gate is what catches it.
#[must_use]
pub fn paren(s: &str) -> String {
    if is_atom(s) {
        s.to_string()
    } else {
        format!("({s})")
    }
}

/// An atom binds tighter than any operator, so it never needs wrapping: a bare
/// identifier (`value`, `Ball.x`) or a bare number. Deliberately conservative —
/// `(a+b)` starts with a paren but a leading paren does not prove the whole string
/// is one group (`(a+b)*c` is not), so we do not try to be clever.
fn is_atom(s: &str) -> bool {
    let s = s.trim();
    if s.is_empty() {
        return false;
    }
    let ident = s
        .chars()
        .enumerate()
        .all(|(i, c)| c.is_ascii_alphanumeric() || c == '_' || c == '.' || (i == 0 && c == '-'));
    ident && !s.contains(' ')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn atoms_are_left_alone_and_compounds_are_wrapped() {
        for a in ["value", "Ball.x", "12", "0.5", "-10", "time"] {
            assert_eq!(paren(a), a, "{a} is an atom");
        }
        for c in ["value + 1", "a*b", "min(a, b)", "value + wiggle(2, 30)"] {
            assert_eq!(paren(c), format!("({c})"), "{c} needs wrapping");
        }
    }

    #[test]
    fn a_non_finite_knob_becomes_zero_not_an_identifier() {
        assert_eq!(fmt_num(f32::INFINITY), "0");
        assert_eq!(fmt_num(f32::NAN), "0");
        assert_eq!(fmt_num(2.0), "2");
        assert_eq!(fmt_num(-10.0), "-10");
        assert_eq!(fmt_num(0.5), "0.5");
    }
}
