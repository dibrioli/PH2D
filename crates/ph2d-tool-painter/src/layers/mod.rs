//! Layer stack data model (W3.T3.1) — `docs/Painter_projeto/02_layers.md`.
//!
//! Pure data + operations: create / reorder / nest / visibility / opacity /
//! blend-mode / active-selection. The actual pixel buffers (RGBA8 raster,
//! R8 mask) live in the tool's canvas + the GPU `LayerCache`; this model
//! holds only metadata (dimensions + handles + flags), so it stays cheap
//! to clone, diff, serialize, and reason about in tests.
//!
//! # Kinds vs modifiers
//!
//! The design doc (§2.1) lists *Clipping Mask*, *Reference*, and
//! *Alpha-lock* alongside Raster/Group/Mask, but it also states each is a
//! **modifier of a raster layer**, not a standalone bitmap. We model that
//! faithfully: [`LayerKind`] has the three real kinds (Raster, Group,
//! Mask), and the modifiers are boolean flags on [`Layer`]. *Adjustment*
//! layers are W4 (§7) and intentionally absent here.
//!
//! # Z-order
//!
//! [`LayerStack::root`] and [`GroupLayer::children`] are ordered
//! **top-to-bottom** (index 0 = topmost, matching the layer panel). The
//! compositor walks them in reverse (bottom-up) per §2.11.

use ph2d_painter_effects::BlendMode;
use ph2d_painter_effects::adjustments::AdjustmentLayer;
use serde::{Deserialize, Serialize};

pub use texture::TextureLayer;

/// Maximum group nesting depth (§2.6). A would-be level-9 group folds to
/// level 8 (the deeper insert is rejected).
pub const MAX_GROUP_DEPTH: usize = 8;

/// Hard cap on total layers per canvas (§2.5), mirrors Procreate. The
/// dynamic budget (`f(dimensions, format, MemoryBudget)`) clamps below
/// this; the stack itself only enforces the hard ceiling.
pub const HARD_CAP_LAYERS: usize = 999;

/// A layer's [`Layer::impasto_depth`] that changes nothing: the relief composites exactly as sculpted.
/// The neutral element of the whole feature — every layer is born here, so a document that never
/// touches Impasto composites byte-identically to one built before it existed.
pub const DEPTH_NEUTRAL: f32 = 1.0;

/// Stable per-canvas layer identity. Allocated monotonically by
/// [`LayerStack`]; never reused within a stack's lifetime so stale handles
/// (undo, cache keys) resolve unambiguously.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct LayerId(pub u64);

/// Bitmap raster layer (RGBA8 / RGBA16F per canvas profile). Pixels live
/// in the tool canvas + GPU cache; the model holds dimensions only.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RasterLayer {
    pub width: u32,
    pub height: u32,
}

/// Grayscale (R8) mask bound to a parent raster layer (§2.7). White =
/// visible, black = hidden; multiplies the parent's alpha.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MaskLayer {
    pub width: u32,
    pub height: u32,
    /// `Invert mask` toggle — composite uses `1 - value` when set.
    pub inverted: bool,
}

/// Container grouping N child layers (§2.1). Applies its blend-mode +
/// opacity to the composited child stack.
#[derive(Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct GroupLayer {
    /// Child ids, top-to-bottom (same convention as [`LayerStack::root`]).
    pub children: Vec<LayerId>,
    pub collapsed: bool,
}

/// The real layer kinds. Modifiers (clip/reference/alpha-lock) are flags on
/// [`Layer`], not kinds (see module docs). W4 (ADR-0045 + amendment-1): the
/// non-destructive `Adjustment` kind carries an [`AdjustmentLayer`] payload
/// whose inner fields (opacity/blend/mask/…) are authoritative over the outer
/// [`Layer`]'s for an adjustment node.
///
/// `Eq` was dropped when `Adjustment` landed — `AdjustmentLayer` holds `f32`
/// params, so the enum is `PartialEq` only (it was never used as a map key).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum LayerKind {
    Raster(RasterLayer),
    Mask(MaskLayer),
    Group(GroupLayer),
    Adjustment(AdjustmentLayer),
    /// A procedural-texture fill (the brush textures) recoloured by a Color Ramp, covering the whole
    /// sprite. Raster-backed: its pixels are rendered into the tool's `images[id]` from the
    /// [`TextureLayer`] spec, so the compositor blends it exactly like a raster (mask / clip / blend /
    /// opacity / groups all apply). Real-time editable from the layers panel.
    Texture(TextureLayer),
}

