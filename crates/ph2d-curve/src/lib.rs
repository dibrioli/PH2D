#![forbid(unsafe_code)]
//! `ph2d-curve` — a normalized 1-D transfer curve (Motion Nodes A1).
//!
//! A curve is an ascending list of control points in the unit square, each
//! carrying the interpolation of the segment that FOLLOWS it. It is the shape a
//! `ParamWidget::Curve` row authors and the transfer a `field.remap` **Curve**
//! contour applies (`value.curve` / `force.curve` reuse it). Like `ph2d-expr`'s
//! IR this crate is dependency-free and value-typed; the consuming node stores
//! the [`serialize`]d string in a **text param** (`Graph::set_text_param`, doc
//! 32) — the frozen `ParamSpec` is f32-only, and a curve is not one number.
//!
//! **Transcendental-free (HR-5).** Every segment is a lerp, a `smoothstep`
//! polynomial (`u²(3−2u)`), or a hold — no trig, no `powf` — so the curve is
//! bit-identical CPU↔GPU once A1-gpu bakes it to a LUT.
//!
//! **The neutral is the identity.** An empty curve evaluates to `t` (a
//! `field.remap` Curve contour with nothing authored is an exact passthrough);
//! [`Curve::identity`] is the diagonal the editor opens on.

/// How a segment reaches the NEXT point from the point that carries it. The last
/// point's interp is inert (there is no following segment).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Interp {
    /// Straight line to the next point.
    Linear,
    /// `smoothstep` ease (`u²(3−2u)`): flat tangents at both ends. HR-5.
    Smooth,
    /// Hold this point's value until the next point (a step).
    Hold,
}

impl Interp {
    /// The single-char serialization tag.
    const fn tag(self) -> char {
        match self {
            Interp::Linear => 'L',
            Interp::Smooth => 'S',
            Interp::Hold => 'H',
        }
    }

    fn from_tag(c: &str) -> Option<Interp> {
        match c {
            "L" => Some(Interp::Linear),
            "S" => Some(Interp::Smooth),
            "H" => Some(Interp::Hold),
            _ => None,
        }
    }
}

/// One control point. `x`/`y` are conventionally in `[0, 1]` (a transfer over
/// the unit square) but nothing here clamps them — a curve that overshoots is a
/// legal authored shape, and the CONSUMER clamps its own output (`field.remap`
/// has a `clamp` param). `interp` governs the segment toward the next point.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Point {
    pub x: f32,
    pub y: f32,
    pub interp: Interp,
}

/// A 1-D transfer curve: control points **ascending in `x`**. The editor keeps
/// them sorted; [`eval`](Curve::eval) reads them in order (it does not sort — a
/// per-instance sort would allocate 262k times on a field, and the authoring
/// path guarantees the invariant).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Curve {
    pub points: Vec<Point>,
}

impl Curve {
    /// The diagonal `(0,0)→(1,1)`, linear — the identity transfer, and the shape
    /// the curve editor opens on. `eval(t) == t`.
    #[must_use]
    pub fn identity() -> Curve {
        Curve {
            points: vec![
                Point {
                    x: 0.0,
                    y: 0.0,
                    interp: Interp::Linear,
                },
                Point {
                    x: 1.0,
                    y: 1.0,
                    interp: Interp::Linear,
                },
            ],
        }
    }

    /// Sample the curve at `t` (clamped to `[0, 1]`).
    ///
    /// - **empty** → `t` (the identity — an unauthored Curve contour is a
    ///   passthrough);
    /// - **one point** → its `y` (a constant);
    /// - **before the first / after the last** point → that endpoint's `y` (the
    ///   curve is held flat outside its authored span);
    /// - **inside a segment** → the point's [`Interp`] between the two `y`s.
    ///
    /// Assumes the points are ascending in `x` (see the type doc). HR-5.
    #[must_use]
    pub fn eval(&self, t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);
        let p = &self.points;
        match p.len() {
            0 => t,
            1 => p[0].y,
            _ => {
                let last = p.len() - 1;
                if t <= p[0].x {
                    return p[0].y;
                }
                if t >= p[last].x {
                    return p[last].y;
                }
                // The segment `[p[i], p[i+1]]` that brackets `t`. A linear scan:
                // an authored curve has a handful of points, and the whole eval
                // is called per instance, so a branchy binary search would not
                // pay. The node parses the string ONCE per cook, not per element.
                let mut i = 0;
                while i + 1 < last && t > p[i + 1].x {
                    i += 1;
                }
                let a = p[i];
                let b = p[i + 1];
                let span = b.x - a.x;
                // Coincident x (a vertical jump): take the left value, so the
                // segment is a clean step rather than a divide-by-zero.
                let u = if span > 0.0 { (t - a.x) / span } else { 0.0 };
                match a.interp {
                    Interp::Hold => a.y,
                    Interp::Linear => a.y + (b.y - a.y) * u,
                    Interp::Smooth => {
                        let s = u * u * (3.0 - 2.0 * u);
                        a.y + (b.y - a.y) * s
                    }
                }
            }
        }
    }
}

