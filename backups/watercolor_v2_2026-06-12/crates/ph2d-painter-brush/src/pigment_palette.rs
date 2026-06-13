//! Real artist watercolor pigments (ADR-0081) — a curated palette where each pigment carries
//! its characteristic PHYSICAL behaviour on top of the K–M multi-pigment field (ADR-0080):
//!
//! - **granulation** — sedimentary pigments (French Ultramarine, Cerulean, Raw Umber) settle in
//!   the paper tooth valleys; staining/smooth pigments (Phthalo, Quinacridone) are ~0.
//! - **staining** — 1 = permanent stain that binds the paper and resists lifting (Phthalo,
//!   Quinacridone); 0 = fully liftable/sedimentary (Ultramarine, Raw Sienna).
//! - **transparency** — 1 = transparent glaze (Quinacridone, Phthalo); 0 = opaque (Cadmium,
//!   Cerulean, Titanium White).
//!
//! These are STANDARD artist characterisations (Winsor&Newton / Daniel Smith handbooks), not
//! measured spectral data. The masstone (full-strength) `srgb` feeds the K–M field's K/S; the
//! dilute undertone emerges for free from the field's `mass` (concentration). The behaviour
//! travels per-cell in the field (ADR-0081 §2.2), so a pigment keeps granulating/staining even
//! after it mixes with another. Picking a pigment is OPT-IN — no pigment selected = raw colour
//! (the validated path is bit-identical).

use crate::watercolor::WatercolorParams;
use ph2d_color::srgb::srgb_to_linear_byte;

/// A real watercolor pigment with its characteristic physical behaviour (ADR-0081).
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Pigment {
    /// Display name (English — UI canon HR-15).
    pub name: &'static str,
    /// Masstone (full-strength) colour, sRGB-8. The K–M field derives K/S from this; dilution is
    /// the field's `mass`, so masstone↔undertone is free.
    pub srgb: [u8; 3],
    /// Granulation ∈ [0,1] — settling in the paper tooth valleys (mapped to the deposition
    /// granulation; [`PIGMENT_GRANULATION_MAX`] is the physical scale).
    pub granulation: f32,
    /// Staining ∈ [0,1] — 1 = permanent stain (resists lifting), 0 = liftable/sedimentary.
    pub staining: f32,
    /// Transparency ∈ [0,1] — 1 = transparent glaze, 0 = opaque.
    pub transparency: f32,
}

/// Physical `WatercolorParams.granulation` a pigment's `granulation = 1.0` maps to (within the
/// control's `[0, 8]` range, ADR-0079). A heavy granulator (~0.85) → ~5; smooth (0) → 0.
pub const PIGMENT_GRANULATION_MAX: f32 = 6.0;

impl Pigment {
    /// Masstone colour in linear sRGB (the K–M field's K/S source).
    #[must_use]
    pub fn color_linear(&self) -> [f32; 3] {
        [
            srgb_to_linear_byte(self.srgb[0]),
            srgb_to_linear_byte(self.srgb[1]),
            srgb_to_linear_byte(self.srgb[2]),
        ]
    }

    /// The physical `WatercolorParams.granulation` value for this pigment (`granulation ×
    /// PIGMENT_GRANULATION_MAX`, clamped to the control range). The dab carries this per-cell so
    /// the granulation survives mixing; the per-brush slider is the default for raw-colour dabs.
    #[must_use]
    pub fn granulation_param(&self) -> f32 {
        (self.granulation * PIGMENT_GRANULATION_MAX).clamp(0.0, 8.0)
    }

    /// Apply this pigment's granulation to a brush's watercolor params (the colour + the per-dab
    /// `staining` are wired separately by the tool). Other params (diffusion/flow/capillary) are
    /// left as the brush's — the pigment changes only the granulation behaviour here.
    pub fn apply_granulation(&self, wc: &mut WatercolorParams) {
        wc.granulation = self.granulation_param();
    }
}

