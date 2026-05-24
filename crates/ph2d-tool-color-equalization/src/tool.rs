//! [`ColorEqualizationTool`] — stateful editor Tool for Color EQ.
//!
//! Model: `ColorEqualizationParams` (CLAHE clip / tile grid + tonal
//! adjusts + auto-WB toggle) + a cached source snapshot + a live preview
//! thumbnail (512² aspect-fit, mirroring the BgRemoval cap). The Tool
//! reacts only to its panel widgets, never to the canvas directly.
//!
//! ## Apply flow
//!
//! On every panel edit, [`Self::apply_ui_edit`] mutates `params` and (when
//! `params_dirty`) rebuilds the preview against the cached small source.
//! On Apply, [`Self::apply_ui_edit`] sets `pending_apply`. The shell drains
//! it via [`Self::take_pending_apply`] each frame; on `true` it reads each
//! selected sprite's live `Sprite.source`, runs
//! [`Self::run_full_resolution`] against that pixel buffer, and swaps the
//! texture (the BgRemoval / Padding precedent). Multi-sprite broadcast is
//! handled shell-side — the shell emits one `OneShotImageOp` per
//! `hero.gizmo.iter_selected()`.
//!
//! `RasterEditTool` (renamed from `ImageEditTool` in ADR-0041) is
//! intentionally NOT implemented yet (DIRETRIZ §3.8.3.1 — production
//! tools currently use the `as_any_mut` downcast pattern; the generic
//! raster-edit channel is fan-out Etapa 2 work in the Wave 10 perfection
//! plan).

use ph2d_editor_core::floating_panel::{FloatingPanel, PanelAnchor, ToolId};
use ph2d_editor_core::tool::{PanelEvent, RasterEditTool, Tool};

