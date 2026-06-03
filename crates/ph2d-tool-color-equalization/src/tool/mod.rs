//! [`ColorEqualizationTool`] — stateful editor Tool for Color EQ.
//!
//! Model: `ColorEqualizationParams` (CLAHE clip / tile grid + tonal
//! adjusts + auto-WB toggle) + a cached source snapshot + a live preview
//! thumbnail (512² aspect-fit, mirroring the BgRemoval cap). The Tool
//! reacts only to its panel widgets, never to the canvas directly.
//!
//! ## Apply flow
//!
//! On every panel edit, [`ColorEqualizationTool::apply_ui_edit`] mutates
//! `params` and (when `params_dirty`) rebuilds the preview against the
//! cached small source. On Apply, [`ColorEqualizationTool::apply_ui_edit`]
//! sets `pending_apply`. The shell drains it via
//! [`ColorEqualizationTool::take_pending_apply`] each frame; on `true` it
//! reads each selected sprite's live `Sprite.source`, runs
//! [`ColorEqualizationTool::run_full_resolution`] against that pixel
//! buffer, and swaps the texture (the BgRemoval / Padding precedent).
//! Multi-sprite broadcast is handled shell-side — the shell emits one
//! `OneShotImageOp` per `hero.gizmo.iter_selected()`.
//!
//! `RasterEditTool` (renamed from `ImageEditTool` in ADR-0041) is
//! intentionally NOT implemented yet (DIRETRIZ §3.8.3.1 — production
//! tools currently use the `as_any_mut` downcast pattern; the generic
//! raster-edit channel is fan-out Etapa 2 work in the Wave 10 perfection
//! plan).
//!
//! ## Module layout
//!
//! - `mod.rs` — the [`ColorEqualizationTool`] struct + inherent methods
//!   (source snapshot, preview, drain flags, full-res bake).
//! - [`traits`] — `impl Tool` (panel event routing) + `impl RasterEditTool`
//!   (generic raster I/O lifecycle, ADR-0041).

mod traits;

use super::algorithm::{
    HistogramData, aspect_fit_within, compute_histogram, resize_bilinear_rgba, run_pipeline,
};
use super::params::{
    ColorEqualizationParams, ColorEqualizationUiEdit, ColorEqualizationUiSnapshot, apply_ui_edit,
    brightness_to_slider, clip_limit_to_slider, contrast_to_slider, exposure_to_slider,
    lut_intensity_to_slider, lut_mix_to_slider, posterize_dither_grain_to_slider,
    posterize_dither_strength_to_slider, saturation_to_slider, sharpen_amount_to_slider,
    sharpen_radius_to_slider, temperature_to_slider, tile_grid_to_slider, tint_to_slider,
    vibrance_to_slider,
};

/// Side cap (px) for the live preview overlay. The preview re-runs the
/// full CLAHE + adjusts pipeline on every parameter change; doing that
/// at full source resolution makes each slider tick janky. The preview
/// is drawn scaled to the sprite footprint anyway, so it re-processes a
/// copy of the source downscaled to fit this box (aspect preserved, no
/// letterbox) instead, keeping slider drags smooth. Apply still bakes
/// at full source resolution via
/// [`ColorEqualizationTool::run_full_resolution`].
pub const PREVIEW_MAX_DIM: u32 = 512;

/// Editor Tool implementing the Color Equalization feature.
#[derive(Clone, Debug, Default)]
pub struct ColorEqualizationTool {
    /// User-tunable parameters, projected into the floating panel.
    pub params: ColorEqualizationParams,

    /// Latest source snapshot pushed by the host (set via
    /// [`Self::set_source_snapshot`]). Empty until the host calls.
    /// Layout: RGBA8, length `source_w * source_h * 4` (straight alpha).
    source_rgba: Vec<u8>,
    source_w: u32,
    source_h: u32,

