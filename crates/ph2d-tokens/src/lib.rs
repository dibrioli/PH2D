#![forbid(unsafe_code)]
//! ph2d-tokens — design tokens (color/type/spacing/radius/motion/layer)
//! per ADR-0023 §12 + design system canônico em [`docs/design/tokens.json`].
//!
//! ## Pipeline
//!
//! ```text
//! docs/design/tokens.json   ←  source of truth (4 themes, OKLCH)
//!         ↓ (manual sync — codegen futuro via build.rs)
//! crates/ph2d-tokens/src/   ←  Rust mirror (this crate)
//!         ↓
//! ph2d-editor widgets       ←  consume via ColorToken::resolve(theme)
//! ```
//!
//! Source of truth para tudo que tem cor, tamanho, espaçamento, raio,
//! sombra, motion ou z-stack na UI. Tokens semânticos (não literais):
//! widget code NUNCA usa hex/px direto — sempre via
//! `ColorToken::resolve(theme)`, `Spacing::px()`, etc.
//!
//! ### Por quê
//!
//! - **4 themes** (`forge-sdf` default, `paint-studio`, `sunstone`,
//!   `blueprint`) viram lookup, não fork de código.
//! - **WCAG 2.2 AA contrast** validado em tests deste crate (não
//!   pulverizado). Mudou um valor → o teste de contraste falha
//!   imediatamente.
//! - **OKLCH na fonte** — perceptualmente uniforme; conversão pra sRGB
//!   centralizada em `oklch_to_srgb`.
//! - **Lint** futuro (`ph2d-clippy::no-literal-color`) falha em
//!   qualquer `Color::from_hex(...)` fora deste crate.
//!
//! ### Não inclui
//!
//! - Rendering (Vello, raster) — vive em `ph2d-editor` / `ph2d-vector`.
//! - Theme persistence — vive em `ph2d-editor` settings.
//! - Codegen automático do tokens.json → este crate (planejado, ver
//!   "Pipeline" acima — por hora sync é manual).

pub mod color;
pub mod layer;
pub mod motion;
pub mod radius;
pub mod spacing;
pub mod theme;
pub mod typography;

pub use color::{Color, ColorToken, oklch_to_srgb};
pub use layer::Layer;
pub use motion::{Duration, Easing};
pub use radius::Radius;
pub use spacing::{Density, ICON_BTN_SIZE_PX, ROW_H_PX, SECTION_GAP_PX, Spacing};
pub use theme::{PanelLayout, Theme};
pub use typography::{
    FONT_DISPLAY, FONT_MONO, FONT_SANS, FontWeight, LetterSpacing, LineHeight, TypeToken,
};
