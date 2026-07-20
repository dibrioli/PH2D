//! `TextureLayer` — the generator state of a [`super::LayerKind::Texture`] layer.
//!
//! A Texture layer fills the whole sprite with one of the procedural textures the brush engine uses
//! (`ph2d-painter-brush`), recoloured by a [`ColorRamp`]. It is **raster-backed**: the tool renders
//! these fields into a canvas-sized RGBA8 buffer in `images[id]` whenever they change, and the
//! compositor blends that buffer exactly like a raster layer — so a Texture layer composes with
//! masks / clipping / groups / blend / opacity for free, on both the CPU and GPU paths. This struct
//! holds only the cheap-to-clone *spec* (no pixels); the pixels live in the tool's `images` map.

use super::{HARD_CAP_LAYERS, Layer, LayerId, LayerKind, LayerStack, MaskLayer};
use ph2d_color::ColorRamp;
use ph2d_painter_brush::MAX_TEX_PARAMS;
use serde::{Deserialize, Serialize};

/// Generator spec of a Texture layer — all serde-safe plain data so the layer stack still
/// round-trips. The geometric texture fields mirror the brush's `TextureSettings` (kind + per-pattern
/// params + scale + offset); the [`ColorRamp`] recolours the texture's scalar, with `ramp_alpha_mode`
/// deciding whether the ramp's alpha makes texels transparent. Per-dab concepts (mapping / rotation /
/// rake / random) are omitted — a full-cover layer maps at identity.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TextureLayer {
    /// Texture kind wire discriminant ([`ph2d_painter_brush::TextureKind::to_u8`]).
    pub kind: u8,
    /// Per-pattern parameter slots, normalized `[0, 1]` (slots 0/1 = Contrast/Brightness, 2.. = the
    /// kind's shape knobs — see `ph2d_painter_brush::param_specs`).
    pub params: [f32; MAX_TEX_PARAMS],
    /// Per-axis scale (`[TEX_SIZE_MIN, TEX_SIZE_MAX]`; `1.0` ≈ the default tiling).
    pub size: [f32; 2],
    /// Per-axis offset in tile fractions (`[TEX_OFFSET_MIN, TEX_OFFSET_MAX]`).
    pub offset: [f32; 2],
    /// Whether the Color Ramp recolours the texture (off → grayscale fill).
    pub ramp_enabled: bool,
    /// The recolour gradient (linear RGBA stops + interpolation space/mode).
    pub ramp: ColorRamp,
    /// What the ramp colour's alpha does ([`ph2d_painter_brush::RampAlphaMode::to_u8`]): `0` opaque
    /// recolour · `1`/`2` carry the stop alpha into the layer (translucent texels).
    pub ramp_alpha_mode: u8,
}

impl Default for TextureLayer {
    /// A soft, immediately-visible default: billowy **Clouds** recoloured by the ramp's default
    /// black→white gradient (so the layer reads as a neutral grayscale cloud and any ramp edit colours
    /// it at once), opaque.
    fn default() -> Self {
        let kind = ph2d_painter_brush::TextureKind::Clouds;
        let mut params = [0.5f32; MAX_TEX_PARAMS];
        for (i, s) in ph2d_painter_brush::param_specs(kind).iter().enumerate() {
            params[i] = s.default;
        }
        Self {
            kind: kind.to_u8(),
            params,
            size: [1.0, 1.0],
            offset: [0.0, 0.0],
            ramp_enabled: true,
            ramp: ColorRamp::default(),
            ramp_alpha_mode: ph2d_painter_brush::RampAlphaMode::None.to_u8(),
        }
    }
}

impl LayerStack {
    /// Add a Texture layer (procedural fill recoloured by a ramp) at the top of the root stack and
    /// make it active. The pixels are rendered by the tool into `images[id]` (or `canvas_rgba` while
    /// active); this only inserts the spec into the model. Returns `None` at the hard cap.
    pub fn add_texture(&mut self, spec: TextureLayer) -> Option<LayerId> {
        if self.len() >= HARD_CAP_LAYERS {
            return None;
        }
        let n = self.all_ids().filter(|&i| self.is_texture(i)).count() + 1;
        let id = self.alloc_id();
        self.arena.push(Layer::new(
            id,
            format!("Texture {n}"),
            LayerKind::Texture(spec),
        ));
        self.root.insert(0, id);
        self.active = Some(id);
        Some(id)
    }

    /// `true` when `id` is a Texture layer.
    #[must_use]
    pub fn is_texture(&self, id: LayerId) -> bool {
        self.get(id).is_some_and(super::Layer::is_texture)
    }

    /// Mutable access to a Texture layer's spec (for the UI to edit its kind / params / size / offset /
    /// ramp). `None` if `id` is not a Texture layer.
    pub fn texture_mut(&mut self, id: LayerId) -> Option<&mut TextureLayer> {
        match &mut self.get_mut(id)?.kind {
            LayerKind::Texture(t) => Some(t),
            _ => None,
        }
    }

    /// Create a grayscale mask (§2.7) on a raster **or texture** `parent`, with explicit canvas dims
    /// (a Texture layer is full-cover and carries no dims in its spec, so the tool passes its
    /// `source_size`). Owner-attached (not in the z-order). Rejected (`None`) if `parent` is unknown,
    /// isn't a raster/texture, already has a mask, or the hard cap is reached. The generalization of
    /// the raster-only [`LayerStack::add_mask`] (kept for the existing raster tests).
    pub fn add_mask_for(&mut self, parent: LayerId, width: u32, height: u32) -> Option<LayerId> {
        let eligible = matches!(
            self.get(parent),
            Some(Layer {
                kind: LayerKind::Raster(_) | LayerKind::Texture(_),
                mask: None,
                ..
            })
        );
        if !eligible || self.len() >= HARD_CAP_LAYERS {
            return None;
        }
        let id = self.alloc_id();
        self.arena.push(Layer::new(
            id,
            "Mask",
            LayerKind::Mask(MaskLayer {
                width,
                height,
                inverted: false,
            }),
        ));
        if let Some(p) = self.get_mut(parent) {
            p.mask = Some(id);
        }
        Some(id)
    }
}
