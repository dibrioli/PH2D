//! [`VectorTool`] — style model + panel for the Vector drawing tool.
//!
//! The tool is deliberately thin: it holds the current stroke colour, fill
//! colour, and stroke width, projected into a [`FloatingPanel`] (a Width
//! slider plus two colour palettes). The shell's `vector_bridge` reads this
//! style each frame (downcast via [`Tool::as_any_mut`]) to restyle newly drawn
//! paths and to recolour the selected path when the palette changes.
//!
//! Colour uses a **curated palette** (RadioGroup) rather than a live picker:
//! the picker is a chrome-side widget system and a tool `FloatingPanel` routes
//! only `Click`/`SetValue`/`SelectOption` through the shell — a palette is the
//! part that works end-to-end without new cross-system plumbing. A full OKLCH
//! picker is a follow-up increment.

use ph2d_a11y::NodeId;
use ph2d_editor_core::floating_panel::{
    FloatingPanel, PanelAnchor, PanelControl, PanelTab, ToolId,
};
use ph2d_editor_core::tool::{PanelEvent, Tool};
use ph2d_editor_core::widget::{RadioGroup, RadioOption, Slider};

/// Curated stroke / fill palette: `(key, label, sRGB8)`. The `key` is the
/// stable RadioGroup option value (HR-15-safe); the shell maps the tool's
/// `[u8; 4]` straight into `ph2d_vec_scene::Rgba8`.
pub const PALETTE: &[(&str, &str, [u8; 4])] = &[
    ("white", "White", [240, 240, 245, 255]),
    ("black", "Black", [20, 20, 24, 255]),
    ("gray", "Gray", [130, 130, 138, 255]),
    ("red", "Red", [220, 60, 60, 255]),
    ("orange", "Orange", [230, 140, 40, 255]),
    ("yellow", "Yellow", [235, 205, 50, 255]),
    ("green", "Green", [70, 190, 90, 255]),
    ("cyan", "Cyan", [60, 190, 205, 255]),
    ("blue", "Blue", [90, 150, 230, 255]),
    ("purple", "Purple", [160, 110, 220, 255]),
];

/// Fill "None" sentinel key + colour (alpha 0 = no visible fill on close).
const FILL_NONE_KEY: &str = "none";
const FILL_NONE: [u8; 4] = [0, 0, 0, 0];

/// Default stroke width in screen pixels (matches the old `PenTool` default).
pub const DEFAULT_STROKE_WIDTH_PX: f64 = 3.0;
/// Width slider maps normalized `0..=1` to `MIN..=MAX` px.
const WIDTH_MIN_PX: f64 = 1.0;
const WIDTH_MAX_PX: f64 = 20.0;

// Panel control ids. Tool-panel ids are independent of the chrome `ids` set
// (per the Shape tool); they need only be unique within THIS panel.
const WIDTH_ID: NodeId = NodeId(300);
const STROKE_GROUP: NodeId = NodeId(310);
const FILL_GROUP: NodeId = NodeId(330);
const STROKE_OPT_BASE: u64 = 311; // 311.. one per palette entry
const FILL_OPT_BASE: u64 = 331; // 331.. palette entries, then None

/// Vector drawing tool — style model only.
#[derive(Debug, Clone, PartialEq)]
pub struct VectorTool {
    stroke: [u8; 4],
    /// Fill applied on close; alpha 0 ⇒ no fill.
    fill: [u8; 4],
    /// Stroke width in screen pixels, held in `WIDTH_MIN_PX..=WIDTH_MAX_PX`.
    stroke_width_px: f64,
    /// Set when a palette option is chosen → the shell recolours the selected
    /// path. Drained by [`Self::take_apply_to_selected`].
    apply_to_selected: bool,
}

impl Default for VectorTool {
    fn default() -> Self {
        Self {
            stroke: color_of("white").unwrap_or([240, 240, 245, 255]),
            fill: color_of("blue").unwrap_or([90, 150, 230, 255]),
            stroke_width_px: DEFAULT_STROKE_WIDTH_PX,
            apply_to_selected: false,
        }
    }
}

