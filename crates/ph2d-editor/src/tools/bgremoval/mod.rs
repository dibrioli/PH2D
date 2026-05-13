//! [`BgRemovalTool`] — interactive background removal.
//!
//! ## Architecture
//!
//! - **Algorithm core** (this module's children): pure Rust, no UI
//!   knowledge. `apply()` is the single entry point the Integrator
//!   calls. Four algorithms (auto / colorkey / edge-aware / luminance)
//!   feed a chain of refinements (opening+closing → expand → guided
//!   filter → F-H feather) and produce a fresh RGBA buffer.
//!
//! - **Tool state + panel** (this file): implements [`Tool`] with a
//!   `FloatingPanel` describing every control. The Integrator can
//!   either mount that panel as-is (Procreate-style floater) or read
//!   the same set of widgets / NodeIds and re-render them in the
//!   Inspector. Both paths route events through
//!   `handle_panel_event()` — identical state machine.
//!
//! ## What this tool does NOT do (yet)
//!
//! - Pointer events on the canvas. Eyedropper "Sample" mode and
//!   protection-mask Paint/Erase modes are arm-able via toggles, but
//!   actually capturing canvas drags is the Integrator's job (see
//!   tool.rs §5.2 of the diretriz Implementador).
//! - Mount the tool on the toolbar / Inspector / IconId. All
//!   wiring lives outside the island.
//! - Persist or undo. Apply/Cancel return their decision and let the
//!   Integrator command-queue / undo-stack it.
//!
//! ## Public API summary for the Integrator
//!
//! ```ignore
//! let mut tool = BgRemovalTool::default();
//! // Pipe panel events from the UI:
//! tool.handle_panel_event(PanelEvent::SetValue(node::TOLERANCE, 0.4));
//! // Sample a color from the canvas (Integrator reads pixel at click):
//! tool.add_sampled_color(RgbColor::new(240, 240, 235));
//! // Compute the result (decoupled from UI):
//! let out = tool.run(&rgba, w, h, None);
//! ```

pub mod apply;
pub mod border_detect;
pub mod colorkey;
pub mod edge_aware;
pub mod island;
pub mod luminance;
pub mod oklab;
pub mod params;
pub mod refinement;

pub use apply::{Workspace, apply};
pub use border_detect::{BorderDetectOpts, detect_border_colors};
pub use island::{Island, IslandOpts, separate_islands};
pub use params::{BgRemovalAlgorithm, BgRemovalParams, RgbColor};

use crate::floating_panel::{FloatingPanel, PanelAnchor, PanelControl, PanelTab, ToolId};
use crate::tool::{PanelEvent, Tool};
use crate::widget::{RadioGroup, RadioOption, Slider, Toggle};

/// Stable a11y / panel-event NodeIds. Public so the Integrator's
/// Inspector renderer can target the same identities — single
/// contract between Tool state and any container.
pub mod node {
    use ph2d_a11y::NodeId;

    // Range chosen to not collide with `BrushTool` (100s) or
    // `MoveTool` (200s) — both seed tools use NodeId(1xx) / (2xx).
    pub const ALGORITHM: NodeId = NodeId(300);
    pub const ALG_OPT_AUTO: NodeId = NodeId(301);
    pub const ALG_OPT_COLORKEY: NodeId = NodeId(302);
    pub const ALG_OPT_EDGE: NodeId = NodeId(303);
    pub const ALG_OPT_LUMINANCE: NodeId = NodeId(304);

    pub const TOLERANCE: NodeId = NodeId(310);
    pub const EDGE_THRESHOLD: NodeId = NodeId(311);
    pub const FEATHER_WIDTH: NodeId = NodeId(312);
    pub const FEATHER_STRENGTH: NodeId = NodeId(313);
    pub const SMOOTH: NodeId = NodeId(314);
    pub const EXPAND: NodeId = NodeId(315);

    pub const INVERT: NodeId = NodeId(320);
    pub const AUTO_CLEAN: NodeId = NodeId(321);
    pub const SEPARATE_ISLANDS: NodeId = NodeId(322);

    pub const SAMPLE_MODE: NodeId = NodeId(330);
    pub const MASK_PAINT_MODE: NodeId = NodeId(331);
    pub const MASK_ERASE_MODE: NodeId = NodeId(332);
}

