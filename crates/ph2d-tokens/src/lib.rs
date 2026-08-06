#![forbid(unsafe_code)]
//! ph2d-tokens — design tokens (color/type/spacing/radius/motion/layer)
//! per ADR-0023 §12 + canonical design system at
//! [`docs/design/tokens.json`].
//!
//! ## Pipeline
//!
//! ```text
//! docs/design/tokens.json   ←  source of truth (4 themes, OKLCH)
//!         ↓ build.rs        ←  parses the JSON, emits `crate::generated::*`
//! crates/ph2d-tokens/src/   ←  enums read those consts
//!         ↓
//! ph2d-editor widgets       ←  consume via ColorToken::resolve(theme)
//! ```
//!
//! The codegen is real, and `tests/design_token_sync.rs` guards it by re-parsing
//! the JSON with an **independent** parser (`serde_json`) and asserting the public
//! token API agrees. A build.rs parser bug — or a renamed JSON key that still
//! matches something — fails CI instead of drifting silently.
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
//! - **Authoring** the tokens. The table is a hand-edited JSON and the four
//!   themes are enum variants; aliases, math, artist-defined modes and DTCG
//!   import/export are not here. See `docs/Vector Module/Estudos/
//!   PLANO_UI_UX_padrao_figma.md` §4/W4.

pub mod chrome;
pub mod color;
pub mod layer;
pub mod motion;
/// **A camada de OVERRIDE de cor** (plano UI/UX W6) — o que o artista autora sobre a tabela
/// gerada. Vazia, `ColorToken::resolve` é byte-idêntico ao de sempre.
pub mod contrast;
pub mod overrides;
pub mod radius;
pub mod spacing;
pub mod stroke;
pub mod theme;
pub mod typography;

/// Auto-generated tables from `docs/design/tokens.json` via `build.rs`.
/// See Wave 2 PR 11.1. Consumed by `color.rs::ColorToken::resolve`.
/// Module visibility: `pub(crate)` — runtime is the public API.
pub(crate) mod generated {
    include!(concat!(env!("OUT_DIR"), "/tokens_generated.rs"));
}

pub use chrome::{
    CHECKBOX_BOX_PX, DIVIDER_GAP_PX, EDGE_PAD_PX, HERO_VIEWPORT_H_PX, HERO_VIEWPORT_W_PX,
    HIER_ROW_H_PX, HIERARCHY_W_PX, HUD_BOTTOM_PAD_PX, HUD_H_PX, INSPECTOR_W_PX, PANEL_HEAD_PAD_PX,
    PANEL_RADIUS_PX, PANEL_RESIZE_HANDLE_SIZE_PX, PILL_PADDING_PX, TOOL_CHIP_PX, TOPBAR_GAP_PX,
    TOPBAR_H_PX,
};
pub use color::{
    Color, ColorToken, ColorValue, oklch_in_gamut, oklch_to_linear_srgb, oklch_to_srgb,
    srgb_to_oklch,
};
pub use layer::Layer;
pub use motion::{Duration, Easing};
pub use radius::Radius;
pub use spacing::{Density, ICON_BTN_SIZE_PX, ROW_H_PX, SECTION_GAP_PX, Spacing};
pub use stroke::StrokeToken;
pub use theme::{PanelLayout, Theme};
pub use typography::{
    FONT_DISPLAY, FONT_MONO, FONT_SANS, FontWeight, LetterSpacing, LineHeight, SnapX,
    TextRendering, TextRenderingParams, TypeToken, crisp_weight_boost_for,
};
