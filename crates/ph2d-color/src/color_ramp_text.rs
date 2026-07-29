//! Compact **text form** of a [`ColorRamp`] — the serialization the Motion Nodes
//! text-param channel carries (doc 32: a param that is *not one f32* lives as a
//! string on the `Graph`, never in the frozen `NodeManifest`). This is the
//! gradient's exact analog of `ph2d-curve`'s `serialize`/`parse` for the Curve
//! text param, and it exists for the SAME reason: `motion.color_ramp`'s Custom
//! ramp is multi-stop, and a variable-length list of coloured stops cannot be a
//! fixed set of `f32` params.
//!
//! **Format** — mirrors `ph2d-curve`'s (`c1 x:y:interp …`):
//! ```text
//! g1 <interp_u8> <pos>:<r>,<g>,<b> <pos>:<r>,<g>,<b> …
//! ```
//! - `g1` is the version tag (a later field is a new tag, never a silent extra
//!   token — [`parse_gradient`] rejects a malformed stop rather than ignoring it).
//! - `<interp_u8>` is [`RampInterp::to_u8`] — a **global** interp for the whole
//!   ramp (Blender's Color Ramp interpolation dropdown). It lives in the STRING,
//!   not a sibling `f32` param, because the GPU LUT fill ([`LutSpec::fill`]) only
//!   ever sees this string — the interp has to travel with the stops or the
//!   device bake could not match the CPU `eval`.
//! - each stop is `pos:r,g,b` with `{}`-formatted `f32` (Rust's shortest decimal
//!   that round-trips), in **linear** RGB — the wire space the `tint` column and
//!   the compositor use. Alpha is implicit `1.0` (an opaque tint, matching every
//!   `motion.color_ramp` preset); a per-stop alpha is an append-only future field.
//!
//! Colour mode is always [`RampColorMode::Rgb`] and hue [`RampHue::Near`] in v1
//! (the two the node evaluates); an HSV/HSL custom ramp is a future token, not a
//! reinterpretation of an old string.

use crate::color_ramp::{ColorRamp, RampColorMode, RampInterp, RampStop};

/// Serialize a ramp to the compact text form (the inverse of [`parse_gradient`]).
/// Alpha is dropped (implicit `1.0`); colour mode/hue are not stored in v1.
#[must_use]
pub fn serialize_gradient(ramp: &ColorRamp) -> String {
    let mut s = format!("g1 {}", ramp.interp.to_u8());
    for stop in ramp.stops() {
        // `{}` on f32 is the shortest decimal that round-trips (Rust's Grisu/Ryū),
        // so parse-then-serialize is byte-stable.
        let [r, g, b, _a] = stop.color;
        s.push_str(&format!(" {}:{},{},{}", stop.pos, r, g, b));
    }
    s
}

/// Parse the compact text form. Returns `None` for anything malformed OR for
/// fewer than two stops — a one-stop "gradient" has nothing to interpolate, so
/// the caller falls back to a sensible default (`ColorRamp::default()`), exactly
/// as `ph2d-curve::parse` returns `None` on a degenerate curve.
#[must_use]
pub fn parse_gradient(s: &str) -> Option<ColorRamp> {
    let mut it = s.split_whitespace();
    if it.next()? != "g1" {
        return None;
    }
    let interp = RampInterp::from_u8(it.next()?.parse::<u8>().ok()?);
    let mut stops = Vec::new();
    for tok in it {
        let (pos_str, rgb_str) = tok.split_once(':')?;
        let pos = pos_str.parse::<f32>().ok()?;
        let mut c = rgb_str.split(',');
        let r = c.next()?.parse::<f32>().ok()?;
        let g = c.next()?.parse::<f32>().ok()?;
        let b = c.next()?.parse::<f32>().ok()?;
        if c.next().is_some() {
            return None; // a 4th channel is malformed, not extra data to ignore.
        }
        if !(pos.is_finite() && r.is_finite() && g.is_finite() && b.is_finite()) {
            return None;
        }
        stops.push(RampStop::new(pos, [r, g, b, 1.0]));
    }
    if stops.len() < 2 {
        return None;
    }
    Some(ColorRamp::new(stops, RampColorMode::Rgb, interp))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Round-trip: serialize then parse gives back the same stops + interp,
    /// bit-for-bit (the shortest-decimal `{}` form is stable).
    #[test]
    fn parse_is_the_inverse_of_serialize() {
        let ramp = ColorRamp::new(
            vec![
                RampStop::new(0.0, [1.0, 0.0, 0.0, 1.0]),
                RampStop::new(0.5, [0.0, 1.0, 0.0, 1.0]),
                RampStop::new(1.0, [0.0, 0.0, 1.0, 1.0]),
            ],
            RampColorMode::Rgb,
            RampInterp::Ease,
        );
        let s = serialize_gradient(&ramp);
        let back = parse_gradient(&s).expect("round-trips");
        assert_eq!(back.interp, RampInterp::Ease, "interp survives");
        assert_eq!(back.len(), 3, "stop count survives");
        for (a, b) in ramp.stops().iter().zip(back.stops()) {
            assert_eq!(a.pos, b.pos, "pos byte-stable");
            assert_eq!(a.color[..3], b.color[..3], "rgb byte-stable");
            assert_eq!(b.color[3], 1.0, "alpha is implicit 1.0");
        }
    }

    /// A serialized string that we then parse and re-serialize is byte-identical
    /// (the format is canonical — no drift on re-save).
    #[test]
    fn serialize_is_the_inverse_of_parse() {
        let s = "g1 0 0:1,0,0 0.5:0,1,0 1:0,0,1";
        let ramp = parse_gradient(s).unwrap();
        assert_eq!(serialize_gradient(&ramp), s, "canonical round-trip");
    }

    /// Malformed / degenerate strings return `None` so the caller uses its
    /// default — never a half-built ramp.
    #[test]
    fn malformed_and_degenerate_return_none() {
        assert!(parse_gradient("").is_none(), "empty");
        assert!(
            parse_gradient("c1 0 0:0,0,0 1:1,1,1").is_none(),
            "wrong tag"
        );
        assert!(parse_gradient("g1 0 0.5:0.5,0.5,0.5").is_none(), "one stop");
        assert!(
            parse_gradient("g1 0 0:1,0,0,9 1:0,0,1").is_none(),
            "a 4th channel is malformed"
        );
        assert!(
            parse_gradient("g1 0 0:1,0 1:0,0,1").is_none(),
            "a missing channel is malformed"
        );
        assert!(
            parse_gradient("g1 0 nan:1,0,0 1:0,0,1").is_none(),
            "non-finite pos rejected"
        );
    }

    /// The interp `u8` travels in the string (it is the ONLY place the GPU LUT
    /// fill can read it). A different interp yields a different string and a
    /// different parsed ramp.
    #[test]
    fn interp_rides_the_string() {
        // `RampInterp::to_u8`: Ease=0, Linear=2 (Blender's menu order).
        let linear = "g1 2 0:0,0,0 1:1,1,1";
        let ease = "g1 0 0:0,0,0 1:1,1,1";
        assert_eq!(parse_gradient(linear).unwrap().interp, RampInterp::Linear);
        assert_eq!(parse_gradient(ease).unwrap().interp, RampInterp::Ease);
        assert_ne!(linear, ease, "the interp is a distinguishing token");
    }
}