/// Mutually-exclusive canvas interaction modes the user can arm via
/// the panel toggles. The Integrator inspects `tool.mode()` to know
/// which pointer-event handler to route through.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum InteractionMode {
    /// Sliders adjust parameters; no canvas interaction.
    #[default]
    Idle,
    /// Eyedropper armed — next canvas click samples a color.
    Sampling,
    /// Protection brush armed — drags add to the protection mask.
    MaskPaint,
    /// Protection brush armed — drags erase from the protection mask.
    MaskErase,
}

/// Brush settings used by the protection-mask paint/erase modes.
/// Sliders surface these to the user; the algorithm doesn't care
/// — the Integrator owns the actual painting into a Float32 buffer.
#[derive(Copy, Clone, Debug)]
pub struct MaskBrushSettings {
    /// Brush radius in pixels. 5..=150 (legacy range).
    pub size: f32,
    /// Hardness of the falloff. 0..=100 (0 = soft, 100 = hard).
    pub hardness: f32,
}

impl Default for MaskBrushSettings {
    fn default() -> Self {
        Self {
            size: 30.0,
            hardness: 60.0,
        }
    }
}

/// Background-removal Tool. Owns the parameters, the sampled color
/// list, the interaction mode, and the brush settings. The actual
/// algorithm + scratch buffers live in [`Workspace`] — created and
/// passed in by the Integrator (cheap to keep one per active sprite).
pub struct BgRemovalTool {
    pub params: BgRemovalParams,
    pub mode: InteractionMode,
    pub brush: MaskBrushSettings,
    pub workspace: Workspace,
}

impl Default for BgRemovalTool {
    fn default() -> Self {
        Self {
            params: BgRemovalParams::default(),
            mode: InteractionMode::Idle,
            brush: MaskBrushSettings::default(),
            workspace: Workspace::new(),
        }
    }
}

impl BgRemovalTool {
    /// Snapshot of the interaction mode. The Integrator uses this to
    /// decide which canvas event handler is active (sample / paint /
    /// erase / neither).
    pub fn mode(&self) -> InteractionMode {
        self.mode
    }

    /// Add `color` to the sampled list. Deduplicates per-component
    /// (≤8 unit difference) — matches legacy "fast dedup" behavior.
    pub fn add_sampled_color(&mut self, color: RgbColor) {
        if self
            .params
            .sampled_colors
            .iter()
            .any(|c| close(*c, color, 8))
        {
            return;
        }
        self.params.sampled_colors.push(color);
    }

    /// Drop all sampled colors. Next `run()` will fall back to k-means
    /// border auto-detection.
    pub fn clear_sampled_colors(&mut self) {
        self.params.sampled_colors.clear();
    }

    /// Wipe the sampled list + parameters → defaults, leave brush
    /// and mode alone.
    pub fn reset(&mut self) {
        self.params = BgRemovalParams::default();
    }

    /// Run the algorithm with the current parameters. Returns a fresh
    /// RGBA buffer (same dimensions as input).
    pub fn run(&mut self, rgba: &[u8], w: u32, h: u32, protection: Option<&[f32]>) -> Vec<u8> {
        apply::apply(rgba, w, h, &self.params, protection, &mut self.workspace)
    }

    /// Convenience: run the island-separation pass on an already-
    /// processed RGBA buffer (typically the output of `run()` after
    /// the user committed). Defers to the standalone `separate_islands`.
    pub fn split_islands(rgba: &[u8], w: u32, h: u32, opts: IslandOpts) -> Vec<Island> {
        island::separate_islands(rgba, w, h, opts)
    }
}

#[inline]
fn close(a: RgbColor, b: RgbColor, eps: i32) -> bool {
    (a.r as i32 - b.r as i32).abs() <= eps
        && (a.g as i32 - b.g as i32).abs() <= eps
        && (a.b as i32 - b.b as i32).abs() <= eps
}

impl Tool for BgRemovalTool {
    fn id(&self) -> ToolId {
        ToolId::new("bgremoval")
    }

    fn label(&self) -> &str {
        "BG Removal"
    }

    fn icon_slug(&self) -> &str {
        // Integrator maps this slug → SVG. Recommended Lucide icon:
        // `wand-sparkles` (magic-select metaphor).
        "bgremoval"
    }