/// The curated real-pigment palette (ADR-0081). A solid 18-pigment artist set spanning the wheel
/// plus earths and neutrals, with each pigment's standard granulation/staining/transparency.
/// APPEND-only is NOT required (this is data, not a contract index), but keep names stable for UI.
pub const PALETTE: &[Pigment] = &[
    // ── Neutrals ──
    Pigment {
        name: "Titanium White",
        srgb: [244, 244, 242],
        granulation: 0.10,
        staining: 0.05,
        transparency: 0.00,
    },
    Pigment {
        name: "Lamp Black",
        srgb: [22, 22, 24],
        granulation: 0.00,
        staining: 0.70,
        transparency: 0.60,
    },
    Pigment {
        name: "Payne's Grey",
        srgb: [42, 50, 66],
        granulation: 0.50,
        staining: 0.40,
        transparency: 0.55,
    },
    // ── Blues ──
    Pigment {
        name: "French Ultramarine",
        srgb: [42, 48, 128],
        granulation: 0.85,
        staining: 0.15,
        transparency: 0.60,
    },
    Pigment {
        name: "Phthalo Blue",
        srgb: [12, 42, 96],
        granulation: 0.00,
        staining: 0.95,
        transparency: 0.90,
    },
    Pigment {
        name: "Cerulean Blue",
        srgb: [34, 112, 176],
        granulation: 0.80,
        staining: 0.20,
        transparency: 0.25,
    },
    Pigment {
        name: "Cobalt Blue",
        srgb: [32, 58, 150],
        granulation: 0.40,
        staining: 0.20,
        transparency: 0.60,
    },
    // ── Greens ──
    Pigment {
        name: "Phthalo Green",
        srgb: [0, 78, 66],
        granulation: 0.00,
        staining: 0.90,
        transparency: 0.90,
    },
    Pigment {
        name: "Viridian",
        srgb: [12, 112, 92],
        granulation: 0.70,
        staining: 0.20,
        transparency: 0.70,
    },
    Pigment {
        name: "Sap Green",
        srgb: [62, 96, 36],
        granulation: 0.20,
        staining: 0.50,
        transparency: 0.80,
    },
    // ── Yellows / earths ──
    Pigment {
        name: "Cadmium Yellow",
        srgb: [252, 200, 24],
        granulation: 0.00,
        staining: 0.10,
        transparency: 0.10,
    },
    Pigment {
        name: "Hansa Yellow",
        srgb: [250, 224, 32],
        granulation: 0.00,
        staining: 0.40,
        transparency: 0.85,
    },
    Pigment {
        name: "Yellow Ochre",
        srgb: [198, 150, 56],
        granulation: 0.40,
        staining: 0.20,
        transparency: 0.35,
    },
    Pigment {
        name: "Raw Sienna",
        srgb: [164, 110, 50],
        granulation: 0.40,
        staining: 0.25,
        transparency: 0.60,
    },
    Pigment {
        name: "Burnt Sienna",
        srgb: [140, 66, 40],
        granulation: 0.45,
        staining: 0.30,
        transparency: 0.55,
    },
    Pigment {
        name: "Burnt Umber",
        srgb: [84, 54, 40],
        granulation: 0.35,
        staining: 0.35,
        transparency: 0.50,
    },
    // ── Reds / magentas ──
    Pigment {
        name: "Quinacridone Rose",
        srgb: [198, 30, 92],
        granulation: 0.00,
        staining: 0.90,
        transparency: 0.90,
    },
    Pigment {
        name: "Cadmium Red",
        srgb: [218, 46, 30],
        granulation: 0.00,
        staining: 0.20,
        transparency: 0.15,
    },
];

/// Look up a pigment by exact name (UI selection / replay).
#[must_use]
pub fn pigment_by_name(name: &str) -> Option<&'static Pigment> {
    PALETTE.iter().find(|p| p.name == name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn palette_is_curated_and_well_formed() {
        assert!(PALETTE.len() >= 12, "a usable artist palette");
        for p in PALETTE {
            assert!(!p.name.is_empty());
            for v in [p.granulation, p.staining, p.transparency] {
                assert!(
                    (0.0..=1.0).contains(&v),
                    "{} prop out of [0,1]: {v}",
                    p.name
                );
            }
            let c = p.color_linear();
            assert!(
                c.iter().all(|&x| (0.0..=1.0).contains(&x)),
                "{} colour out of range",
                p.name
            );
            assert!(
                p.granulation_param() <= 8.0,
                "{} granulation within the control cap",
                p.name
            );
        }
    }

    #[test]
    fn pigment_characterisations_are_distinct_and_correct() {
        // The defining contrasts the feature exists to express (ADR-0081).
        let ultra = pigment_by_name("French Ultramarine").unwrap();
        let phthalo = pigment_by_name("Phthalo Blue").unwrap();
        // Ultramarine granulates + lifts (low staining); Phthalo is smooth + stains.
        assert!(
            ultra.granulation > 0.6 && ultra.staining < 0.3,
            "ultramarine granulates + lifts"
        );
        assert!(
            phthalo.granulation < 0.1 && phthalo.staining > 0.8,
            "phthalo smooth + stains"
        );
        assert!(
            ultra.granulation_param() > phthalo.granulation_param() + 3.0,
            "ultramarine deposits far more granular than phthalo"
        );
        // Cadmium is opaque; Quinacridone is a transparent stain.
        assert!(
            pigment_by_name("Cadmium Red").unwrap().transparency < 0.3,
            "cadmium opaque"
        );
        assert!(
            pigment_by_name("Quinacridone Rose").unwrap().transparency > 0.8,
            "quin transparent"
        );
    }

    #[test]
    fn apply_granulation_sets_only_granulation() {
        let mut wc = WatercolorParams::default();
        let before = wc;
        pigment_by_name("Cerulean Blue")
            .unwrap()
            .apply_granulation(&mut wc);
        assert!((wc.granulation - 4.8).abs() < 1e-4, "0.8 × 6.0 = 4.8"); // Cerulean granulation
        // Everything else untouched (non-destructive — only granulation changes).
        assert_eq!(wc.diffusivity, before.diffusivity);
        assert_eq!(wc.capillary, before.capillary);
        assert_eq!(wc.flow_outward, before.flow_outward);
    }
}
