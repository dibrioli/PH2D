#![forbid(unsafe_code)]

//! ph2d-painter-effects — layer blend modes + adjustment-layer effects.
//!
//! Extracted verbatim from `ph2d-painter-brush` when the painting/brush
//! engine was deleted. The **layers + effects** subsystem is preserved as a
//! working feature; this crate is its effects half:
//!
//! - [`blend`] — the 22 canonical W3C blend modes ([`BlendMode`] + [`apply`]),
//!   operating on straight linear-sRGB RGBA. Consumed by the CPU compositor
//!   in `ph2d-tool-painter` and mirrored by the GPU compositor in `ph2d-render`.
//! - [`adjustments`] — non-destructive adjustment layers (HSB, Curves, Levels,
//!   Gaussian/Motion blur, Sharpen, Bloom, Shadows/Highlights, Color Lookup,
//!   Gradient Map, Channel Mixer, Selective Color, …) + their CPU kernels.
//!
//! Contract caps for the effects surface are gated by
//! `ph2d-painter-contracts::architecture_painter_contract_surface`.

pub mod adjustments;
pub mod blend;

pub use blend::{BlendMode, MAX_BLEND_MODES, apply as apply_blend};