    fn build_panel(&self) -> FloatingPanel {
        // Algorithm radio.
        let algo_options = vec![
            RadioOption {
                value: BgRemovalAlgorithm::Auto.tag().to_string(),
                label: "Auto".to_string(),
                id: node::ALG_OPT_AUTO,
            },
            RadioOption {
                value: BgRemovalAlgorithm::ColorKey.tag().to_string(),
                label: "Color".to_string(),
                id: node::ALG_OPT_COLORKEY,
            },
            RadioOption {
                value: BgRemovalAlgorithm::EdgeAware.tag().to_string(),
                label: "Edge".to_string(),
                id: node::ALG_OPT_EDGE,
            },
            RadioOption {
                value: BgRemovalAlgorithm::Luminance.tag().to_string(),
                label: "Luma".to_string(),
                id: node::ALG_OPT_LUMINANCE,
            },
        ];
        let mut algorithm = RadioGroup::new(node::ALGORITHM, "Algorithm", algo_options);
        algorithm.select(self.params.algorithm.tag().to_string());

        // Sliders — all normalized 0..=1 for the widget; the
        // `handle_panel_event` map converts back to documented ranges.
        let mut tolerance = Slider::new(node::TOLERANCE, "Tolerance");
        tolerance.set_value(self.params.tolerance / 100.0);
        tolerance.accent = true;

        let mut edge_threshold = Slider::new(node::EDGE_THRESHOLD, "Edge");
        edge_threshold.set_value(self.params.edge_threshold / 100.0);

        let mut feather_w = Slider::new(node::FEATHER_WIDTH, "Feather W");
        feather_w.set_value(self.params.feather_width / 20.0);

        let mut feather_s = Slider::new(node::FEATHER_STRENGTH, "Feather S");
        feather_s.set_value(self.params.feather_strength / 100.0);

        let mut smooth = Slider::new(node::SMOOTH, "Smooth");
        smooth.set_value(self.params.smooth_amount / 10.0);

        let mut expand = Slider::new(node::EXPAND, "Expand");
        // -5..=5 → 0..=1 (0.5 = no expansion).
        expand.set_value((self.params.mask_expand + 5.0) / 10.0);

        // Toggles.
        let mut invert = Toggle::new(node::INVERT, "Invert");
        invert.on = self.params.invert_mask;
        let mut auto_clean = Toggle::new(node::AUTO_CLEAN, "Clean");
        auto_clean.on = self.params.auto_clean;
        let mut separate_isl = Toggle::new(node::SEPARATE_ISLANDS, "Split");
        // Toggle reflects user intent — actual split is the
        // Integrator's call on apply commit.
        separate_isl.on = false;

        let mut sample = Toggle::new(node::SAMPLE_MODE, "Sample");
        sample.on = self.mode == InteractionMode::Sampling;
        let mut mask_paint = Toggle::new(node::MASK_PAINT_MODE, "Mask+");
        mask_paint.on = self.mode == InteractionMode::MaskPaint;
        let mut mask_erase = Toggle::new(node::MASK_ERASE_MODE, "Mask-");
        mask_erase.on = self.mode == InteractionMode::MaskErase;

        let controls = vec![
            PanelControl::RadioGroup(algorithm),
            PanelControl::Slider(tolerance),
            PanelControl::Slider(edge_threshold),
            PanelControl::Slider(feather_w),
            PanelControl::Slider(feather_s),
            PanelControl::Slider(smooth),
            PanelControl::Slider(expand),
            PanelControl::Toggle(invert),
            PanelControl::Toggle(auto_clean),
            PanelControl::Toggle(separate_isl),
            PanelControl::Toggle(sample),
            PanelControl::Toggle(mask_paint),
            PanelControl::Toggle(mask_erase),
        ];

        let mut panel = FloatingPanel::new(self.id(), "BG Removal")
            .with_tabs(vec![PanelTab {
                label: "Algorithm".into(),
                icon: None,
                active: true,
            }])
            .with_controls(controls);
        panel.anchor = PanelAnchor::BottomCenter;
        // Wider than Brush — many controls. Inspector hosting still
        // uses arbitrary width; this size only matters for the
        // standalone preview path.
        panel.width = 720.0;
        panel.height = 110.0;
        panel
    }

