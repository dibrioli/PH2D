//! Ramp **LUT baking** for the stamp paths — the colour-owner resolution + the bake of the engine's
//! single colour LUT (`texture_ramp_lut`) and the Shape **tone** LUT (`shape_ramp_lut`). Split from
//! `stamp_cache` for the workspace LOC cap; the stamp/blit paths that consume these live there.
//!
//! There is exactly one colour ramp in the engine at a time (the **owner**): the Shape ramp colourises
//! the silhouette when its B&W filter is off, else the Grain ramp colourises its pattern. With B&W on,
//! the Shape ramp is instead the grayscale **tone** remap of the silhouette (a separate value LUT).

use crate::tool::PainterTool;

/// Which ramp last baked into the engine's single colour LUT (`texture_ramp_lut`) — the Shape ramp
/// (it colourises the silhouette when its B&W filter is off), the Grain ramp, or neither. Tracked so
/// the LUT re-bakes when the **owner** flips even if neither ramp's own `_dirty` is set.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum RampLutOwner {
    None,
    Shape,
    Grain,
}

impl PainterTool {
    /// Which ramp (if any) owns the painted COLOUR — the engine's single colour ramp. The Shape ramp
    /// wins when its B&W filter is OFF (it colourises the silhouette); otherwise the Grain ramp owns
    /// colour when enabled + a Grain texture is active. With B&W on the Shape ramp is the tone remap,
    /// not a colour owner. `grain_active` is `brush.texture.is_active()`.
    pub(super) fn color_ramp_owner(&self, grain_active: bool) -> RampLutOwner {
        if self.paint.shape_color_ramp_enabled && !self.paint.shape_color_ramp_bw {
            RampLutOwner::Shape
        } else if grain_active && self.paint.texture_ramp_enabled {
            RampLutOwner::Grain
        } else {
            RampLutOwner::None
        }
    }

    /// The alpha action of whichever ramp owns the colour (`None` mode when no owner).
    pub(super) fn active_ramp_alpha_mode(
        &self,
        owner: RampLutOwner,
    ) -> ph2d_painter_brush::RampAlphaMode {
        match owner {
            RampLutOwner::Shape => self.paint.shape_color_ramp_alpha_mode,
            RampLutOwner::Grain => self.paint.texture_ramp_alpha_mode,
            RampLutOwner::None => ph2d_painter_brush::RampAlphaMode::None,
        }
    }

    /// (Re)bake the engine's single colour LUT from whichever ramp `owner` selects (Shape or Grain),
    /// applying that owner's **B&W** filter (desaturate to Rec.709 luminance in LINEAR space). `eval`
    /// is linear RGBA, but the dab blends in the layer's straight-sRGB space (matching `brush.color`),
    /// so RGB is converted linear → sRGB; alpha stays straight. Re-bakes when the owner flips or its
    /// ramp changed; a no-op otherwise.
    pub(super) fn ensure_ramp_lut(&mut self, owner: RampLutOwner) {
        let fresh = self.paint.texture_ramp_dirty
            || self.paint.ramp_lut_owner != owner
            || self.paint.texture_ramp_lut.len() != 256;
        if !fresh {
            return;
        }
        let (ramp, bw) = match owner {
            RampLutOwner::Shape => (&self.paint.shape_color_ramp, false),
            // Grain (or None — the Grain ramp is a harmless default when nothing owns colour).
            _ => (&self.paint.texture_ramp, self.paint.texture_ramp_bw),
        };
        let mut lut = vec![[0.0f32; 4]; 256];
        ramp.bake_into(&mut lut);
        for c in &mut lut {
            if bw {
                let l = 0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2];
                c[0] = l;
                c[1] = l;
                c[2] = l;
            }
            c[0] = f32::from(ph2d_color::srgb::linear_to_srgb_byte(c[0])) / 255.0;
            c[1] = f32::from(ph2d_color::srgb::linear_to_srgb_byte(c[1])) / 255.0;
            c[2] = f32::from(ph2d_color::srgb::linear_to_srgb_byte(c[2])) / 255.0;
        }
        self.paint.texture_ramp_lut = lut;
        self.paint.texture_ramp_dirty = false;
        self.paint.ramp_lut_owner = owner;
    }

    /// Bake the Shape **tone** LUT (256 grayscale entries) from the `shape_color_ramp`'s Rec.709
    /// luminance (LINEAR — it remaps the silhouette's coverage value, no gamma). The B&W filter's paint
    /// role: the ramp's tone, ignoring its chroma. A no-op when clean.
    pub(super) fn ensure_shape_ramp_lut(&mut self) {
        if !self.paint.shape_ramp_dirty && self.paint.shape_ramp_lut.len() == 256 {
            return;
        }
        let mut clut = [[0.0f32; 4]; 256];
        self.paint.shape_color_ramp.bake_into(&mut clut);
        let mut lut = vec![0.0f32; 256];
        for (o, c) in lut.iter_mut().zip(clut.iter()) {
            *o = (0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2]).clamp(0.0, 1.0);
        }
        self.paint.shape_ramp_lut = lut;
        self.paint.shape_ramp_dirty = false;
    }

    /// The Shape **tone** LUT slice when the Shape ramp is in B&W (tone) mode — then it remaps the
    /// silhouette coverage (the Grain, if any, owns colour). Caller `ensure_shape_ramp_lut`s first.
    pub(super) fn shape_tone_lut_slice(&self) -> Option<&[f32]> {
        (self.paint.shape_color_ramp_enabled && self.paint.shape_color_ramp_bw)
            .then_some(self.paint.shape_ramp_lut.as_slice())
    }
}