/// Look up a palette colour by key.
fn color_of(key: &str) -> Option<[u8; 4]> {
    PALETTE
        .iter()
        .find(|(k, _, _)| *k == key)
        .map(|(_, _, c)| *c)
}

/// The palette key whose colour equals `rgba` (or `None` sentinel for a≈0).
fn key_of(rgba: [u8; 4]) -> &'static str {
    if rgba[3] == 0 {
        return FILL_NONE_KEY;
    }
    PALETTE
        .iter()
        .find(|(_, _, c)| *c == rgba)
        .map(|(k, _, _)| *k)
        .unwrap_or("white")
}

impl VectorTool {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Current stroke colour (sRGB8).
    #[must_use]
    pub fn stroke_rgba(&self) -> [u8; 4] {
        self.stroke
    }

    /// Current fill colour (sRGB8); alpha 0 ⇒ no fill on close.
    #[must_use]
    pub fn fill_rgba(&self) -> [u8; 4] {
        self.fill
    }

    /// Current stroke width in screen pixels.
    #[must_use]
    pub fn stroke_width_px(&self) -> f64 {
        self.stroke_width_px
    }

    /// Drain the "recolour the selected path" request (set on any palette pick).
    pub fn take_apply_to_selected(&mut self) -> bool {
        std::mem::take(&mut self.apply_to_selected)
    }

    fn stroke_options() -> Vec<RadioOption<String>> {
        PALETTE
            .iter()
            .enumerate()
            .map(|(i, (k, label, _))| {
                RadioOption::new(NodeId(STROKE_OPT_BASE + i as u64), (*k).to_string(), *label)
            })
            .collect()
    }

    fn fill_options() -> Vec<RadioOption<String>> {
        let mut opts: Vec<RadioOption<String>> = PALETTE
            .iter()
            .enumerate()
            .map(|(i, (k, label, _))| {
                RadioOption::new(NodeId(FILL_OPT_BASE + i as u64), (*k).to_string(), *label)
            })
            .collect();
        opts.push(RadioOption::new(
            NodeId(FILL_OPT_BASE + PALETTE.len() as u64),
            FILL_NONE_KEY.to_string(),
            "None",
        ));
        opts
    }
}

impl Tool for VectorTool {
    fn id(&self) -> ToolId {
        ToolId::new("vector")
    }

    fn label(&self) -> &str {
        "Vector"
    }

    fn icon_slug(&self) -> &str {
        "vector"
    }

    fn build_panel(&self) -> FloatingPanel {
        let mut width = Slider::new(WIDTH_ID, "Width");
        // Normalize current px into 0..=1 for the slider bar.
        let norm = ((self.stroke_width_px() - WIDTH_MIN_PX) / (WIDTH_MAX_PX - WIDTH_MIN_PX)) as f32;
        width.set_value(norm);

        let mut stroke = RadioGroup::new(STROKE_GROUP, "Stroke", Self::stroke_options());
        stroke.select(key_of(self.stroke).to_string());

        let mut fill = RadioGroup::new(FILL_GROUP, "Fill", Self::fill_options());
        fill.select(key_of(self.fill).to_string());

        let mut panel = FloatingPanel::new(self.id(), "Vector")
            .with_tabs(vec![PanelTab {
                label: "Style".into(),
                icon: None,
                active: true,
            }])
            .with_controls(vec![
                PanelControl::Slider(width),
                PanelControl::RadioGroup(stroke),
                PanelControl::RadioGroup(fill),
            ]);
        panel.anchor = PanelAnchor::BottomCenter;
        panel
    }