/// How a layer's **relief** enters the composed height field the Impasto light reads
/// (`docs/Painter/16_impasto_plano_implementacao.md` §6 — "Composite Depth por camada").
///
/// The default is [`Self::Add`], and it is the physical one: paint laid over paint is *more material*,
/// so the heights sum. [`Self::Level`] is the painter's override — thick opaque paint that **buries**
/// the texture under it instead of inheriting it, weighted by its own coverage. It is the one behaviour
/// a depth *scale* cannot express, which is the only reason it is a mode and not a number: `Add` with a
/// negative scale already carves, and `Add` with scale `0` already ignores.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReliefComposite {
    /// The layer's relief is ADDED to what is under it (paint piles up). The default.
    #[default]
    Add,
    /// The layer's relief REPLACES what is under it, in proportion to its own paint coverage: where
    /// this layer's paint is solid, the surface is *this* layer's; where it is bare, the layers below
    /// show through untouched. (A `Level` layer with no relief at all still buries — that is what makes
    /// it a scrape, and why the panel only offers it on layers that carry relief.)
    Level,
}

/// A single layer: identity + kind + composite params + modifier flags.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Layer {
    pub id: LayerId,
    pub name: String,
    pub kind: LayerKind,
    pub blend_mode: BlendMode,
    /// Layer opacity in `[0, 1]`.
    pub opacity: f32,
    pub visible: bool,
    /// `Lock` — blocks edits (paint/transform) but still composites.
    pub locked: bool,
    /// Alpha-lock modifier (§2.10) — paint restricted to existing alpha.
    pub alpha_locked: bool,
    /// Clipping-mask modifier (§2.8) — clips to the layer directly below.
    pub clipping: bool,
    /// Reference-layer modifier (§2.9) — geometry source for ColorDrop.
    pub is_reference: bool,
    /// Optional grayscale mask child (§2.7).
    pub mask: Option<LayerId>,
    /// **Impasto depth** of this layer's relief in `[-1, 1]` — the opacity of thickness.
    ///
    /// `1` (the default) composites the relief as it was sculpted; `0` mutes it without destroying it;
    /// negative INVERTS it (the ridges become grooves). Unlike the brush's Depth — which is baked into
    /// each stroke as it lands and afterwards only reaches the *last* one — this is a live handle on
    /// **everything ever sculpted on the layer**, and it is the only one. Sibling of [`Self::opacity`]
    /// in every sense: a composite parameter, non-destructive, and edited from the same row.
    pub impasto_depth: f32,
    /// How this layer's relief meets the relief BELOW it. See [`ReliefComposite`].
    pub impasto_composite: ReliefComposite,
    /// Whether this layer carries any relief at all — a **projection** of the tool's height map
    /// (`PainterTool::heights`), not an authority.
    ///
    /// The panel needs it and cannot derive it: the pixels (and the relief) live in the tool, while the
    /// panel only ever sees a clone of this stack. It is what decides whether the Depth row is painted
    /// on a row at all, so a document nobody has sculpted shows no impasto chrome anywhere. The tool is
    /// the single writer (`PainterTool::sync_relief_flags`, called wherever `heights` changes), and a
    /// gate pins the invariant — a stale flag here means a knob that does nothing, the exact species
    /// this line has already exterminated twice.
    pub has_relief: bool,
}

impl Layer {
    fn new(id: LayerId, name: impl Into<String>, kind: LayerKind) -> Self {
        Self {
            id,
            name: name.into(),
            kind,
            blend_mode: BlendMode::Normal,
            opacity: 1.0,
            visible: true,
            locked: false,
            alpha_locked: false,
            clipping: false,
            is_reference: false,
            mask: None,
            impasto_depth: DEPTH_NEUTRAL,
            impasto_composite: ReliefComposite::Add,
            has_relief: false,
        }
    }

    #[must_use]
    pub fn is_group(&self) -> bool {
        matches!(self.kind, LayerKind::Group(_))
    }

    #[must_use]
    pub fn is_texture(&self) -> bool {
        matches!(self.kind, LayerKind::Texture(_))
    }
}

/// Modifier flags of the active layer — the layers panel's modifier toolbar
/// (Mask / Clip / Lock / Ref) paints its toggle state from this. `Copy`.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct LayerModifiers {
    /// Only a raster can take a mask / be clipped / alpha-locked.
    pub is_raster: bool,
    pub has_mask: bool,
    pub clipping: bool,
    pub alpha_locked: bool,
    pub is_reference: bool,
}

/// The layer stack for one canvas. A flat arena (`arena`) keyed by
/// [`LayerId`], plus the top-level z-order (`root`); groups reference
/// their children by id. This keeps reorder/nest cheap and lets the
/// compositor walk the tree recursively without moving pixel data.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LayerStack {
    arena: Vec<Layer>,
    root: Vec<LayerId>,
    active: Option<LayerId>,
    next_id: u64,
}

impl Default for LayerStack {
    fn default() -> Self {
        Self::new()
    }
}

// ── Submodules (god-object split, 2026-06-04; pure move) ──
mod stack;
#[cfg(test)]
mod tests;
mod texture;