    /// Source downscaled to fit [`PREVIEW_MAX_DIM`] (aspect preserved, no
    /// letterbox) — the input of the on-canvas live preview. Rebuilt only
    /// when the source snapshot changes, so a slider drag re-processes
    /// this small image instead of the full-res source.
    preview_src_rgba: Vec<u8>,
    preview_src_w: u32,
    preview_src_h: u32,

    /// Preview output — result of `run_pipeline` on `preview_src_rgba` with
    /// the current `params`. Layout matches `preview_src_*` dims. Reused
    /// across runs (HR-3 — the allocation persists).
    preview_rgba: Vec<u8>,
    preview_dirty: bool,

    /// Set to `true` when the user presses Apply on the panel. The host
    /// drains it via [`Self::take_pending_apply`] and bakes at full
    /// resolution against every selected sprite.
    pending_apply: bool,

    /// Set to `true` by any panel-edit mutator (param change OR
    /// Apply trigger). The shell uses this as the gate for rerunning the
    /// on-canvas live preview — mirrors `BgRemovalTool::take_params_dirty`.
    params_dirty: bool,

    /// `Some(slot)` (1 or 2) when the user just picked an option from the
    /// LUT slot's grouped dropdown — the panel paint reads this and
    /// force-closes that dropdown's open state on the next frame.
    /// Without it, the popover would stay open until the user clicked
    /// the chip again. Drained via [`Self::take_pending_close_lut_dropdown`].
    pending_close_lut_dropdown: Option<u8>,

    /// `true` when the params were just reset to defaults (Reset button
    /// OR `on_activate`). The shell bridge drains this and re-runs
    /// `Panel::populate(store)` so the slider knob / chip text
    /// positions snap back to defaults too — without this, only the
    /// params struct resets while the WidgetStore retains whatever
    /// drag position the user last left (knobs stay where they were,
    /// but pipeline runs against defaults — a confusing desync).
    pending_panel_reset: bool,
}

impl ColorEqualizationTool {
    /// Push a fresh source RGBA snapshot from the host (selection changed
    /// or tool became active). Rebuilds the capped preview source and
    /// marks the preview dirty so the next per-frame paint re-runs the
    /// pipeline against it.
    ///
    /// `pixels` must be straight-alpha `SrgbRgba` of length `w * h`.
    /// Internally stored as `Vec<u8>` (downstream consumes bytes);
    /// cast via `bytemuck::cast_vec` is zero-copy.
    pub fn set_source_snapshot(&mut self, pixels: Vec<ph2d_color::SrgbRgba>, w: u32, h: u32) {
        assert_eq!(pixels.len(), (w as usize) * (h as usize));
        self.source_rgba = bytemuck::allocation::cast_vec(pixels);
        self.source_w = w;
        self.source_h = h;
        self.rebuild_preview_src();
        self.preview_dirty = true;
        // Cached on-canvas preview the shell holds was computed against
        // the previous selection — mark dirty so the next bridge tick
        // rebuilds it.
        self.params_dirty = true;
    }

    /// Whether the host has pushed a source snapshot at least once.
    pub fn has_source(&self) -> bool {
        !self.source_rgba.is_empty()
    }

    /// Source texture resolution `(w, h)` of the active snapshot, or
    /// `(0, 0)` before any source is pushed.
    pub fn source_size(&self) -> (u32, u32) {
        (self.source_w, self.source_h)
    }

    /// Drain the pending-apply flag. Returns `true` exactly once after
    /// each Apply trigger; host calls this in its per-frame drain loop;
    /// on `true` it runs [`Self::run_full_resolution`] against each
    /// selected sprite.
    pub fn take_pending_apply(&mut self) -> bool {
        let p = self.pending_apply;
        self.pending_apply = false;
        p
    }

    /// Drain the params-dirty flag. Returns `true` exactly once when any
    /// panel-edit mutator has run since the last call.
    pub fn take_params_dirty(&mut self) -> bool {
        std::mem::take(&mut self.params_dirty)
    }

