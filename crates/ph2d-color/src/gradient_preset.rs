//! Named **gradient presets** — the starting gradients a `ColorRamp` editor offers
//! (Rainbow / Heat / Ice / Grayscale). They are colour DATA, so they live here in the
//! colour leaf rather than in one node: any gradient editor loads them the same way, and
//! picking one **seeds the editable ramp** (its stops become draggable/recolourable) — never
//! a separate immutable "preset mode". A preset's stops are evenly spaced (`i/(n−1)`), RGB,
//! Linear interp — the shape you then edit.

use crate::color_ramp::{ColorRamp, RampColorMode, RampInterp, RampStop};

/// A built-in gradient the editor offers as a one-click starting point.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum GradientPreset {
    /// The colour wheel, closing back to red (7 stops).
    Rainbow,
    /// Black → red → orange → yellow → white (5 stops).
    Heat,
    /// Deep blue → cyan → pale (3 stops).
    Ice,
    /// Black → white (2 stops).
    Grayscale,
}

impl GradientPreset {
    /// Every preset, in menu order.
    pub const ALL: [GradientPreset; 4] = [
        GradientPreset::Rainbow,
        GradientPreset::Heat,
        GradientPreset::Ice,
        GradientPreset::Grayscale,
    ];

    /// English label (HR-15).
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            GradientPreset::Rainbow => "Rainbow",
            GradientPreset::Heat => "Heat",
            GradientPreset::Ice => "Ice",
            GradientPreset::Grayscale => "Grayscale",
        }
    }

    /// The preset as an editable [`ColorRamp`] — evenly-spaced RGB stops, Linear interp.
    #[must_use]
    pub fn ramp(self) -> ColorRamp {
        let rgb = |r: f32, g: f32, b: f32| [r, g, b, 1.0];
        let colors: &[[f32; 4]] = match self {
            GradientPreset::Rainbow => &[
                rgb(1.0, 0.0, 0.0),
                rgb(1.0, 1.0, 0.0),
                rgb(0.0, 1.0, 0.0),
                rgb(0.0, 1.0, 1.0),
                rgb(0.0, 0.0, 1.0),
                rgb(1.0, 0.0, 1.0),
                rgb(1.0, 0.0, 0.0),
            ],
            GradientPreset::Heat => &[
                rgb(0.0, 0.0, 0.0),
                rgb(0.7, 0.0, 0.0),
                rgb(1.0, 0.4, 0.0),
                rgb(1.0, 1.0, 0.2),
                rgb(1.0, 1.0, 1.0),
            ],
            GradientPreset::Ice => &[
                rgb(0.0, 0.0, 0.25),
                rgb(0.0, 0.55, 1.0),
                rgb(0.75, 1.0, 1.0),
            ],
            GradientPreset::Grayscale => &[rgb(0.0, 0.0, 0.0), rgb(1.0, 1.0, 1.0)],
        };
        let n = colors.len().max(2);
        let stops: Vec<RampStop> = colors
            .iter()
            .enumerate()
            .map(|(i, c)| RampStop::new(i as f32 / (n as f32 - 1.0), *c))
            .collect();
        ColorRamp::new(stops, RampColorMode::Rgb, RampInterp::Linear)
    }
}

/// The gradient a fresh editor / an unset gradient string opens on — **Rainbow**, so a
/// freshly-dropped `motion.color_ramp` is colourful and editable from the first frame
/// (the node's historical default), and the CPU eval, the GPU LUT fill and the panel all
/// fall back to the SAME ramp (they must agree on "nothing authored").
#[must_use]
pub fn default_gradient() -> ColorRamp {
    GradientPreset::Rainbow.ramp()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Each preset has the documented stop count, is RGB/Linear, and starts at position 0
    /// and ends at 1 (a valid gradient the editor can open on).
    #[test]
    fn presets_are_well_formed_ramps() {
        for p in GradientPreset::ALL {
            let r = p.ramp();
            assert!(r.len() >= 2, "{} has >= 2 stops", p.name());
            assert_eq!(r.color_mode, RampColorMode::Rgb);
            assert_eq!(r.interp, RampInterp::Linear);
            assert!(
                (r.stops()[0].pos - 0.0).abs() < 1e-6,
                "{} starts at 0",
                p.name()
            );
            assert!(
                (r.stops()[r.len() - 1].pos - 1.0).abs() < 1e-6,
                "{} ends at 1",
                p.name()
            );
        }
        assert_eq!(GradientPreset::Rainbow.ramp().len(), 7);
        assert_eq!(GradientPreset::Heat.ramp().len(), 5);
        assert_eq!(GradientPreset::Ice.ramp().len(), 3);
        assert_eq!(GradientPreset::Grayscale.ramp().len(), 2);
    }

    /// The default gradient is Rainbow and spans hues (a fresh node is colourful).
    #[test]
    fn the_default_gradient_is_the_colourful_rainbow() {
        let r = default_gradient();
        assert_eq!(r.len(), 7);
        // The blue channel sweeps across the wheel — not a flat colour.
        let (lo, hi) = r.stops().iter().fold((f32::MAX, f32::MIN), |(lo, hi), s| {
            (lo.min(s.color[2]), hi.max(s.color[2]))
        });
        assert!(hi - lo > 0.8, "rainbow spans (blue {lo}..{hi})");
    }
}
