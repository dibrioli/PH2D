//! The **local plane** — the least-squares fit that is the whole engine behind Flatten, Scrape and Fill
//! (`docs/Painter/18_plano_sculpt_relevo.md` §7).
//!
//! ## Why it is TILTED, and why that is the whole point
//!
//! The naive spatula flattens toward a horizontal plane at the local mean height. On a slope — which is
//! most of any painting — that does not flatten the surface, it **cuts a crater into the hillside**: the
//! uphill half is above the mean and gets scraped away, the downhill half is below it and gets filled in.
//! The brush-marks survive and the *form* is destroyed, which is the exact opposite of what the artist
//! asked for.
//!
//! A tilted plane — fitted to the footprint, gradient and all — reproduces the slope exactly, so the fit
//! removes **only the deviation from it**: the brush-marks, the grain, the bumps. That is the difference
//! between a tool and a bug, and it is one gate:
//! `flatten_on_a_pure_ramp_is_a_no_op` (`ph2d-tool-painter`, `sculpt_tests::plane`).
//!
//! ## The fit
//!
//! Weighted least squares of `h(x, y) ≈ a + gx·dx + gy·dy` over the dab's footprint, each texel weighted by
//! the **dab's own mask** — so the Shape image, the falloff, the Grain and the pressure all shape which
//! part of the surface the spatula is reading, exactly as they shape which part it touches. Nine
//! accumulators, one symmetric 3×3 solve, **zero transcendentals** (HR-5).
//!
//! The accumulators are `f64` deliberately: a dab of radius 100 sums ~40 000 samples, and an `f32`
//! accumulator drifts enough over that to tilt the plane visibly. The *output* is `f32` — the surface is.
//!
//! ## Isolation (ADR-0107)
//!
//! A sibling of [`crate::sculpt`], append-only, and a pure function of its inputs: no canvas, no brush
//! state, no session. It is `ph2d-painter-brush`'s share of Wave 2 and it touches nothing that exists.

/// A plane over the dab's local frame: `h(dx, dy) = h0 + gx·dx + gy·dy`, with `(dx, dy)` measured from the
/// dab centre in texels.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Plane {
    /// Height at the dab centre.
    pub h0: f32,
    /// Slope along x, in height-units per texel.
    pub gx: f32,
    /// Slope along y.
    pub gy: f32,
}

impl Plane {
    /// The plane's height at a point in the dab's local frame.
    #[must_use]
    pub fn at(&self, dx: f32, dy: f32) -> f32 {
        self.h0 + self.gx * dx + self.gy * dy
    }
}

/// Accumulator for the weighted least-squares fit. Feed it every texel of the footprint, then [`solve`].
///
/// [`solve`]: PlaneFit::solve
#[derive(Default, Clone, Debug)]
pub struct PlaneFit {
    w: f64,
    wx: f64,
    wy: f64,
    wxx: f64,
    wxy: f64,
    wyy: f64,
    wh: f64,
    wxh: f64,
    wyh: f64,
}

impl PlaneFit {
    /// One weighted sample: the surface height `h` at `(dx, dy)` in the dab's local frame, weight `w`.
    pub fn add(&mut self, dx: f32, dy: f32, h: f32, w: f32) {
        let (x, y, h, w) = (f64::from(dx), f64::from(dy), f64::from(h), f64::from(w));
        self.w += w;
        self.wx += w * x;
        self.wy += w * y;
        self.wxx += w * x * x;
        self.wxy += w * x * y;
        self.wyy += w * y * y;
        self.wh += w * h;
        self.wxh += w * x * h;
        self.wyh += w * y * h;
    }