use super::algorithm::{
    HistogramData, aspect_fit_within, compute_histogram, resize_bilinear_rgba, run_pipeline,
};
use super::ids;
use super::params::{
    ColorEqualizationParams, ColorEqualizationUiEdit, ColorEqualizationUiSnapshot, apply_ui_edit,
    brightness_to_slider, clip_limit_to_slider, contrast_to_slider, denoise_strength_to_slider,
    exposure_to_slider, lut_intensity_to_slider, lut_mix_to_slider, saturation_to_slider,
    sharpen_amount_to_slider, sharpen_radius_to_slider, temperature_to_slider, tile_grid_to_slider,
    tint_to_slider, vibrance_to_slider,
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
            denoise_strength01: denoise_strength_to_slider(self.params.denoise_strength),
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
            denoise_strength: self.params.denoise_strength,
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
    /// when Quantize / Bilateral are active), reclaiming the budget
    /// the debounce-on-release strategy assumed.
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
                    &self.params,
                    &mut self.preview_rgba,
                );
            }
            self.preview_dirty = false;
        }
        (&self.preview_rgba, self.preview_src_w, self.preview_src_h)
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
        chain.run_chained(
            &gpu,
            &mut self.preview_rgba,
            self.preview_src_w,
            self.preview_src_h,
            &self.params,
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
    /// (Phase 1 tonal batch, LUT, bilateral, sharpen, auto-WB) into one
    /// upload + N dispatches + one readback via
    /// [`super::gpu::ChainedPipelineCache`]. CLAHE + auto-* normalisations
    /// still run CPU-side inside the chain. Returns `None` when no GPU
    /// adapter is available (CI runners, headless without Metal/Vulkan);
    /// callers should fall back to [`Self::run_full_resolution`] in that
    /// case.
    #[cfg(feature = "gpu")]
    pub fn run_full_resolution_gpu(&mut self, out: &mut Vec<u8>) -> Option<(u32, u32)> {
        use super::gpu::{ChainedPipelineCache, try_headless_gpu};
        assert!(self.has_source(), "set_source_snapshot must run first");
        let gpu = try_headless_gpu()?;
        let cache = ChainedPipelineCache::new(&gpu);
        let expected = (self.source_w as usize) * (self.source_h as usize) * 4;
        out.clear();
        out.resize(expected, 0);
        out.copy_from_slice(&self.source_rgba);
        cache.run_chained(&gpu, out, self.source_w, self.source_h, &self.params);
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

impl Tool for ColorEqualizationTool {
    fn id(&self) -> ToolId {
        ToolId::new("color_equalization")
    }

    fn label(&self) -> &str {
        "Color Equalization"
    }

    fn icon_slug(&self) -> &str {
        "color-equalization"
    }

    fn build_panel(&self) -> FloatingPanel {
        // The real UI is the typed `ph2d-panel-color-equalization` crate;
        // the FloatingPanel here is a minimal shell so `Tool::build_panel`
        // has a value (mirrors `PaddingTool::build_panel`). The docked
        // panel reads its layout from the snapshot the host publishes.
        let mut panel = FloatingPanel::new(self.id(), "Color Equalization");
        panel.anchor = PanelAnchor::BottomCenter;
        panel
    }

    fn on_activate(&mut self) {
        // Reset every adjustment back to defaults so re-opening the
        // panel never inherits a previous session's slider state.
        // MUST route through `apply_ui_edit::ResetAll` (not assign
        // `params` directly) so the `pending_panel_reset` flag is
        // staged for the bridge to drain — without that, only the
        // tool's params reset while the WidgetStore keeps the user's
        // last drag positions and the sliders stay visually stuck.
        self.apply_ui_edit(ColorEqualizationUiEdit::ResetAll);
    }

    fn on_deactivate(&mut self) {
        // Clear the one-shot drain flags so a cancel-mid-Apply (or any
        // deactivation while the bridge hasn't yet drained) does not fire
        // a phantom bake nor a spurious preview rerun on next activation.
        self.pending_apply = false;
        self.params_dirty = false;
        self.pending_close_lut_dropdown = None;
    }

    fn handle_panel_event(&mut self, event: PanelEvent) {
        // Sliders carry the normalized 0..1 track; the paired number chips
        // carry the natural unit. `apply_ui_edit` (in `params.rs`)
        // centralizes the clamps — every NodeId match here just routes to
        // the matching variant.
        match event {
            // Sliders (normalized 0..1 track).
            PanelEvent::SetValue(id, v) if id == ids::CEQ_CLIP_LIMIT => {
                self.apply_ui_edit(ColorEqualizationUiEdit::ClipLimitSlider(v as f32));
            }
            PanelEvent::SetValue(id, v) if id == ids::CEQ_TILE_GRID => {
                self.apply_ui_edit(ColorEqualizationUiEdit::TileGridSlider(v as f32));
            }
            PanelEvent::SetValue(id, v) if id == ids::CEQ_EXPOSURE => {
                self.apply_ui_edit(ColorEqualizationUiEdit::ExposureSlider(v as f32));
            }
            PanelEvent::SetValue(id, v) if id == ids::CEQ_TEMPERATURE => {
                self.apply_ui_edit(ColorEqualizationUiEdit::TemperatureSlider(v as f32));
            }
            PanelEvent::SetValue(id, v) if id == ids::CEQ_TINT => {
                self.apply_ui_edit(ColorEqualizationUiEdit::TintSlider(v as f32));
            }
            PanelEvent::SetValue(id, v) if id == ids::CEQ_BRIGHTNESS => {
                self.apply_ui_edit(ColorEqualizationUiEdit::BrightnessSlider(v as f32));
            }
            PanelEvent::SetValue(id, v) if id == ids::CEQ_CONTRAST => {
                self.apply_ui_edit(ColorEqualizationUiEdit::ContrastSlider(v as f32));
            }
            PanelEvent::SetValue(id, v) if id == ids::CEQ_VIBRANCE => {
                self.apply_ui_edit(ColorEqualizationUiEdit::VibranceSlider(v as f32));
            }
            PanelEvent::SetValue(id, v) if id == ids::CEQ_SATURATION => {
                self.apply_ui_edit(ColorEqualizationUiEdit::SaturationSlider(v as f32));
            }
            // Number chips (natural unit).
            PanelEvent::SetValue(id, v) if id == ids::CEQ_CLIP_LIMIT_NUM => {
                self.apply_ui_edit(ColorEqualizationUiEdit::ClipLimit(v as f32));
            }
            PanelEvent::SetValue(id, v) if id == ids::CEQ_TILE_GRID_NUM => {
                self.apply_ui_edit(ColorEqualizationUiEdit::TileGrid(v.round() as u32));
            }
            PanelEvent::SetValue(id, v) if id == ids::CEQ_EXPOSURE_NUM => {
                self.apply_ui_edit(ColorEqualizationUiEdit::Exposure(v as f32));
            }
            PanelEvent::SetValue(id, v) if id == ids::CEQ_TEMPERATURE_NUM => {
                self.apply_ui_edit(ColorEqualizationUiEdit::Temperature(v as f32));
            }
            PanelEvent::SetValue(id, v) if id == ids::CEQ_TINT_NUM => {
                self.apply_ui_edit(ColorEqualizationUiEdit::Tint(v as f32));
            }
            PanelEvent::SetValue(id, v) if id == ids::CEQ_BRIGHTNESS_NUM => {
                self.apply_ui_edit(ColorEqualizationUiEdit::Brightness(v as f32));
            }
            PanelEvent::SetValue(id, v) if id == ids::CEQ_CONTRAST_NUM => {
                self.apply_ui_edit(ColorEqualizationUiEdit::Contrast(v as f32));
            }
            PanelEvent::SetValue(id, v) if id == ids::CEQ_VIBRANCE_NUM => {
                self.apply_ui_edit(ColorEqualizationUiEdit::Vibrance(v as f32));
            }
            PanelEvent::SetValue(id, v) if id == ids::CEQ_SATURATION_NUM => {
                self.apply_ui_edit(ColorEqualizationUiEdit::Saturation(v as f32));
            }
            // Phase 2 sliders.
            PanelEvent::SetValue(id, v) if id == ids::CEQ_SHARPEN_AMOUNT => {
                self.apply_ui_edit(ColorEqualizationUiEdit::SharpenAmountSlider(v as f32));
            }
            PanelEvent::SetValue(id, v) if id == ids::CEQ_SHARPEN_AMOUNT_NUM => {
                self.apply_ui_edit(ColorEqualizationUiEdit::SharpenAmount(v as f32));
            }
            PanelEvent::SetValue(id, v) if id == ids::CEQ_SHARPEN_RADIUS => {
                self.apply_ui_edit(ColorEqualizationUiEdit::SharpenRadiusSlider(v as f32));
            }
            PanelEvent::SetValue(id, v) if id == ids::CEQ_SHARPEN_RADIUS_NUM => {
                self.apply_ui_edit(ColorEqualizationUiEdit::SharpenRadius(v as f32));
            }
            PanelEvent::SetValue(id, v) if id == ids::CEQ_DENOISE_STRENGTH => {
                self.apply_ui_edit(ColorEqualizationUiEdit::DenoiseStrengthSlider(v as f32));
            }
            PanelEvent::SetValue(id, v) if id == ids::CEQ_DENOISE_STRENGTH_NUM => {
                self.apply_ui_edit(ColorEqualizationUiEdit::DenoiseStrength(v as f32));
            }
            // Phase 3 LUT sliders.
            PanelEvent::SetValue(id, v) if id == ids::CEQ_LUT_INTENSITY => {
                self.apply_ui_edit(ColorEqualizationUiEdit::LutIntensitySlider(v as f32));
            }
            PanelEvent::SetValue(id, v) if id == ids::CEQ_LUT_INTENSITY_NUM => {
                self.apply_ui_edit(ColorEqualizationUiEdit::LutIntensity(v as f32));
            }
            PanelEvent::SetValue(id, v) if id == ids::CEQ_LUT_MIX => {
                self.apply_ui_edit(ColorEqualizationUiEdit::LutMixSlider(v as f32));
            }
            PanelEvent::SetValue(id, v) if id == ids::CEQ_LUT_MIX_NUM => {
                self.apply_ui_edit(ColorEqualizationUiEdit::LutMix(v as f32));
            }
            // Phase 3 LUT grouped-select option clicks. The dropdown chip
            // toggle is handled automatically by `InteractiveState::
            // Dropdown` in the dispatch (chip is registered in
            // populate.rs); we only need to react to OPTION clicks here.
            // Guarded by `CEQ_LUT_{1,2}_OPTS.contains(&id)` so this arm
            // doesn't shadow the auto-* button arms below.
            PanelEvent::Click(id) if ids::CEQ_LUT_1_OPTS.contains(&id) => {
                let idx = ids::CEQ_LUT_1_OPTS
                    .iter()
                    .position(|o| *o == id)
                    .unwrap_or(0);
                let preset = crate::lut_presets::LutPreset::ALL[idx];
                self.apply_ui_edit(ColorEqualizationUiEdit::SetLutPreset1(preset));
                self.pending_close_lut_dropdown = Some(1);
            }
            PanelEvent::Click(id) if ids::CEQ_LUT_2_OPTS.contains(&id) => {
                let idx = ids::CEQ_LUT_2_OPTS
                    .iter()
                    .position(|o| *o == id)
                    .unwrap_or(0);
                let preset = crate::lut_presets::LutPreset::ALL[idx];
                self.apply_ui_edit(ColorEqualizationUiEdit::SetLutPreset2(preset));
                self.pending_close_lut_dropdown = Some(2);
            }
            // Phase 5 Posterize option click. Mirror of LUT — option
            // NodeIds are unique per dropdown so the guard is
            // sufficient; `pending_close_lut_dropdown` is reused with
            // slot tags 3 (Posterize) / 4 (Quantize) so the panel's
            // close drain handles all four dropdowns uniformly.
            PanelEvent::Click(id) if ids::CEQ_POSTERIZE_OPTS.contains(&id) => {
                let idx = ids::CEQ_POSTERIZE_OPTS
                    .iter()
                    .position(|o| *o == id)
                    .unwrap_or(0);
                let levels = ids::CEQ_POSTERIZE_LEVELS[idx];
                self.apply_ui_edit(ColorEqualizationUiEdit::SetPosterizeLevels(levels));
                self.pending_close_lut_dropdown = Some(3);
            }
            PanelEvent::Click(id) if ids::CEQ_QUANTIZE_OPTS.contains(&id) => {
                let idx = ids::CEQ_QUANTIZE_OPTS
                    .iter()
                    .position(|o| *o == id)
                    .unwrap_or(0);
                let colors = ids::CEQ_QUANTIZE_COLORS[idx];
                self.apply_ui_edit(ColorEqualizationUiEdit::SetQuantizeColors(colors));
                self.pending_close_lut_dropdown = Some(4);
            }
            PanelEvent::Click(id) if id == ids::CEQ_POSTERIZE_DITHERING => {
                self.apply_ui_edit(ColorEqualizationUiEdit::ToggleDithering);
            }
            PanelEvent::Toggle(id, _) if id == ids::CEQ_POSTERIZE_DITHERING => {
                self.apply_ui_edit(ColorEqualizationUiEdit::ToggleDithering);
            }
            // Toggles + buttons.
            PanelEvent::Click(id) if id == ids::CEQ_AUTO_LEVELS => {
                self.apply_ui_edit(ColorEqualizationUiEdit::ToggleAutoLevels);
            }
            PanelEvent::Toggle(id, _) if id == ids::CEQ_AUTO_LEVELS => {
                self.apply_ui_edit(ColorEqualizationUiEdit::ToggleAutoLevels);
            }
            PanelEvent::Click(id) if id == ids::CEQ_AUTO_CONTRAST => {
                self.apply_ui_edit(ColorEqualizationUiEdit::ToggleAutoContrast);
            }
            PanelEvent::Toggle(id, _) if id == ids::CEQ_AUTO_CONTRAST => {
                self.apply_ui_edit(ColorEqualizationUiEdit::ToggleAutoContrast);
            }
            PanelEvent::Click(id) if id == ids::CEQ_AUTO_COLORS => {
                self.apply_ui_edit(ColorEqualizationUiEdit::ToggleAutoColors);
            }
            PanelEvent::Toggle(id, _) if id == ids::CEQ_AUTO_COLORS => {
                self.apply_ui_edit(ColorEqualizationUiEdit::ToggleAutoColors);
            }
            PanelEvent::Click(id) if id == ids::CEQ_AUTO_WB => {
                self.apply_ui_edit(ColorEqualizationUiEdit::ToggleAutoWb);
            }
            PanelEvent::Toggle(id, _) if id == ids::CEQ_AUTO_WB => {
                self.apply_ui_edit(ColorEqualizationUiEdit::ToggleAutoWb);
            }
            PanelEvent::Click(id) if id == ids::CEQ_APPLY => {
                self.apply_ui_edit(ColorEqualizationUiEdit::Apply);
            }
            PanelEvent::Click(id) if id == ids::CEQ_RESET => {
                self.apply_ui_edit(ColorEqualizationUiEdit::ResetAll);
            }
            _ => {}
        }
    }

    fn as_raster_edit_mut(&mut self) -> Option<&mut dyn RasterEditTool> {
        // Wave 10 / Etapa 2 (ADR-0041): CEQ joins BgRemoval as the
        // second production tool on the RasterEditTool generic channel.
        // The shell's tool-runtime helpers compose over this upcast —
        // bridges no longer need `downcast_mut::<ColorEqualizationTool>`
        // for the generic raster I/O lifecycle (set_source /
        // current_preview / take_pending_commit / run_full / deactivate).
        // CEQ-specific concerns (panel snapshot, LUT dropdown close,
        // histogram) still go through `as_any_mut` downcast — ADR-0040
        // §3 documented exception.
        Some(self)
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

/// Wave 10 / Etapa 2 (ADR-0041): CEQ drives its raster I/O lifecycle
/// through the generic `RasterEditTool` channel.
///
/// Mapping:
/// - `set_source` → wraps `set_source_snapshot`.
/// - `current_preview` → drains `params_dirty`; if dirty, calls
///   `preview_rgba()` (which internally recomputes when
///   `preview_dirty` and returns a ref to the internal cache).
///   No extra field needed — CEQ's existing `preview_rgba` is the cache.
/// - `take_pending_commit` → wraps `take_pending_apply`.
/// - `run_full` → wraps `run_full_resolution(&mut Vec)` into the
///   owned `(Vec, w, h)` shape the contract requires. Tries GPU first
///   (when `feature = "gpu"`), falls back to CPU.
/// - `deactivate` → mirrors `Tool::on_deactivate` (drain pending_apply
///   + params_dirty + close-LUT-dropdown).
impl RasterEditTool for ColorEqualizationTool {
    fn set_source(&mut self, rgba: Vec<u8>, width: u32, height: u32) {
        self.set_source_snapshot(bytemuck::allocation::cast_vec(rgba), width, height);
    }

    fn current_preview(&mut self) -> Option<(&[u8], u32, u32)> {
        if !self.take_params_dirty() {
            return None;
        }
        if !self.has_source() {
            return None;
        }
        let (pixels, w, h) = self.preview_rgba();
        if pixels.is_empty() || w == 0 || h == 0 {
            return None;
        }
        Some((pixels, w, h))
    }

    fn take_pending_commit(&mut self) -> bool {
        self.take_pending_apply()
    }

    fn run_full(&mut self) -> (Vec<u8>, u32, u32) {
        let mut out = Vec::new();
        // Prefer GPU path when `--features gpu` is on (15-30 ms at 512²);
        // CPU fallback when not (150-380 ms when Bilateral/Quantize armed).
        // Parity with the legacy bridge logic pre-Wave-10.
        #[cfg(feature = "gpu")]
        if let Some((w, h)) = self.run_full_resolution_gpu(&mut out) {
            return (out, w, h);
        }
        let (w, h) = self.run_full_resolution(&mut out);
        (out, w, h)
    }

    fn deactivate(&mut self) {
        // Mirror `Tool::on_deactivate` so the generic shell-runtime
        // deactivate path is equivalent to the registry-switch path.
        self.pending_apply = false;
        self.params_dirty = false;
        self.pending_close_lut_dropdown = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn solid(w: u32, h: u32, rgb: [u8; 3]) -> Vec<u8> {
        let mut v = Vec::with_capacity((w * h * 4) as usize);
        for _ in 0..(w * h) {
            v.extend_from_slice(&[rgb[0], rgb[1], rgb[2], 255]);
        }
        v
    }

    #[test]
    fn default_tool_has_no_source_and_no_pending() {
        let mut t = ColorEqualizationTool::default();
        assert!(!t.has_source());
        assert_eq!(t.preview_rgba().0.len(), 0);
        assert!(!t.take_pending_apply());
        assert!(!t.take_params_dirty());
    }

    #[test]
    fn id_label_icon() {
        let t = ColorEqualizationTool::default();
        assert_eq!(t.id(), ToolId::new("color_equalization"));
        assert_eq!(t.label(), "Color Equalization");
        assert_eq!(t.icon_slug(), "color-equalization");
    }

    #[test]
    fn ui_snapshot_mirrors_defaults() {
        let t = ColorEqualizationTool::default();
        let s = t.ui_snapshot();
        let dft = ColorEqualizationUiSnapshot::default();
        assert_eq!(s, dft);
    }

    #[test]
    fn slider_event_updates_params() {
        let mut t = ColorEqualizationTool::default();
        // Brightness slider mid-range (0.75 → 0.5 brightness).
        t.handle_panel_event(PanelEvent::SetValue(ids::CEQ_BRIGHTNESS, 0.75));
        assert!((t.params.brightness - 0.5).abs() < 1e-5);
        // Tile grid chip clamps.
        t.handle_panel_event(PanelEvent::SetValue(ids::CEQ_TILE_GRID_NUM, 100.0));
        assert_eq!(t.params.tile_grid_size, 16);
    }

    #[test]
    fn apply_arms_pending_once() {
        let mut t = ColorEqualizationTool::default();
        assert!(!t.take_pending_apply());
        t.handle_panel_event(PanelEvent::Click(ids::CEQ_APPLY));
        assert!(t.take_pending_apply());
        assert!(!t.take_pending_apply());
    }

    #[test]
    fn auto_wb_toggle_event_flips_param() {
        let mut t = ColorEqualizationTool::default();
        assert!(!t.params.auto_wb);
        t.handle_panel_event(PanelEvent::Click(ids::CEQ_AUTO_WB));
        assert!(t.params.auto_wb);
        t.handle_panel_event(PanelEvent::Toggle(ids::CEQ_AUTO_WB, false));
        assert!(!t.params.auto_wb);
    }

    #[test]
    fn deactivate_clears_pending_but_keeps_params() {
        let mut t = ColorEqualizationTool::default();
        t.handle_panel_event(PanelEvent::SetValue(ids::CEQ_BRIGHTNESS, 1.0));
        t.handle_panel_event(PanelEvent::Click(ids::CEQ_APPLY));
        t.on_deactivate();
        assert!(!t.take_pending_apply());
        assert!(!t.take_params_dirty());
        // Params persist.
        assert!((t.params.brightness - 1.0).abs() < 1e-5);
    }

    #[test]
    fn set_source_snapshot_marks_has_source_true() {
        let mut t = ColorEqualizationTool::default();
        let buf = solid(8, 8, [120, 80, 200]);
        t.set_source_snapshot(bytemuck::allocation::cast_vec(buf), 8, 8);
        assert!(t.has_source());
        assert_eq!(t.source_size(), (8, 8));
    }

    #[test]
    fn preview_is_built_lazily_after_param_edit() {
        let mut t = ColorEqualizationTool::default();
        t.set_source_snapshot(
            bytemuck::allocation::cast_vec(solid(8, 8, [180, 120, 60])),
            8,
            8,
        );
        // Drain the initial dirty marker that source-push armed.
        let _ = t.preview_rgba();
        t.handle_panel_event(PanelEvent::SetValue(ids::CEQ_BRIGHTNESS, 1.0));
        let (rgba, w, h) = t.preview_rgba();
        assert_eq!(w, 8);
        assert_eq!(h, 8);
        assert!(!rgba.is_empty());
        // Pixels should have been lifted by brightness.
        assert!(rgba[0] > 180);
    }

    #[test]
    fn preview_caps_at_max_dim_for_large_sources() {
        let mut t = ColorEqualizationTool::default();
        // 1024² source → preview at 512² (PREVIEW_MAX_DIM).
        t.set_source_snapshot(
            bytemuck::allocation::cast_vec(solid(1024, 1024, [128, 128, 128])),
            1024,
            1024,
        );
        let (rgba, w, h) = t.preview_rgba();
        assert_eq!(w, PREVIEW_MAX_DIM);
        assert_eq!(h, PREVIEW_MAX_DIM);
        assert_eq!(rgba.len(), (PREVIEW_MAX_DIM * PREVIEW_MAX_DIM * 4) as usize);
    }

    #[test]
    fn run_full_resolution_returns_source_dims() {
        let mut t = ColorEqualizationTool::default();
        t.set_source_snapshot(
            bytemuck::allocation::cast_vec(solid(7, 11, [50, 100, 200])),
            7,
            11,
        );
        let mut out = Vec::new();
        let (w, h) = t.run_full_resolution(&mut out);
        assert_eq!((w, h), (7, 11));
        assert_eq!(out.len(), 7 * 11 * 4);
    }

    #[test]
    fn run_full_resolution_works_after_per_sprite_source_swap() {
        // Mirrors the shell drain pattern: one ColorEqualizationTool
        // instance bakes N sprites in sequence via
        // set_source_snapshot → run_full_resolution per entity. Each
        // bake must reflect the CURRENT snapshot, not leftover state
        // from a previous sprite. This is the contract `drain_color_
        // equalization` depends on for multi-select Apply.
        let mut t = ColorEqualizationTool::default();
        // Apply non-trivial params so the pipeline actually runs (defaults
        // would still bake CLAHE, but we force a tonal change too).
        t.handle_panel_event(PanelEvent::SetValue(ids::CEQ_BRIGHTNESS, 1.0));

        // Sprite 1: red source.
        t.set_source_snapshot(
            bytemuck::allocation::cast_vec(solid(4, 4, [200, 50, 30])),
            4,
            4,
        );
        let mut out1 = Vec::new();
        let (w1, h1) = t.run_full_resolution(&mut out1);
        assert_eq!((w1, h1), (4, 4));
        let red_first_px = out1[0];

        // Sprite 2: different size + colour. The tool MUST re-bake
        // against the fresh snapshot, not reuse out1's dims/pixels.
        t.set_source_snapshot(
            bytemuck::allocation::cast_vec(solid(8, 6, [30, 200, 50])),
            8,
            6,
        );
        let mut out2 = Vec::new();
        let (w2, h2) = t.run_full_resolution(&mut out2);
        assert_eq!((w2, h2), (8, 6));
        assert_eq!(out2.len(), 8 * 6 * 4);
        // Pixel 0 of sprite 2 starts from green source — must differ
        // from sprite 1's red pixel under the same brightness boost.
        assert_ne!(red_first_px, out2[0], "per-sprite source swap leaked state");
        // Sprite 2 should have a green-dominant pixel after boost.
        assert!(out2[1] > out2[0]);
        assert!(out2[1] > out2[2]);
    }

    #[test]
    fn on_activate_resets_params_and_arms_panel_repopulate() {
        // Regression cover (§12.3 / §12.4 UI_Bugs): `on_activate` MUST
        // route through `apply_ui_edit::ResetAll` so (a) params snap to
        // defaults AND (b) `pending_panel_reset` arms so the shell
        // bridge re-runs `Panel::populate(store)` and the slider knobs
        // visually snap back. Direct mutation of `self.params` bypasses
        // the flag and leaves sliders stuck — that bug shipped twice
        // this session.
        let mut t = ColorEqualizationTool::default();
        // Dirty the state — simulate a prior session.
        t.handle_panel_event(PanelEvent::SetValue(ids::CEQ_BRIGHTNESS, 1.0));
        t.handle_panel_event(PanelEvent::Click(ids::CEQ_AUTO_WB));
        assert!((t.params.brightness - 1.0).abs() < 1e-5);
        assert!(t.params.auto_wb);
        // Drain any stray reset flag first.
        let _ = t.take_pending_panel_reset();

        t.on_activate();

        let dft = ColorEqualizationUiSnapshot::default();
        assert_eq!(t.ui_snapshot(), dft, "on_activate must reset params");
        assert!(
            t.take_pending_panel_reset(),
            "on_activate must arm pending_panel_reset"
        );
    }

    #[test]
    fn downcast_via_as_any_mut_round_trips() {
        // Mirrors the shell's downcast path for raster I/O (DIRETRIZ
        // §3.8.3.1 — production tools still use this pattern).
        let mut boxed: Box<dyn Tool> = Box::new(ColorEqualizationTool::default());
        let any = boxed.as_any_mut();
        let tool = any.downcast_mut::<ColorEqualizationTool>().unwrap();
        tool.set_source_snapshot(
            bytemuck::allocation::cast_vec(solid(4, 4, [10, 20, 30])),
            4,
            4,
        );
        assert!(tool.has_source());
    }

    // ─────────────────────────────────────────────────────────────────────
    // Wave 10 / Etapa 2 — RasterEditTool impl tests (ADR-0041 follow-up)
    // ─────────────────────────────────────────────────────────────────────

    #[test]
    fn as_raster_edit_mut_returns_some_for_ceq() {
        let mut t = ColorEqualizationTool::default();
        assert!(<dyn Tool as Tool>::as_raster_edit_mut(&mut t).is_some());
    }

    #[test]
    fn raster_edit_set_source_delegates() {
        let mut t = ColorEqualizationTool::default();
        RasterEditTool::set_source(&mut t, solid(8, 8, [100, 150, 200]), 8, 8);
        assert!(t.has_source());
        assert_eq!(t.source_size(), (8, 8));
    }

    #[test]
    fn raster_edit_current_preview_drains_dirty() {
        let mut t = ColorEqualizationTool::default();
        RasterEditTool::set_source(&mut t, solid(8, 8, [128, 128, 128]), 8, 8);
        // First call after set_source: dirty drains, preview returned.
        let frame = RasterEditTool::current_preview(&mut t);
        assert!(
            frame.is_some(),
            "first call after set_source must return Some"
        );
        let (pixels, w, h) = frame.unwrap();
        assert!(pixels.len() >= 4 && w > 0 && h > 0);
        // Second call without new edit: drained, returns None.
        let frame2 = RasterEditTool::current_preview(&mut t);
        assert!(frame2.is_none(), "dirty drained — no new frame");
    }

    #[test]
    fn raster_edit_current_preview_none_without_source() {
        let mut t = ColorEqualizationTool {
            params_dirty: true,
            ..ColorEqualizationTool::default()
        };
        assert!(RasterEditTool::current_preview(&mut t).is_none());
    }

    #[test]
    fn raster_edit_take_pending_commit_drains() {
        let mut t = ColorEqualizationTool::default();
        t.apply_ui_edit(ColorEqualizationUiEdit::Apply);
        assert!(RasterEditTool::take_pending_commit(&mut t));
        assert!(!RasterEditTool::take_pending_commit(&mut t));
    }

    #[test]
    fn raster_edit_run_full_returns_owned_buffer() {
        let mut t = ColorEqualizationTool::default();
        RasterEditTool::set_source(&mut t, solid(4, 4, [50, 100, 150]), 4, 4);
        let (out, w, h) = RasterEditTool::run_full(&mut t);
        assert!(w > 0 && h > 0);
        assert_eq!(out.len(), (w as usize) * (h as usize) * 4);
    }

    #[test]
    fn raster_edit_deactivate_clears_transient_flags() {
        let mut t = ColorEqualizationTool::default();
        t.apply_ui_edit(ColorEqualizationUiEdit::Apply);
        // params_dirty + pending_apply both should be set after Apply.
        // (apply_ui_edit marks dirty for every edit + arms pending_apply).
        RasterEditTool::deactivate(&mut t);
        assert!(!t.take_pending_apply());
        assert!(!t.take_params_dirty());
    }
}
