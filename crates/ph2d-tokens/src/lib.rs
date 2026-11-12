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
//! - **Authoring** the tokens está AQUI desde a W6: a camada de override ([`overrides`],
//!   [`num_overrides`]) dá alias e math sobre a tabela gerada, e o [`route`] é a porta por onde
//!   uma tabela vinda de FORA encontra o token dela.
//!
//!   ⚠️ Ficam de fora, cada um com o seu motivo: **os modos são os quatro variants do
//!   [`Theme`]** (o artista não cria um quinto), e o **codec DTCG** vive na `ph2d-tokens-dtcg`,
//!   porque esta crate é a folha de que 44 widgets dependem e um parser de JSON é uma aresta que
//!   ela não paga. Ver `docs/Vector Module/Estudos/PLANO_UI_UX_padrao_figma.md` §4/W4.

mod alias_walk;
mod camadas_de_colisao;
/// ⭐ A paleta categórica (`categorical-01..12`): uma cor por categoria, pelo índice.
pub mod categorical;
mod changed;
pub mod chrome;
pub mod color;
mod color_alias;
mod color_space;
mod color_value;
/// **A camada de OVERRIDE de cor** (plano UI/UX W6) — o que o artista autora sobre a tabela
/// gerada. Vazia, `ColorToken::resolve` é byte-idêntico ao de sempre.
pub mod contrast;
/// **A camada de OVERRIDE numérica** — a irmã da [`overrides`], no molde dela.
/// ⭐ **A derivação de um tema moderno** — cinco entradas → todos os slots (regras do Godot 4.6, MIT).
pub mod derive;
mod faint_row;
/// O Kanban (TrelloClone): as etiquetas por `[tom][matiz]` e a tinta de cada tom.
pub mod kanban;
pub mod layer;
pub mod motion;
/// **A identidade de um token NUMÉRICO** (plano UI/UX W4c.1) — a família que se mede em px.
pub mod num;
pub mod num_expr;
pub mod num_overrides;
pub mod num_runtime;
pub mod overrides;
pub mod radius;
/// **A chave decide a família** — a porta única por onde uma tabela de FORA (o arquivo de projeto,
/// um `.tokens.json` DTCG) encontra o token a que pertence.
pub mod route;
mod row_hover;
/// ⭐ **O corpo de uma pele vestida** (F18) — o tema pelo PAPEL de cada token sobre o desenho.
pub mod skin_body;
pub mod slider_style;
pub mod spacing;
mod state_layer;
pub mod stroke;
pub mod text_style;
pub mod theme;
pub mod timeline_tinta;
pub mod type_colour;
pub mod typography;
pub mod ui_scale;
/// ⭐ **A tabela de estados do widget** — a porta única de fundo / borda / raio por estado.
pub mod visuals;

/// Auto-generated tables from `docs/design/tokens.json` via `build.rs`.
/// See Wave 2 PR 11.1. Consumed by `color.rs::ColorToken::resolve`.
/// Module visibility: `pub(crate)` — runtime is the public API.
pub(crate) mod generated {
    include!(concat!(env!("OUT_DIR"), "/tokens_generated.rs"));
}

pub use camadas_de_colisao::{LAYER_OUTLINE_OFF_ALPHA, OUTLINE_SHADOW_ALPHA};
pub use categorical::{CATEGORICAL, categorical};
pub use changed::{changed_band, changed_ink, value_ink};
pub use chrome::{
    CHECKBOX_BOX_PX, DIVIDER_GAP_PX, DRAG_GHOST_ALPHA, DRAG_GHOST_SCALE, EDGE_PAD_PX,
    HERO_VIEWPORT_H_PX, HERO_VIEWPORT_W_PX, HIERARCHY_W_PX, HUD_BOTTOM_PAD_PX, HUD_H_PX,
    INSPECTOR_W_PX, PANEL_HEAD_PAD_PX, PANEL_MIN_W_PX, PANEL_RADIUS_PX,
    PANEL_RESIZE_HANDLE_SIZE_PX, PILL_PADDING_PX, ROW_SELECTED_ALPHA, TIMELINE_ALCA_GUIA_ALPHA,
    TIMELINE_ALCA_HALO_ALPHA, TIMELINE_ALCA_HOVER_MIX, TIMELINE_FAIXA_H_PX, TIMELINE_FRAME_PX,
    TIMELINE_KEY_HALF_PX, TIMELINE_KEY_HIT_HALF_PX, TIMELINE_KEY_RING_PX, TIMELINE_LISTA_BARRA_PX,
    TIMELINE_LOSANGO_LIVRE_ALPHA, TIMELINE_MISTURA_ALPHA, TIMELINE_MISTURA_BORDA_ALPHA,
    TIMELINE_OBJECT_DOT_HALF_PX, TIMELINE_PECA_BORDA_ALPHA, TIMELINE_PECA_MARGEM_PX,
    TIMELINE_PECA_MUDA_ALPHA, TIMELINE_PECA_PE_ALPHA, TIMELINE_PECA_RAIO_PX,
    TIMELINE_PECA_TOPO_ALPHA, TIMELINE_PESO_RAIO_PX, TIMELINE_SUMMARY_CELL_ALPHA,
    TIMELINE_SUMMARY_TINT_ALPHA, TOOL_CHIP_PX, TOPBAR_GAP_PX, TOPBAR_H_PX,
};
pub use color::{Color, ColorToken, ColorValue, faint_row_bg};
/// ⭐ A aritmética de cor saiu para um irmão (tecto de LOC), e o endereço público não muda.
pub use color_space::{oklch_in_gamut, oklch_to_linear_srgb, oklch_to_srgb, srgb_to_oklch};
pub use layer::Layer;
pub use motion::{Duration, Easing};
pub use num::NumToken;
pub use radius::Radius;
pub use slider_style::{SLIDER_DENSITIES, SLIDER_RADII, SliderDesign, SliderStyle, UiLook};
pub use spacing::{
    Density, ICON_BTN_SIZE_PX, INLINE_ICON_PX, ROW_H_PX, Spacing, area_gap_px, card_gap_px,
    card_pad_px, control_gap_px, icon_label_gap_px, list_indent_px, list_row_gap_px, row_pitch_px,
    section_gap_px, tree_chevron_col_px,
};
pub use state_layer::HOVER_LAYER_ALPHA;
pub use stroke::StrokeToken;
pub use text_style::{UiFont, UiTextSize, UiTextStyle, UiWeight};
pub use theme::{PanelLayout, Theme};
pub use timeline_tinta::{PECAS, TimelineTinta};
pub use typography::{
    CODE_STEM_DARKENING_PX, FONT_DISPLAY, FONT_MONO, FONT_SANS, FontWeight, LetterSpacing,
    LineHeight, STEM_DARKENING_PX, STEM_DARKENING_UI_SURFACE, SnapX, TextRendering,
    TextRenderingParams, TypeToken, code_stem_darkening_px, crisp_weight_boost_for,
    stem_darkening_px,
};
pub use ui_scale::UiScale;