    /// Solve the normal equations. `None` when there is no weight at all (an empty footprint).
    ///
    /// ```text
    ///   [ Σw    Σwx   Σwy  ] [h0]   [ Σwh  ]
    ///   [ Σwx   Σwxx  Σwxy ] [gx] = [ Σwxh ]
    ///   [ Σwy   Σwxy  Σwyy ] [gy]   [ Σwyh ]
    /// ```
    ///
    /// **Degenerate footprints fall back to the flat weighted mean**, and that is the right answer rather
    /// than a defensive shrug: a dab covering a single texel — or a perfect line of them — has no
    /// determinable slope, and the least-squares problem is genuinely singular there. Inventing a gradient
    /// from a singular system would give an arbitrary tilt whose direction depends on the last bit of the
    /// accumulator, which is exactly the sort of thing that renders differently on two machines. A flat
    /// plane through the mean is the fit's own limit as the footprint collapses.
    #[must_use]
    pub fn solve(&self) -> Option<Plane> {
        if self.w <= 0.0 {
            return None;
        }
        // Symmetric 3×3 by Cramer. The matrix is a Gram matrix (positive semi-definite by construction),
        // so `det >= 0` and it is 0 exactly when the sample points are collinear — the degenerate case.
        let (a, b, c) = (self.w, self.wx, self.wy);
        let (d, e) = (self.wxx, self.wxy);
        let f = self.wyy;
        // | a b c |
        // | b d e |
        // | c e f |
        let det = a * (d * f - e * e) - b * (b * f - e * c) + c * (b * e - d * c);
        // Scale-aware epsilon: `det` grows as (Σw)³·(spread²)², so a bare absolute threshold would call a
        // large, healthy fit "singular" at one radius and a truly collinear one "fine" at another.
        let scale = a * a * a;
        if !det.is_finite() || det.abs() <= 1e-9 * scale.max(1e-30) {
            return Some(Plane {
                h0: (self.wh / self.w) as f32,
                gx: 0.0,
                gy: 0.0,
            });
        }
        let (u, v, t) = (self.wh, self.wxh, self.wyh);
        let h0 = (u * (d * f - e * e) - b * (v * f - e * t) + c * (v * e - d * t)) / det;
        let gx = (a * (v * f - e * t) - u * (b * f - e * c) + c * (b * t - v * c)) / det;
        let gy = (a * (d * t - v * e) - b * (b * t - v * c) + u * (b * e - d * c)) / det;
        if !h0.is_finite() || !gx.is_finite() || !gy.is_finite() {
            return Some(Plane {
                h0: (self.wh / self.w) as f32,
                gx: 0.0,
                gy: 0.0,
            });
        }
        Some(Plane {
            h0: h0 as f32,
            gx: gx as f32,
            gy: gy as f32,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **A plane is fitted exactly.** The fit's whole job is to reproduce a slope; if it cannot reproduce
    /// one it was handed *exactly*, everything built on it is guesswork.
    ///
    /// **Mutation that must bleed:** return `gx: 0.0, gy: 0.0` from `solve` (the horizontal fit — the bug
    /// this module exists to prevent).
    #[test]
    fn an_exact_plane_is_recovered_exactly() {
        let mut fit = PlaneFit::default();
        let (h0, gx, gy) = (0.35_f32, 0.021_f32, -0.013_f32);
        for y in -20..=20 {
            for x in -20..=20 {
                let (dx, dy) = (x as f32, y as f32);
                // A non-uniform weight, because a uniform one would hide a bug in the cross terms.
                let w = 1.0 / (1.0 + 0.01 * (dx * dx + dy * dy));
                fit.add(dx, dy, h0 + gx * dx + gy * dy, w);
            }
        }
        let p = fit.solve().expect("a weighted footprint has a fit");
        assert!(
            (p.h0 - h0).abs() < 1e-4 && (p.gx - gx).abs() < 1e-5 && (p.gy - gy).abs() < 1e-5,
            "the fit did not recover the plane it was given: got h0={} gx={} gy={}, want h0={h0} gx={gx} gy={gy}",
            p.h0,
            p.gx,
            p.gy
        );
    }

    /// **Noise averages out; the slope survives.** A ramp buried in a saw-tooth still fits the ramp — which
    /// is precisely what lets Scrape take the brush-marks off a hillside without eating the hill.
    #[test]
    fn the_slope_survives_the_brush_marks() {
        let mut fit = PlaneFit::default();
        let (h0, gx) = (0.5_f32, 0.03_f32);
        for y in -15..=15_i32 {
            for x in -15..=15_i32 {
                let (dx, dy) = (x as f32, y as f32);
                let saw = if (x + y).rem_euclid(3) == 0 {
                    0.2
                } else {
                    -0.1
                };
                fit.add(dx, dy, h0 + gx * dx + saw, 1.0);
            }
        }
        let p = fit.solve().expect("a fit");
        assert!(
            (p.gx - gx).abs() < 2e-3,
            "the brush-marks tilted the plane: gx={} (want {gx})",
            p.gx
        );
        assert!(
            p.gy.abs() < 2e-3,
            "a ramp with no y-slope fitted gy={} — the cross terms are wrong",
            p.gy
        );
    }

    /// **A degenerate footprint gets the flat mean, not an invented tilt.** One texel has no slope, and a
    /// singular system solved anyway would produce a gradient whose sign lives in the accumulator's last
    /// bit — a different picture on a different machine (HR-5's whole concern).
    #[test]
    fn a_collinear_footprint_falls_back_to_the_mean() {
        // A single texel.
        let mut one = PlaneFit::default();
        one.add(0.0, 0.0, 0.7, 1.0);
        let p = one.solve().expect("one sample is still a fit");
        assert_eq!((p.h0, p.gx, p.gy), (0.7, 0.0, 0.0));

        // A perfect LINE of texels: the x-slope is determinable, the y-slope is not — but the system is
        // singular as a whole, so the honest answer is the flat mean rather than half an answer.
        let mut line = PlaneFit::default();
        for x in -5..=5 {
            line.add(x as f32, 0.0, 0.5 + 0.01 * x as f32, 1.0);
        }
        let p = line.solve().expect("a line is still a fit");
        assert!(
            p.gx.is_finite() && p.gy.is_finite() && p.h0.is_finite(),
            "a collinear footprint produced a non-finite plane: {p:?}"
        );
        assert_eq!(p.gy, 0.0, "a line along x cannot know a y-slope");

        // Nothing at all.
        assert!(PlaneFit::default().solve().is_none());
    }
}