/// Serialize a curve to a compact text-param string: `c1 x:y:tag x:y:tag …`,
/// where `tag` is `L`/`S`/`H` and each `x`/`y` is Rust's canonical shortest
/// round-trip `f32` `Display`. An empty curve is just `"c1"`.
///
/// **Byte-exact round-trip:** because `Display` is the canonical shortest form
/// and `parse::<f32>` is its exact inverse, `serialize(parse(s)) == s` for any
/// `s` this function could have produced (the gate `serialize_is_the_inverse_of_parse`).
#[must_use]
pub fn serialize(c: &Curve) -> String {
    let mut s = String::from("c1");
    for p in &c.points {
        // `{}` on f32 is the shortest decimal that round-trips (Rust's Grisu/Ryū);
        // parse then Display of the same value is byte-stable.
        s.push_str(&format!(" {}:{}:{}", p.x, p.y, p.interp.tag()));
    }
    s
}

/// Parse a curve produced by [`serialize`]. `None` on a missing/wrong header, a
/// malformed point, or an unknown interp tag — the caller (a node's `eval`)
/// treats `None` as the identity, exactly as an unset text param.
///
/// Order-preserving: it is the exact inverse of [`serialize`] and never
/// reorders points (that would break the byte-exact round-trip and is the
/// editor's job, not the parser's).
#[must_use]
pub fn parse(s: &str) -> Option<Curve> {
    let mut it = s.split_whitespace();
    if it.next()? != "c1" {
        return None;
    }
    let mut points = Vec::new();
    for tok in it {
        let mut f = tok.split(':');
        let x = f.next()?.parse::<f32>().ok()?;
        let y = f.next()?.parse::<f32>().ok()?;
        let interp = Interp::from_tag(f.next()?)?;
        if f.next().is_some() {
            return None; // a 4th `:field` is malformed, not extra data to ignore.
        }
        points.push(Point { x, y, interp });
    }
    Some(Curve { points })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn c(points: &[(f32, f32, Interp)]) -> Curve {
        Curve {
            points: points
                .iter()
                .map(|&(x, y, interp)| Point { x, y, interp })
                .collect(),
        }
    }

    #[test]
    fn empty_and_single_and_identity() {
        // Empty is the identity transfer.
        for t in [0.0, 0.25, 0.5, 0.75, 1.0] {
            assert_eq!(Curve::default().eval(t), t);
            assert_eq!(Curve::identity().eval(t), t);
        }
        // A single point is a constant.
        let k = c(&[(0.5, 0.3, Interp::Linear)]);
        assert_eq!(k.eval(0.0), 0.3);
        assert_eq!(k.eval(1.0), 0.3);
    }

    #[test]
    fn clamps_the_domain_and_holds_outside_the_span() {
        let s = c(&[(0.25, 0.4, Interp::Linear), (0.75, 0.9, Interp::Linear)]);
        // Outside `[0,1]` the input is clamped; outside the authored span the
        // output holds flat at the endpoint.
        assert_eq!(s.eval(-1.0), 0.4);
        assert_eq!(s.eval(0.0), 0.4);
        assert_eq!(s.eval(0.25), 0.4);
        assert_eq!(s.eval(0.75), 0.9);
        assert_eq!(s.eval(2.0), 0.9);
        // Midpoint of the linear segment.
        assert!((s.eval(0.5) - 0.65).abs() < 1e-6);
    }

    #[test]
    fn linear_smooth_hold_differ_where_they_must() {
        // Same endpoints, three interps: at the segment MIDPOINT they diverge.
        let lin = c(&[(0.0, 0.0, Interp::Linear), (1.0, 1.0, Interp::Linear)]);
        let smo = c(&[(0.0, 0.0, Interp::Smooth), (1.0, 1.0, Interp::Linear)]);
        let hol = c(&[(0.0, 0.0, Interp::Hold), (1.0, 1.0, Interp::Linear)]);
        // Linear: 0.5. Smooth: smoothstep(0.5)=0.5 too — so probe at u=0.25 where
        // they part (a fixture that contains the phenomenon).
        assert!((lin.eval(0.25) - 0.25).abs() < 1e-6);
        // smoothstep(0.25) = 0.25²·(3−0.5) = 0.15625.
        assert!((smo.eval(0.25) - 0.156_25).abs() < 1e-6);
        // Hold stays at the left value across the whole segment.
        assert_eq!(hol.eval(0.25), 0.0);
        assert_eq!(hol.eval(0.99), 0.0);
    }

    #[test]
    fn parse_is_the_inverse_of_serialize_struct() {
        let cur = c(&[
            (0.0, 0.0, Interp::Linear),
            (0.5, 0.75, Interp::Smooth),
            (0.5, 0.2, Interp::Hold), // coincident x (a vertical jump) is legal.
            (1.0, 1.0, Interp::Linear),
        ]);
        assert_eq!(parse(&serialize(&cur)), Some(cur));
    }

    #[test]
    fn serialize_is_the_inverse_of_parse_bytes() {
        // A canonical string round-trips BYTE for byte (the A1 acceptance).
        let canonical = "c1 0:0:L 0.5:0.75:S 1:1:L";
        assert_eq!(serialize(&parse(canonical).unwrap()), canonical);
        // Including the empty curve.
        assert_eq!(serialize(&parse("c1").unwrap()), "c1");
    }

    #[test]
    fn malformed_strings_reject() {
        assert_eq!(parse(""), None); // no header
        assert_eq!(parse("v1 0:0:L"), None); // wrong header
        assert_eq!(parse("c1 0:0"), None); // missing interp field
        assert_eq!(parse("c1 0:0:Z"), None); // unknown interp tag
        assert_eq!(parse("c1 0:0:L:extra"), None); // trailing field
        assert_eq!(parse("c1 x:0:L"), None); // non-numeric
    }
}