    /// Drain the "close this LUT dropdown" request. Returns `Some(slot)`
    /// (1 or 2) exactly once after the user picks an option from the
    /// grouped dropdown; the panel calls this each paint and force-flips
    /// the matching `InteractiveState::Dropdown.open` to `false`.
    pub fn take_pending_close_lut_dropdown(&mut self) -> Option<u8> {
        self.pending_close_lut_dropdown.take()
    }

    /// Drain the "panel store needs repopulation" request. Returns
    /// `true` exactly once after Reset or `on_activate` fired
    /// `apply_ui_edit::ResetAll`; the shell bridge then calls
    /// `Panel::populate(store)` to snap every slider knob + chip
    /// text back to the default-constructed state. Mirrors the other
    /// `take_pending_*` drains.
    pub fn take_pending_panel_reset(&mut self) -> bool {
        std::mem::take(&mut self.pending_panel_reset)
    }

    /// Project the live params into the snapshot the typed
    /// `ph2d-panel-color-equalization` paints. Host publishes a fresh
    /// snapshot once per frame while the tool is active.
    pub fn ui_snapshot(&self) -> ColorEqualizationUiSnapshot {
        ColorEqualizationUiSnapshot {
            clip_limit01: clip_limit_to_slider(self.params.clip_limit),
            tile_grid01: tile_grid_to_slider(self.params.tile_grid_size),
            exposure01: exposure_to_slider(self.params.exposure),
            temperature01: temperature_to_slider(self.params.temperature),
            tint01: tint_to_slider(self.params.tint),
            brightness01: brightness_to_slider(self.params.brightness),
            contrast01: contrast_to_slider(self.params.contrast),
            vibrance01: vibrance_to_slider(self.params.vibrance),
            saturation01: saturation_to_slider(self.params.saturation),
            sharpen_amount01: sharpen_amount_to_slider(self.params.sharpen_amount),
            sharpen_radius01: sharpen_radius_to_slider(self.params.sharpen_radius),
            lut_intensity01: lut_intensity_to_slider(self.params.lut_intensity),
            lut_mix01: lut_mix_to_slider(self.params.lut_mix),
            auto_levels: self.params.auto_levels,
            auto_contrast: self.params.auto_contrast,
            auto_colors: self.params.auto_colors,
            auto_wb: self.params.auto_wb,
            lut_preset_1: self.params.lut_preset_1,
            lut_preset_2: self.params.lut_preset_2,
            posterize_levels: self.params.posterize_levels,
            posterize_dithering: self.params.posterize_dithering,
            posterize_dither_strength01: posterize_dither_strength_to_slider(
                self.params.posterize_dither_strength,
            ),
            posterize_dither_strength: self.params.posterize_dither_strength,
            posterize_dither_grain01: posterize_dither_grain_to_slider(
                self.params.posterize_dither_grain,
            ),
            posterize_dither_grain: self.params.posterize_dither_grain,
            quantize_colors: self.params.quantize_colors,
            clip_limit: self.params.clip_limit,
            tile_grid_size: self.params.tile_grid_size,
            exposure: self.params.exposure,
            temperature: self.params.temperature,
            tint: self.params.tint,
            brightness: self.params.brightness,
            contrast: self.params.contrast,
            vibrance: self.params.vibrance,
            saturation: self.params.saturation,
            sharpen_amount: self.params.sharpen_amount,
            sharpen_radius: self.params.sharpen_radius,
            lut_intensity: self.params.lut_intensity,
            lut_mix: self.params.lut_mix,
        }
    }

    /// Compute the histogram of the current preview (Phase 2). Pulls
    /// `preview_rgba()` first so the histogram reflects the active
    /// params; returns the default (empty) histogram when no source is
    /// loaded yet. The shell is expected to publish this once per frame
    /// to `ph2d_panel_color_equalization::set_current_histogram` so the
    /// panel overlay paints up-to-date bins.
    pub fn preview_histogram(&mut self) -> HistogramData {
        let (rgba, _, _) = self.preview_rgba();
        if rgba.is_empty() {
            return HistogramData::default();
        }
        compute_histogram(bytemuck::cast_slice(rgba))
    }

