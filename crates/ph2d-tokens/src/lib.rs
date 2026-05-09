#![forbid(unsafe_code)]
//! ph2d-tokens — design tokens (color/type/spacing) per ADR-0023 §12.
//!
//! Source of truth para tudo que tem cor, tamanho ou espaçamento na
//! UI do editor (M12+). Tokens semânticos (não literais): widget code
//! NUNCA usa hex/px direto — sempre via `ColorToken::resolve(theme)`,
//! `TypeToken::px()`, `Spacing::px()`.
//!
//! ### Por quê
//!
//! - **Theme switch (Dark ↔ Light)** vira lookup, não fork de código.
//! - **WCAG 2.2 AA contrast** é validado em tests deste crate (não
//!   pulverizado). Mudou um valor → o teste de contraste falha
//!   imediatamente.
//! - **Lint** futuro (`ph2d-clippy::no-literal-color`) falha em
//!   qualquer `Color::from_hex(...)` fora deste crate.
//!
//! ### Não inclui
//!
//! - Rendering (Vello, raster) — vive em `ph2d-editor` / `ph2d-vector`
//! - Theme persistence — vive em `ph2d-editor` settings
//! - Animation tokens (durations, easings) — adicionar quando primeiro
//!   widget pedir; YAGNI por enquanto

pub mod color;
pub mod spacing;
pub mod theme;
pub mod typography;

pub use color::{Color, ColorToken};
pub use spacing::Spacing;
pub use theme::Theme;
pub use typography::TypeToken;