    fn handle_panel_event(&mut self, event: PanelEvent) {
        match event {
            PanelEvent::SetValue(id, v) if id == node::TOLERANCE => {
                self.params.tolerance = (v.clamp(0.0, 1.0) * 100.0) as f32;
            }
            PanelEvent::SetValue(id, v) if id == node::EDGE_THRESHOLD => {
                self.params.edge_threshold = (v.clamp(0.0, 1.0) * 100.0) as f32;
            }
            PanelEvent::SetValue(id, v) if id == node::FEATHER_WIDTH => {
                self.params.feather_width = (v.clamp(0.0, 1.0) * 20.0) as f32;
            }
            PanelEvent::SetValue(id, v) if id == node::FEATHER_STRENGTH => {
                self.params.feather_strength = (v.clamp(0.0, 1.0) * 100.0) as f32;
            }
            PanelEvent::SetValue(id, v) if id == node::SMOOTH => {
                self.params.smooth_amount = (v.clamp(0.0, 1.0) * 10.0) as f32;
            }
            PanelEvent::SetValue(id, v) if id == node::EXPAND => {
                // 0..=1 → -5..=5
                self.params.mask_expand = (v.clamp(0.0, 1.0) * 10.0 - 5.0) as f32;
            }
            PanelEvent::Toggle(id, on) if id == node::INVERT => {
                self.params.invert_mask = on;
            }
            PanelEvent::Toggle(id, on) if id == node::AUTO_CLEAN => {
                self.params.auto_clean = on;
            }
            PanelEvent::Toggle(id, on) if id == node::SAMPLE_MODE => {
                self.mode = if on {
                    InteractionMode::Sampling
                } else if self.mode == InteractionMode::Sampling {
                    InteractionMode::Idle
                } else {
                    self.mode
                };
            }
            PanelEvent::Toggle(id, on) if id == node::MASK_PAINT_MODE => {
                self.mode = if on {
                    InteractionMode::MaskPaint
                } else if self.mode == InteractionMode::MaskPaint {
                    InteractionMode::Idle
                } else {
                    self.mode
                };
            }
            PanelEvent::Toggle(id, on) if id == node::MASK_ERASE_MODE => {
                self.mode = if on {
                    InteractionMode::MaskErase
                } else if self.mode == InteractionMode::MaskErase {
                    InteractionMode::Idle
                } else {
                    self.mode
                };
            }
            // SEPARATE_ISLANDS toggle: stored externally by Integrator
            // — the split decision applies at commit time, not in
            // params. Ignore here.
            PanelEvent::SelectOption(id, value) if id == node::ALGORITHM => {
                self.params.algorithm = BgRemovalAlgorithm::from_tag(&value);
            }
            _ => {}
        }
        self.params.clamp();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_match_spec() {
        let t = BgRemovalTool::default();
        assert_eq!(t.params.algorithm, BgRemovalAlgorithm::Auto);
        assert_eq!(t.params.tolerance, 30.0);
        assert_eq!(t.mode, InteractionMode::Idle);
        assert_eq!(t.brush.size, 30.0);
        assert_eq!(t.brush.hardness, 60.0);
    }

    #[test]
    fn id_label_icon() {
        let t = BgRemovalTool::default();
        assert_eq!(t.id(), ToolId::new("bgremoval"));
        assert_eq!(t.label(), "BG Removal");
        assert_eq!(t.icon_slug(), "bgremoval");
    }

    #[test]
    fn build_panel_has_one_tab_and_all_controls() {
        let p = BgRemovalTool::default().build_panel();
        assert_eq!(p.tool_id, ToolId::new("bgremoval"));
        assert_eq!(p.title, "BG Removal");
        assert_eq!(p.tabs.len(), 1);
        assert!(p.tabs[0].active);
        // 1 RadioGroup + 6 Sliders + 6 Toggles = 13 controls.
        assert_eq!(p.controls.len(), 13);
        let labels: Vec<&str> = p.controls.iter().map(|c| c.label()).collect();
        assert_eq!(
            labels,
            vec![
                "Algorithm",
                "Tolerance",
                "Edge",
                "Feather W",
                "Feather S",
                "Smooth",
                "Expand",
                "Invert",
                "Clean",
                "Split",
                "Sample",
                "Mask+",
                "Mask-",
            ]
        );
    }

    #[test]
    fn slider_event_updates_tolerance() {
        let mut t = BgRemovalTool::default();
        t.handle_panel_event(PanelEvent::SetValue(node::TOLERANCE, 0.4));
        assert!((t.params.tolerance - 40.0).abs() < f32::EPSILON);
    }

    #[test]
    fn expand_slider_maps_to_negative_when_below_half() {
        let mut t = BgRemovalTool::default();
        t.handle_panel_event(PanelEvent::SetValue(node::EXPAND, 0.0));
        assert!((t.params.mask_expand - (-5.0)).abs() < f32::EPSILON);

        t.handle_panel_event(PanelEvent::SetValue(node::EXPAND, 0.5));
        assert!((t.params.mask_expand - 0.0).abs() < f32::EPSILON);

        t.handle_panel_event(PanelEvent::SetValue(node::EXPAND, 1.0));
        assert!((t.params.mask_expand - 5.0).abs() < f32::EPSILON);
    }

    #[test]
    fn radio_event_switches_algorithm() {
        let mut t = BgRemovalTool::default();
        t.handle_panel_event(PanelEvent::SelectOption(
            node::ALGORITHM,
            "edge".to_string(),
        ));
        assert_eq!(t.params.algorithm, BgRemovalAlgorithm::EdgeAware);

        t.handle_panel_event(PanelEvent::SelectOption(
            node::ALGORITHM,
            "luminance".to_string(),
        ));
        assert_eq!(t.params.algorithm, BgRemovalAlgorithm::Luminance);
    }

    #[test]
    fn toggle_event_updates_invert() {
        let mut t = BgRemovalTool::default();
        assert!(!t.params.invert_mask);
        t.handle_panel_event(PanelEvent::Toggle(node::INVERT, true));
        assert!(t.params.invert_mask);
    }

    #[test]
    fn sample_mode_toggle_arms_and_disarms() {
        let mut t = BgRemovalTool::default();
        assert_eq!(t.mode, InteractionMode::Idle);
        t.handle_panel_event(PanelEvent::Toggle(node::SAMPLE_MODE, true));
        assert_eq!(t.mode, InteractionMode::Sampling);
        t.handle_panel_event(PanelEvent::Toggle(node::SAMPLE_MODE, false));
        assert_eq!(t.mode, InteractionMode::Idle);
    }

    #[test]
    fn mask_paint_mode_toggle_overrides_sample() {
        let mut t = BgRemovalTool::default();
        t.handle_panel_event(PanelEvent::Toggle(node::SAMPLE_MODE, true));
        assert_eq!(t.mode, InteractionMode::Sampling);
        t.handle_panel_event(PanelEvent::Toggle(node::MASK_PAINT_MODE, true));
        assert_eq!(t.mode, InteractionMode::MaskPaint);
    }

    #[test]
    fn add_sampled_color_deduplicates() {
        let mut t = BgRemovalTool::default();
        t.add_sampled_color(RgbColor::new(200, 100, 50));
        t.add_sampled_color(RgbColor::new(202, 102, 52)); // within 8
        t.add_sampled_color(RgbColor::new(100, 50, 25)); // far enough
        assert_eq!(t.params.sampled_colors.len(), 2);
    }

    #[test]
    fn clear_sampled_colors_empties_list() {
        let mut t = BgRemovalTool::default();
        t.add_sampled_color(RgbColor::new(200, 100, 50));
        t.clear_sampled_colors();
        assert!(t.params.sampled_colors.is_empty());
    }

    #[test]
    fn reset_restores_defaults_keeping_mode() {
        let mut t = BgRemovalTool::default();
        t.params.tolerance = 80.0;
        t.params.invert_mask = true;
        t.mode = InteractionMode::Sampling;
        t.reset();
        assert_eq!(t.params.tolerance, 30.0);
        assert!(!t.params.invert_mask);
        assert_eq!(t.mode, InteractionMode::Sampling); // mode preserved
    }

    #[test]
    fn run_smoke_returns_processed_buffer() {
        // 16×16 white image with red 8×8 center — big enough that
        // the deep-center pixel sits outside the default feather
        // radius (=2 px).
        let (w, h) = (16u32, 16u32);
        let mut img = vec![255u8; (w * h * 4) as usize];
        for i in 0..(w * h) as usize {
            img[i * 4 + 3] = 255;
        }
        for y in 4..12 {
            for x in 4..12 {
                let idx = (y * w as usize + x) * 4;
                img[idx] = 220;
                img[idx + 1] = 30;
                img[idx + 2] = 30;
            }
        }
        let mut t = BgRemovalTool::default();
        let out = t.run(&img, w, h, None);
        assert_eq!(out.len(), img.len());
        // Corner background removed.
        assert!(out[3] < 30);
        // Center subject preserved.
        let center_idx = (8 * w as usize + 8) * 4;
        assert!(out[center_idx + 3] > 200);
    }
}
