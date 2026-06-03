//! Trait impls for [`ColorEqualizationTool`]: the editor `Tool` (panel
//! event routing + lifecycle) and the generic `RasterEditTool` raster I/O
//! channel (ADR-0041 Wave 10 / Etapa 2). Split out of `tool/mod.rs` —
//! pure mechanical move, no behaviour change. Child-module access to the
//! parent's private struct fields keeps the lifecycle drains local.

use ph2d_editor_core::floating_panel::{FloatingPanel, PanelAnchor, ToolId};
use ph2d_editor_core::tool::{PanelEvent, RasterEditTool, Tool};

use super::ColorEqualizationTool;
use crate::ids;
use crate::params::ColorEqualizationUiEdit;

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
            PanelEvent::SetValue(id, v) if id == ids::CEQ_POSTERIZE_DITHER_STRENGTH => {
                self.apply_ui_edit(ColorEqualizationUiEdit::DitherStrengthSlider(v as f32));
            }
            PanelEvent::SetValue(id, v) if id == ids::CEQ_POSTERIZE_DITHER_STRENGTH_NUM => {
                self.apply_ui_edit(ColorEqualizationUiEdit::DitherStrength(v as f32));
            }
            PanelEvent::SetValue(id, v) if id == ids::CEQ_POSTERIZE_DITHER_GRAIN => {
                self.apply_ui_edit(ColorEqualizationUiEdit::DitherGrainSlider(v as f32));
            }
            PanelEvent::SetValue(id, v) if id == ids::CEQ_POSTERIZE_DITHER_GRAIN_NUM => {
                self.apply_ui_edit(ColorEqualizationUiEdit::DitherGrain(v as u32));
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
        // CPU fallback when not (150-380 ms when Quantize is armed).
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
