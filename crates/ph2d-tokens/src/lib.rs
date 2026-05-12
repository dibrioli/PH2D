#![forbid(unsafe_code)]
//! ph2d-tokens — design tokens (color/type/spacing/radius/motion/layer)
//! per ADR-0023 §12 + canonical design system at
//! [`docs/design/tokens.json`].
//!
//! ## Pipeline
//!
//! ```text
//! docs/design/tokens.json   ←  source of truth (4 themes, OKLCH)
//!         ↓ (manual sync — future codegen via build.rs)
//! crates/ph2d-tokens/src/   ←  Rust mirror (this crate)
//!         ↓
//! ph2d-editor widgets       ←  consume via ColorToken::resolve(theme)
//! ```
//!
//! Source of truth for everything that carries color, size, spacing,
//! radius, shadow, motion or z-stack in the UI. Semantic tokens (not
//! literals): widget code NEVER uses hex/px directly — always through
//! `ColorToken::resolve(theme)`, `Spacing::px()`, etc.
//!
//! ### Why
//!
//! - **4 themes** (`forge` default, `workshop`, `sunstone`,
//!   `blueprint`) become a lookup, not a code fork.
//! - **WCAG 2.2 AA contrast** validated by tests in this crate (not
//!   scattered). Tweaking one value ⇒ contrast test fails immediately.
//! - **OKLCH at the source** — perceptually uniform; sRGB conversion
//!   centralized in `oklch_to_srgb`.
//! - **Future lint** (`ph2d-clippy::no-literal-color`) fails any
//!   `Color::from_hex(...)` outside this crate.
//!
//! ### Out of scope
//!
//! - Rendering (Vello, raster) — lives in `ph2d-editor` / `ph2d-vector`.
//! - Theme persistence — lives in `ph2d-editor` settings.
//! - Automatic codegen tokens.json → this crate (planned; see
//!   "Pipeline" above — sync is manual for now).

pub mod color;
pub mod layer;
pub mod motion;
pub mod radius;
pub mod spacing;
pub mod theme;
pub mod typography;

pub use color::{Color, ColorToken, ColorValue, oklch_to_srgb, srgb_to_oklch};
pub use layer::Layer;
pub use motion::{Duration, Easing};
pub use radius::Radius;
pub use spacing::{Density, ICON_BTN_SIZE_PX, ROW_H_PX, SECTION_GAP_PX, Spacing};
pub use theme::{PanelLayout, Theme};
pub use typography::{
    FONT_DISPLAY, FONT_MONO, FONT_SANS, FontWeight, LetterSpacing, LineHeight, TypeToken,
};
