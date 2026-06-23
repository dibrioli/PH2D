//! `ph2d-color` — typed color-space wrappers for PH2D's render and
//! image-edit pipelines.
//!
//! Wave 10 / Etapa 5.4. The motivation is to eliminate a recurring bug
//! class where `Vec<u8>` / `[u8; 4]` / `(f32, f32, f32, f32)` flowed
//! between tools, the renderer and panel painters with NO type
//! distinction between sRGB-encoded bytes and linear floats. Mixing
//! the two without conversion produced subtle gamma errors
//! (premultiplied dark edges, washed-out histograms — see UI_Bugs §4
//! and Image Tools Bugs §1).
//!
//! ## Type map
//!
//! | Type                | Channels        | Encoding                  | Notes                                          |
//! |---------------------|-----------------|---------------------------|------------------------------------------------|
//! | [`SrgbRgba`]        | `[u8; 4]`       | sRGB gamma + 8-bit        | Wire format (file IO, web, panels)             |
//! | [`LinearRgba`]      | `[f32; 4]`      | linear-light, [0..1]      | Render math (blending, filters)                |
//! | [`Premultiplied<T>`]| wraps `T`       | alpha-premultiplied       | Marker — required for `wgpu` blend modes       |
//! | [`OklchColor`]      | `f32 × 4`       | OKLCH (L,C,H,A) polar     | Design tokens, hue-stable interpolation        |
//! | [`OklabColor`]      | `f32 × 4` Pod   | OKLab (L,a,b,α) cartesian | Brush stamp wire format (ADR-0044 §2.3)        |
//! | [`PigmentLinearSrgb`]| alias `LinearRgba` | linear-light, [0..1]   | Mixbox pigment mixing working space (ADR-0044 §2.5) |
//!
//! ## Conversion rules
//!
//! Conversions are EXPLICIT — there is no `From<SrgbRgba> for LinearRgba`
//! impl. Use the named methods (`to_linear()`, `to_srgb()`, `premultiply()`)
//! so the cost (gamma transfer, multiply) is visible at the call site.
//!
//! ## Boundary policy
//!
//! - Inside `ph2d-render`: linear-light + premultiplied is the canonical
//!   form. Sample atlases as sRGB, convert to linear in the shader, blend
//!   in linear, convert back at present.
//! - Inside `ph2d-tool-*`: image edits operate on `Premultiplied<SrgbRgba>`
//!   buffers (the on-disk format) but document conversions when working
//!   in linear (e.g. `unmultiply()` → linear math → `premultiply()`).
//! - Inside `ph2d-panel-*`: design tokens are OKLCH (`ColorToken::resolve()`
//!   returns OKLCH internally, materializes as sRGB for paint).

#![forbid(unsafe_code)]

pub mod color_ramp;
pub mod linear;
pub mod oklab;
pub mod oklch;
pub mod pigment_space;
pub mod premultiplied;
pub mod srgb;

pub use color_ramp::{ColorRamp, MAX_RAMP_STOPS, RampColorMode, RampHue, RampInterp, RampStop};
pub use linear::LinearRgba;
pub use oklab::OklabColor;
pub use oklch::OklchColor;
pub use pigment_space::PigmentLinearSrgb;
pub use premultiplied::Premultiplied;
pub use srgb::SrgbRgba;
