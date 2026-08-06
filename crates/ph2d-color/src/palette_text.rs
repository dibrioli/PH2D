//! Compact **text form** of an ordered COLOUR PALETTE — the serialization the Motion
//! Nodes text-param channel carries for `motion.color_array` (doc 32: a param that is
//! *not one f32* lives as a string on the `Graph`, never in the frozen `NodeManifest`).
//!
//! # Why a palette is not a gradient
//!
//! [`crate::color_ramp_text`] is the neighbouring format and the obvious thing to reuse,
//! and it is the wrong shape: a ramp's stops carry a **position** and the ramp carries an
//! **interpolation**, because a ramp answers *what colour is at `t`*. A palette answers
//! *what is the `i`-th colour* — the elements are a LIST, not samples of a curve. Storing
//! positions a palette never reads would put two controls on screen that change nothing,
//! which is the dead knob this codebase keeps one table per menu to prevent.
//!
//! # Format
//!
//! ```text
//! p1 <r>,<g>,<b>,<a> <r>,<g>,<b>,<a> …
//! ```
//!
//! - `p1` is the version tag. A later field is a NEW TAG, never a silent extra token —
//!   [`parse_palette`] rejects a malformed entry rather than ignoring it, so a string
//!   from the future reads as "not a palette" instead of as a shorter one.
//! - each colour is `r,g,b,a` with `{}`-formatted `f32` (Rust's shortest decimal that
//!   round-trips), in **linear** RGBA — the wire space the `tint` column and the
//!   compositor use, the same space `color_ramp_text` writes.
//!
//! ⚠️ **There is no length limit here, and that is the point** (Enio: *"color array
//! poderia ter quantas cores o usuário quisesse, tire os limites"*). The node's cap was
//! four because four was how many `ParamSpec`s somebody wrote down — a limit of the
//! REPRESENTATION wearing the clothes of a decision. A `Vec` costs 16 bytes a colour and
//! the cycle reads `len()`, so nothing downstream has an opinion about how many there are.

/// The palette a node with nothing authored paints with — red / green / blue / yellow.
///
/// ⚠️ **It lives HERE so the node and the panel read the SAME list.** A swatch strip that
/// fell back to a different default than the cook would describe colours nobody paints,
/// which is the two-doors failure this codebase keeps naming. It is a DEFAULT, never a
/// cap: the length lives in the string.
pub const DEFAULT_PALETTE_FALLBACK: &[[f32; 4]] = &[
    [1.0, 0.0, 0.0, 1.0],
    [0.0, 1.0, 0.0, 1.0],
    [0.0, 0.0, 1.0, 1.0],
    [1.0, 1.0, 0.0, 1.0],
];

/// The version tag every palette string starts with.
const TAG: &str = "p1";

/// Serialize a palette to its text form. An empty palette is the tag alone, which
/// round-trips to an empty `Vec` — the honest encoding of "no colours yet".
#[must_use]
pub fn serialize_palette(colors: &[[f32; 4]]) -> String {
    let mut s = String::from(TAG);
    for c in colors {
        s.push(' ');
        s.push_str(&format!("{},{},{},{}", c[0], c[1], c[2], c[3]));
    }
    s
}

/// Parse a palette from its text form.
///
/// `None` for anything that is not a well-formed `p1` string — a missing tag, a colour
/// without four components, a component that is not a number. Refusing is what keeps a
/// typo from silently becoming a SHORTER palette: the caller then falls back to its
/// default, which is visible, instead of quietly losing the artist's last three colours.
#[must_use]
pub fn parse_palette(s: &str) -> Option<Vec<[f32; 4]>> {
    let mut it = s.split_whitespace();
    if it.next()? != TAG {
        return None;
    }
    let mut out = Vec::new();
    for tok in it {
        let mut n = tok.split(',');
        let mut c = [0.0f32; 4];
        for slot in &mut c {
            *slot = n.next()?.parse().ok()?;
        }
        if n.next().is_some() {
            return None; // a fifth component is a different format, not a longer colour
        }
        out.push(c);
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The round trip is exact, because `{}` on `f32` prints the shortest decimal that
    /// reads back to the same bits — the same reason `color_ramp_text` uses it. A palette
    /// that drifted by a ulp per save would be a document that changes colour by being
    /// opened.
    #[test]
    fn a_palette_round_trips_exactly() {
        let p = vec![
            [1.0, 0.0, 0.0, 1.0],
            [0.0, 1.0, 0.0, 0.5],
            [0.123_456_79, 0.25, 1.0, 1.0],
        ];
        assert_eq!(parse_palette(&serialize_palette(&p)), Some(p));
    }

    /// **No length limit.** The cap this format replaces was four `ParamSpec`s somebody
    /// wrote down; a hundred costs a `Vec` of a hundred.
    #[test]
    fn a_palette_has_no_length_limit() {
        #[expect(
            clippy::cast_precision_loss,
            reason = "a test index, exact well past 100"
        )]
        let big: Vec<[f32; 4]> = (0..100)
            .map(|i| [i as f32 / 100.0, 0.0, 0.0, 1.0])
            .collect();
        let back = parse_palette(&serialize_palette(&big)).expect("parses");
        assert_eq!(back.len(), 100);
        assert_eq!(back, big);
    }

    /// The empty palette is the tag alone and round-trips to an empty `Vec` — distinct
    /// from a malformed string, which is `None`. The caller can then tell "the artist
    /// removed every colour" from "this is not a palette".
    #[test]
    fn an_empty_palette_is_the_tag_alone() {
        assert_eq!(serialize_palette(&[]), "p1");
        assert_eq!(parse_palette("p1"), Some(Vec::new()));
    }

    /// **A malformed entry REFUSES the whole string, it does not shorten it.** Truncating
    /// at the first bad token would turn a typo into the silent loss of every colour after
    /// it — the failure a caller cannot see, because a shorter palette is a legal palette.
    #[test]
    fn a_malformed_entry_refuses_rather_than_truncating() {
        assert_eq!(parse_palette(""), None, "no tag");
        assert_eq!(parse_palette("g1 0:1,0,0"), None, "that is a gradient");
        assert_eq!(parse_palette("p1 1,0,0"), None, "three components");
        assert_eq!(parse_palette("p1 1,0,0,1,9"), None, "five components");
        assert_eq!(parse_palette("p1 1,0,0,1 x,0,0,1"), None, "not a number");
        // …and the good prefix of a bad string is not silently kept.
        assert_eq!(parse_palette("p1 1,0,0,1 nope"), None);
    }
}
