//! Per-dab **randomize** setters (Jitter Scale / Jitter Rotate / Randomize Color), the single clamp
//! source for those UI edits. Split from `brush_settings.rs` for the workspace LOC cap; a submodule
//! of `paint` so it shares `PainterTool`'s private `paint.brush` access. The values feed
//! `ph2d_painter_brush::jitter::per_dab` at stamp time.

use crate::tool::PainterTool;
use ph2d_editor_core::tool::PanelEvent;

impl PainterTool {
    /// Route the per-dab randomize controls (Randomize Color enable + Hue/Sat/Value, Jitter Scale,
    /// Jitter Rotate) AND the seamless Tiling toggles from the layers panel's generic channel to the
    /// setters above. Returns `true` when it consumed the event. Mirrors `route_texture_layer_event`;
    /// called from `handle_panel_event` before the main match. These ids are `PAINTER_BRUSH_COLOR_*` /
    /// `PAINTER_BRUSH_JITTER_*` / `PAINTER_BRUSH_TILING_*`, which the texture-layer router does not claim.
    pub(crate) fn route_brush_jitter_event(&mut self, event: &PanelEvent) -> bool {
        use ph2d_editor_core::ids as core_ids;
        match event {
            PanelEvent::Click(id) if *id == core_ids::PAINTER_BRUSH_COLOR_JITTER_ENABLE => {
                self.toggle_brush_color_jitter_enabled();
                true
            }
            PanelEvent::Click(id) if *id == core_ids::PAINTER_BRUSH_TILING_X => {
                self.toggle_brush_tiling(0);
                true
            }
            PanelEvent::Click(id) if *id == core_ids::PAINTER_BRUSH_TILING_Y => {
                self.toggle_brush_tiling(1);
                true
            }
            PanelEvent::Click(id) if *id == core_ids::PAINTER_BRUSH_REPEAT_IMAGE => {
                self.toggle_repeat_image();
                true
            }
            PanelEvent::SetValue(id, v) => {
                let v = *v as f32;
                match *id {
                    x if x == core_ids::PAINTER_BRUSH_COLOR_JITTER_HUE => {
                        self.set_brush_color_jitter(0, v);
                        true
                    }
                    x if x == core_ids::PAINTER_BRUSH_COLOR_JITTER_SAT => {
                        self.set_brush_color_jitter(1, v);
                        true
                    }
                    x if x == core_ids::PAINTER_BRUSH_COLOR_JITTER_VAL => {
                        self.set_brush_color_jitter(2, v);
                        true
                    }
                    x if x == core_ids::PAINTER_BRUSH_JITTER_SCALE => {
                        self.set_brush_jitter_scale(v);
                        true
                    }
                    x if x == core_ids::PAINTER_BRUSH_JITTER_ROTATE => {
                        self.set_brush_jitter_rotate(v);
                        true
                    }
                    _ => false,
                }
            }
            _ => false,
        }
    }

    /// Toggle the **Randomize Color** master enable (per-dab HSV scatter of the brush colour).
    pub fn toggle_brush_color_jitter_enabled(&mut self) {
        self.paint.brush.color_jitter_enabled = !self.paint.brush.color_jitter_enabled;
    }

    /// Set the Randomize-Color amount for `slot` (`0` = Hue, `1` = Saturation, `2` = Value) from the
    /// slider's `0..1` track; out-of-range slots are ignored.
    pub fn set_brush_color_jitter(&mut self, slot: usize, t: f32) {
        let t = t.clamp(0.0, 1.0);
        match slot {
            0 => self.paint.brush.color_jitter_hue = t,
            1 => self.paint.brush.color_jitter_sat = t,
            2 => self.paint.brush.color_jitter_val = t,
            _ => {}
        }
    }

    /// Set **Jitter Scale** (per-dab radius scatter) from the slider's `0..1` track.
    pub fn set_brush_jitter_scale(&mut self, t: f32) {
        self.paint.brush.jitter_scale = t.clamp(0.0, 1.0);
    }

    /// Set **Jitter Rotate** (per-dab texture-rotation scatter) from the slider's `0..1` track.
    pub fn set_brush_jitter_rotate(&mut self, t: f32) {
        self.paint.brush.jitter_rotate = t.clamp(0.0, 1.0);
    }
}