    fn handle_panel_event(&mut self, event: PanelEvent) {
        match event {
            PanelEvent::SetValue(id, v) if id == WIDTH_ID => {
                self.stroke_width_px =
                    WIDTH_MIN_PX + v.clamp(0.0, 1.0) * (WIDTH_MAX_PX - WIDTH_MIN_PX);
            }
            PanelEvent::SelectOption(id, value) if id == STROKE_GROUP => {
                if let Some(c) = color_of(&value) {
                    self.stroke = c;
                    self.apply_to_selected = true;
                }
            }
            PanelEvent::SelectOption(id, value) if id == FILL_GROUP => {
                self.fill = if value == FILL_NONE_KEY {
                    FILL_NONE
                } else {
                    color_of(&value).unwrap_or(FILL_NONE)
                };
                self.apply_to_selected = true;
            }
            _ => {}
        }
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fresh_tool_defaults() {
        let t = VectorTool::new();
        assert_eq!(t.stroke_rgba(), [240, 240, 245, 255]); // white
        assert_eq!(t.fill_rgba(), [90, 150, 230, 255]); // blue
        assert_eq!(t.stroke_width_px(), DEFAULT_STROKE_WIDTH_PX);
    }

    #[test]
    fn width_slider_maps_normalized_to_px() {
        let mut t = VectorTool::new();
        Tool::handle_panel_event(&mut t, PanelEvent::SetValue(WIDTH_ID, 0.0));
        assert_eq!(t.stroke_width_px(), WIDTH_MIN_PX);
        Tool::handle_panel_event(&mut t, PanelEvent::SetValue(WIDTH_ID, 1.0));
        assert_eq!(t.stroke_width_px(), WIDTH_MAX_PX);
    }

    #[test]
    fn stroke_palette_sets_colour_and_flags_apply() {
        let mut t = VectorTool::new();
        assert!(!t.take_apply_to_selected());
        Tool::handle_panel_event(
            &mut t,
            PanelEvent::SelectOption(STROKE_GROUP, "red".to_string()),
        );
        assert_eq!(t.stroke_rgba(), [220, 60, 60, 255]);
        assert!(t.take_apply_to_selected());
        assert!(!t.take_apply_to_selected(), "drained");
    }

    #[test]
    fn fill_none_sets_transparent() {
        let mut t = VectorTool::new();
        Tool::handle_panel_event(
            &mut t,
            PanelEvent::SelectOption(FILL_GROUP, "none".to_string()),
        );
        assert_eq!(t.fill_rgba()[3], 0);
    }

    #[test]
    fn foreign_node_id_ignored() {
        let mut t = VectorTool::new();
        let before = t.clone();
        Tool::handle_panel_event(&mut t, PanelEvent::SetValue(NodeId(999), 0.5));
        Tool::handle_panel_event(&mut t, PanelEvent::SelectOption(NodeId(999), "red".into()));
        assert_eq!(t, before);
    }

    #[test]
    fn panel_has_width_and_two_palettes_with_active_selection() {
        let mut t = VectorTool::new();
        Tool::handle_panel_event(
            &mut t,
            PanelEvent::SelectOption(STROKE_GROUP, "green".to_string()),
        );
        let panel = t.build_panel();
        assert_eq!(panel.controls.len(), 3);
        assert!(matches!(panel.controls[0], PanelControl::Slider(_)));
        let PanelControl::RadioGroup(stroke) = &panel.controls[1] else {
            panic!("expected stroke RadioGroup");
        };
        assert_eq!(stroke.options.len(), PALETTE.len());
        assert_eq!(stroke.selected.as_deref(), Some("green"));
        let PanelControl::RadioGroup(fill) = &panel.controls[2] else {
            panic!("expected fill RadioGroup");
        };
        assert_eq!(fill.options.len(), PALETTE.len() + 1); // + None
    }

    #[test]
    fn id_label_icon_stable() {
        let t = VectorTool::new();
        assert_eq!(t.id(), ToolId::new("vector"));
        assert_eq!(t.label(), "Vector");
        assert_eq!(t.icon_slug(), "vector");
    }
}
