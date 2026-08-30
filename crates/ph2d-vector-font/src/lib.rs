#![forbid(unsafe_code)]
//! `ph2d-vector-font` — variable-font glyphs as native vector networks
//! (Inovação #6, [ADR-0066](../../../docs/architecture/decisions/0066-variable-font-glyph-as-vector-network.md)).
//!
//! The differentiator: a glyph is **not** a rasterized texture nor an opaque
//! path — it is a [`VectorNetwork`](ph2d_vector_doc::VectorNetwork), the same
//! data model every other vector tool edits, with the font's design **axes**
//! (`weight`, `width`, `slant`, …) exposed as graph parameters the animation
//! system drives (ADR-0066 §2.4). Typography that *is* animatable vector.
//!
//! ## Module map
//!
//! | Module                | Role                                                         |
//! |-----------------------|--------------------------------------------------------------|
//! | [`axis`]              | OT axis tag + [`VariableFontAxis`] trait + [`FontAxis`]       |
//! | [`glyph_to_network`]  | outline (pen commands) → `VectorNetwork` (Region per contour) |
//! | [`axis_animation`]    | axes as [`AttributeEvaluator`](ph2d_vector_traits::AttributeEvaluator) inputs |
//! | [`fallback_chain`]    | HR-15 locale-aware font fallback                              |
//! | [`skrifa_bridge`]     | skrifa font → axis-aware [`GlyphOutline`] (the only skrifa-touching module) |
//!
//! ## Isolation
//!
//! A new drop-crate (ADR-0075) reading only frozen contracts (`ph2d-vector-doc`,
//! `ph2d-vector-traits`). A glyph emits a standard `VectorNetwork`, so the
//! existing vector renderer draws it with **no** new wiring; the skrifa→Vello
//! *direct* fast path (ADR-0066 §2.3) is a renderer-side optimization (Coord).

pub mod axis;
pub mod axis_animation;
pub mod fallback_chain;
pub mod glyph_to_network;
pub mod skrifa_bridge;

use std::collections::BTreeMap;

use smallvec::SmallVec;

use ph2d_vector_doc::VectorNetwork;

pub use axis::{AxisOutOfRangeError, AxisTag, FontAxis, VariableFontAxis};
pub use axis_animation::VariableFontAxisCurve;
pub use fallback_chain::{FontFamily, Locale, PlatformHost, resolve_glyph_font};
pub use glyph_to_network::{GlyphOutline, PathCommand, outline_to_network, outline_to_network_em};
pub use skrifa_bridge::{FontFaceError, VariableFont};

/// Max axes carried inline before spilling to the heap (ADR-0066 §2.7 cap).
///
/// ⚠️⚠️ **ISTO NÃO É UM TECTO DE EIXOS — é uma decisão de ALOCAÇÃO**, e a distinção foi paga em
/// 2026-08-30: uma fonte com 12 eixos passa por aqui com **os 12**, o `SmallVec` apenas deixa de
/// ser inline. ⛔ Quem procurar «o número que corta os eixos» e encontrar este vai mudar o número
/// errado — o corte real está no REGISTO de ids de quem os pinta
/// (`ph2d_editor_core::ids::MAX_TEXT_VARIATION_AXES`), e há gate a afirmar a não-truncagem aqui
/// ([`skrifa_bridge::tests::the_inline_capacity_is_not_a_ceiling`]).
///
/// # A medição (2026-08-30) — quantos eixos publicam as fontes que este app alcança
///
/// | corpus | faces | máximo de `fvar.axisCount` |
/// |---|---|---|
/// | a fonte empacotada (`ph2d-text/fonts/InterVariable.ttf`) | 1 | **2** (`wght`, `opsz`) |
/// | as faces instaladas nesta workstation (`fc-list`) | 927 | **2** (3 variáveis ao todo) |
///
/// ⚠️ **E o corpus instalado NÃO é o tecto do produto:** o painel de texto tem *Import Font…*, que
/// aceita qualquer `.ttf`/`.otf` do disco. O `fvar.axisCount` é um `uint16` no formato, e fontes
/// paramétricas correntes (Roboto Flex, Amstelvar, Decovar) publicam mais de dez. *Um tecto
/// derivado do que está instalado mede a máquina de quem mediu, não o que o produto aceita.*
pub const MAX_AXES: usize = 8;

/// A glyph index within a font. Our own newtype (not `skrifa::GlyphId`) so the
/// data model and conversion stay decoupled from skrifa — [`skrifa_bridge`] is
/// the only place the two meet.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct GlyphId(pub u32);

/// A glyph as a native vector network plus its variable-font axis state
/// (ADR-0066 §2.2). Re-solving the network for new axis values is the
/// [`skrifa_bridge`]'s job; this struct is the *result* + the live axis values.
#[derive(Clone, Debug, PartialEq)]
pub struct GlyphVectorNetwork {
    /// The glyph outline as standard vector data (one `NonZero` region per
    /// closed contour).
    pub network: VectorNetwork,
    /// Which glyph this is.
    pub glyph_id: GlyphId,
    /// The axes the source font exposes.
    pub axes: SmallVec<[FontAxis; MAX_AXES]>,
    /// The axis values this network was solved at (`tag → value`). A
    /// [`BTreeMap`] for deterministic iteration (HR-5).
    pub current_axis_values: BTreeMap<AxisTag, f32>,
}

impl GlyphVectorNetwork {
    /// Assemble from a solved network, glyph id, and axes; `current_axis_values`
    /// is captured from each axis's current value.
    pub fn new(
        network: VectorNetwork,
        glyph_id: GlyphId,
        axes: SmallVec<[FontAxis; MAX_AXES]>,
    ) -> Self {
        let current_axis_values = axes
            .iter()
            .map(|a| (a.tag(), a.current()))
            .collect::<BTreeMap<_, _>>();
        Self {
            network,
            glyph_id,
            axes,
            current_axis_values,
        }
    }

    /// The axis with the given tag, if the font exposes it.
    pub fn axis(&self, tag: AxisTag) -> Option<&FontAxis> {
        self.axes.iter().find(|a| a.tag() == tag)
    }
}