    /// Apply one panel-originated edit against the live params. Re-runs
    /// the preview when a param actually changed and a source is loaded.
    /// `Apply` arms the pending-apply latch the host drains via
    /// [`Self::take_pending_apply`].
    pub fn apply_ui_edit(&mut self, edit: ColorEqualizationUiEdit) {
        if matches!(edit, ColorEqualizationUiEdit::Apply) {
            self.pending_apply = true;
            self.params_dirty = true;
            return;
        }
        // Reset also stages a panel-store repopulation (shell drains via
        // `take_pending_panel_reset`) so the slider knobs / chip text
        // snap back to defaults — without it, only `params` resets
        // while the WidgetStore keeps the last drag positions.
        if matches!(edit, ColorEqualizationUiEdit::ResetAll) {
            self.pending_panel_reset = true;
        }
        if apply_ui_edit(&mut self.params, edit) && self.has_source() {
            self.preview_dirty = true;
        }
        self.params_dirty = true;
    }

    /// Borrow the current preview RGBA + its dimensions. Returns an empty
    /// slice + `(0, 0)` until a source is pushed. Lazily reruns the
    /// pipeline when `preview_dirty` is set.
    ///
    /// Uses the GPU [`ChainedPipelineCache`](super::gpu::ChainedPipelineCache)
    /// when built with `--features gpu` AND an adapter is available;
    /// falls back to the CPU pipeline when either fails. The GPU
    /// chain at 512² typically costs ~15-30 ms (vs. 150-380 ms CPU
    /// when Quantize is active), reclaiming the budget the
    /// debounce-on-release strategy assumed.
    pub fn preview_rgba(&mut self) -> (&[u8], u32, u32) {
        if self.preview_dirty && self.has_source() {
            self.preview_rgba.clear();
            self.preview_rgba.resize(
                (self.preview_src_w as usize) * (self.preview_src_h as usize) * 4,
                0,
            );
            let ran_on_gpu = self.try_preview_gpu();
            if !ran_on_gpu {
                run_pipeline(
                    bytemuck::cast_slice(&self.preview_src_rgba),
                    self.preview_src_w,
                    self.preview_src_h,
                    &self.preview_scaled_params(),
                    &mut self.preview_rgba,
                );
            }
            self.preview_dirty = false;
        }
        (&self.preview_rgba, self.preview_src_w, self.preview_src_h)
    }

    /// Enio 2026-05-26: dither grain é em pixels absolutos. Preview
    /// roda em resolução reduzida (`preview_src_w` × `preview_src_h`)
    /// — sem escalar, blocos do dither apareciam visualmente MAIORES
    /// no preview que no resultado final (grain=4 num preview 512 é
    /// 1/128 da largura; no source 2048 é 1/512). Aqui escalamos a
    /// grain pela razão preview/source pra paridade visual.
    fn preview_scaled_params(&self) -> crate::params::ColorEqualizationParams {
        let mut p = self.params;
        if self.source_w > 0 && self.preview_src_w > 0 && p.posterize_dither_grain > 1 {
            let ratio = self.preview_src_w as f32 / self.source_w as f32;
            let scaled = ((p.posterize_dither_grain as f32 * ratio).round() as u32).max(1);
            p.posterize_dither_grain = scaled;
        }
        p
    }

    /// Try to run the preview through the GPU chain. Returns `true`
    /// when it ran (and `self.preview_rgba` holds the result), `false`
    /// to signal the caller should run the CPU fallback. Always
    /// returns `false` when the crate is built without `--features
    /// gpu`. With the feature, uses the process-wide cached
    /// [`super::gpu::try_preview_chain`] so the ~10-20 ms pipeline
    /// compile is paid once, not per slider release.
    #[cfg(not(feature = "gpu"))]
    fn try_preview_gpu(&mut self) -> bool {
        false
    }
    #[cfg(feature = "gpu")]
    fn try_preview_gpu(&mut self) -> bool {
        use super::gpu::{try_headless_gpu, try_preview_chain};
        let Some(gpu) = try_headless_gpu() else {
            return false;
        };
        let Some(chain) = try_preview_chain() else {
            return false;
        };
        // `run_chained` consumes its `rgba` in place — seed with the
        // preview source (the chain reads input + writes output to
        // the same buffer via texture upload + readback).
        self.preview_rgba.copy_from_slice(&self.preview_src_rgba);
        let params = self.preview_scaled_params();
        chain.run_chained(
            &gpu,
            &mut self.preview_rgba,
            self.preview_src_w,
            self.preview_src_h,
            &params,
        );
        true
    }

