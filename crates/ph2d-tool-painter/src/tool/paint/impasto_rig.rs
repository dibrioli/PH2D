//! The canvas's **light rig** — up to [`MAX_LIGHTS`] lights over one relief.
//!
//! ## The number, and the warning that set it
//!
//! Krita's Phong Bumpmap has **four** lights and **twenty-four controls** (4 × azimuth/inclination/colour
//! + ka/kd/ks/shininess) — and `docs/Painter/17_impasto_deposito_pesquisa2.md` §2.4 files it under *"o
//! conto-moral do excesso"*. Rebelle went the other way: it has **no light-angle knob at all** (the
//! direction is baked into environment maps; the dev said so on the record). ArtRage ships one light,
//! Angle + Intensity.
//!
//! So: four lights, because a key/fill/rim rig is a real thing an artist wants and one lamp cannot be
//! made to do it. But **one light on screen at a time** — the card edits the SELECTED light, so the row
//! count never grows with the rig. Lights 2-4 are **off** by default, which is what keeps a canvas that
//! nobody has lit byte-identical to the single-light build.
//!
//! ## The contract: flat paint stays byte-identical — even under coloured light
//!
//! The shading is RELATIVE (`impasto_light`): a pixel's response is divided by a FLAT surface's. That
//! survives the rig, and coloured light does not break it, because the division is **per channel**:
//!
//! ```text
//! diffuse[c] = Σ  wᵢ · colourᵢ[c] · max(N·Lᵢ, 0)
//! flat[c]    = Σ  wᵢ · colourᵢ[c] · Lᵢ.z          (a flat surface: N = (0,0,1) ⇒ N·Lᵢ = Lᵢ.z)
//! ratio[c]   = diffuse[c] / flat[c]
//! ```
//!
//! On flat paint `N·Lᵢ = Lᵢ.z` for every light, so **every channel's ratio is exactly 1** — whatever the
//! colours, whatever the intensities. A warm key and a cold fill therefore tint the paint only where it
//! TILTS, and a flat painting under a red lamp does not turn red. That is not a compromise forced by the
//! contract; it is what a *relative* model means, and it is the only reading under which "the light" is
//! a property of the surface's relief rather than a filter over the picture.

/// How many lights the rig holds. Krita's number (see the module docs) — and the ceiling, not a target:
/// light 0 is the key, and the artist who never opens the rig has exactly one lamp. // CLAMP-OK
pub const MAX_LIGHTS: usize = 4;

/// [`MAX_LIGHTS`], under a name that survives leaving this module (the panel imports it).
pub const MAX_IMPASTO_LIGHTS: usize = MAX_LIGHTS;

/// Lowest elevation a light may sit at, in degrees. At 0 the light grazes the canvas, the flat response
/// goes to zero, and the relative shading divides by ~0. // CLAMP-OK
pub const MIN_ELEV_DEG: u16 = 5;

/// One lamp over the canvas.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ImpastoLight {
    /// Lit at all. Light 0 is the key and starts on; 1..3 start **off**, so a fresh canvas is
    /// byte-identical to the single-light build.
    pub on: bool,
    /// Azimuth in whole degrees (`0..360`) — where the lamp is around the canvas.
    pub angle_deg: u16,
    /// Elevation in whole degrees above the canvas plane (`MIN_ELEV_DEG..=90`).
    pub elev_deg: u16,
    /// How bright (`0..2`). A fill lamp at 0.3 against a key at 1 is the everyday rig.
    pub intensity: f32,
    /// The lamp's colour (linear RGB, `0..1`). White is neutral; a warm key against a cool fill is the
    /// oldest trick in painting, and it costs nothing here because the shading is relative.
    pub color: [f32; 3],
}

impl ImpastoLight {
    /// The KEY light — Enio's dialled-in default (2026-07-12): upper-left at 30°, the reading every eye
    /// resolves as raised rather than engraved.
    pub const KEY: Self = Self {
        on: true,
        angle_deg: 230,
        elev_deg: 30,
        intensity: 1.0,
        color: [1.0, 1.0, 1.0],
    };

    /// A lamp the artist has not switched on yet. Its ANGLE is not the key's — an artist who ticks
    /// "Enable" on light 2 and sees nothing change would rightly call the switch broken, and two lamps
    /// in the same place are one lamp. Fill light: opposite the key, low, dim and cool, which is the
    /// second lamp of every rig ever set up.
    pub const FILL: Self = Self {
        on: false,
        angle_deg: 50, // opposite the key (230 − 180)
        elev_deg: 25,
        intensity: 0.4,
        color: [0.85, 0.9, 1.0], // cool, against a warm key
    };
}

impl Default for ImpastoLight {
    fn default() -> Self {
        Self::KEY
    }
}

/// The whole rig, as the tool holds it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LightRig {
    pub lights: [ImpastoLight; MAX_LIGHTS],
    /// Which lamp the Lighting card is editing (`0..MAX_LIGHTS`). Editing state, not appearance — it
    /// changes no pixel, and it lives here so the panel can stay a pure function of the snapshot.
    pub selected: u8,
}

impl Default for LightRig {
    fn default() -> Self {
        Self {
            lights: [
                ImpastoLight::KEY,
                ImpastoLight::FILL,
                ImpastoLight::FILL,
                ImpastoLight::FILL,
            ],
            selected: 0,
        }
    }
}

impl LightRig {
    /// The lamp the card is editing. Never panics on a bad index — it clamps, because a snapshot that
    /// arrives with a stale selection must not take the panel down.
    #[must_use]
    pub fn current(&self) -> &ImpastoLight {
        &self.lights[(self.selected as usize).min(MAX_LIGHTS - 1)]
    }

    /// Mutable [`Self::current`].
    pub fn current_mut(&mut self) -> &mut ImpastoLight {
        &mut self.lights[(self.selected as usize).min(MAX_LIGHTS - 1)]
    }

    /// Whether any lamp is lit. The pass early-outs on `false` — an artist who switches every light off
    /// gets the unlit canvas back, to the byte, rather than a division by zero.
    #[must_use]
    pub fn any_on(&self) -> bool {
        self.lights.iter().any(|l| l.on && l.intensity > 0.0)
    }
}