    /// Run the full-resolution pipeline against the cached source and
    /// write the result into `out` (resized to fit). Returns the `(w, h)`
    /// of the output.
    ///
    /// Called by the shell drain on Apply for EACH selected sprite (one
    /// `OneShotImageOp` per sprite). The shell is expected to call
    /// [`Self::set_source_snapshot`] with that sprite's pixels before
    /// invoking this so the bake matches the live source.
    pub fn run_full_resolution(&mut self, out: &mut Vec<u8>) -> (u32, u32) {
        assert!(self.has_source(), "set_source_snapshot must run first");
        run_pipeline(
            bytemuck::cast_slice(&self.source_rgba),
            self.source_w,
            self.source_h,
            &self.params,
            out,
        );
        (self.source_w, self.source_h)
    }

    /// GPU twin of [`Self::run_full_resolution`] — chains every GPU stage
    /// (Phase 1 tonal batch, LUT, sharpen, auto-WB) into one upload +
    /// N dispatches + one readback via
    /// [`super::gpu::ChainedPipelineCache`]. CLAHE + auto-* normalisations
    /// still run CPU-side inside the chain. Returns `None` when no GPU
    /// adapter is available (CI runners, headless without Metal/Vulkan);
    /// callers should fall back to [`Self::run_full_resolution`] in that
    /// case.
    #[cfg(feature = "gpu")]
    pub fn run_full_resolution_gpu(&mut self, out: &mut Vec<u8>) -> Option<(u32, u32)> {
        use super::gpu::{try_headless_gpu, try_preview_chain};
        assert!(self.has_source(), "set_source_snapshot must run first");
        let gpu = try_headless_gpu()?;
        // Cached chain — `try_preview_chain` uses `OnceLock<…>` so the
        // pipeline compile (~200 ms) is paid once per process, not per
        // slider release. Without this, every live-bake tick on a 1024²
        // preview paid the full pipeline recompile before the first
        // dispatch even started; with the cache, slider releases drop
        // the chain-build overhead and only pay the GPU work.
        let chain = try_preview_chain()?;
        let expected = (self.source_w as usize) * (self.source_h as usize) * 4;
        out.clear();
        out.resize(expected, 0);
        out.copy_from_slice(&self.source_rgba);
        chain.run_chained(&gpu, out, self.source_w, self.source_h, &self.params);
        Some((self.source_w, self.source_h))
    }

    /// Build [`Self::preview_src_rgba`] — the source downscaled to fit
    /// [`PREVIEW_MAX_DIM`] (aspect preserved, no letterbox). No-op without
    /// a source.
    fn rebuild_preview_src(&mut self) {
        if !self.has_source() {
            self.preview_src_w = 0;
            self.preview_src_h = 0;
            self.preview_src_rgba.clear();
            return;
        }
        let (dw, dh) = aspect_fit_within(self.source_w, self.source_h, PREVIEW_MAX_DIM);
        if dw == self.source_w && dh == self.source_h {
            self.preview_src_rgba.clear();
            self.preview_src_rgba.extend_from_slice(&self.source_rgba);
        } else {
            self.preview_src_rgba = resize_bilinear_rgba(
                bytemuck::cast_slice(&self.source_rgba),
                self.source_w,
                self.source_h,
                dw,
                dh,
            );
        }
        self.preview_src_w = dw;
        self.preview_src_h = dh;
    }
}

#[cfg(test)]
mod tests;
