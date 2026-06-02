//! [`VectorSelectTool`] — network-level selection (click + marquee).
//!
//! Per plan **T2.3** (`docs/Vector Module/17_plano_de_implementacao.md` §5)
//! and Coord decisions (`787b6fc`): Select operates on the shared committed
//! vector scene (`App::committed_vector_pen_paths`) + the shared
//! [`VectorSelection`] (owned by `App`, passed by-ref). It does NOT create
//! geometry — it reads the scene to pick networks.
//!
//! The shell calls the inherent methods below after downcasting via
//! [`Tool::as_any_mut`] (ADR-0040 §3 exception). The persistent selection
//! lives in `App`; this tool holds only the transient **marquee rect**
//! (drag anchor → current) for the overlay.
//!
//! ## Interaction
//!
//! - **Click** on a filled region → select that network (topmost wins);
//!   shift-click toggles; click on empty canvas clears.
//! - **Marquee drag** → select every network whose bounding box matches
//!   the rect under [`MarqueeMode`] (Crossing = touches, Window = fully
//!   contained).
//!
//! Click-select hits filled **regions** (`region_contains_point`); open
//! stroked paths (Pencil / spiral) are selected via the marquee (bbox) —
//! proximity click-select for open paths is a later refinement.

use glam::Vec2;
use ph2d_editor_core::floating_panel::{FloatingPanel, ToolId};
use ph2d_editor_core::tool::{PanelEvent, Tool};
use ph2d_vector_doc::{Ph2dVectorAsset, VectorSelection};

/// How a marquee rect picks networks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MarqueeMode {
    /// Select networks the rect *touches* (bbox intersects) — Illustrator
    /// Selection-tool default.
    #[default]
    Crossing,
    /// Select only networks fully *inside* the rect.
    Window,
}

/// Outcome of a Select interaction (drives overlay invalidation / status).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectOutcome {
    /// A single network was selected (its committed index).
    SelectedNetwork(usize),
    /// The selection toggled a network (shift-click); carries the index.
    ToggledNetwork(usize),
    /// The marquee selected `n` networks.
    MarqueeSelected(usize),
    /// The click hit empty canvas and cleared the selection.
    Cleared,
    /// Nothing changed.
    NoChange,
}

/// Network-level selection tool. Holds only the transient marquee rect.
#[derive(Debug, Clone, Default)]
pub struct VectorSelectTool {
    marquee_anchor: Option<Vec2>,
    marquee_current: Vec2,
}

impl VectorSelectTool {
    /// Fresh tool.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Handle a click at world `pos`. `additive` = shift held.
    ///
    /// Picks the **topmost** network whose filled region contains `pos`
    /// (later committed = drawn on top = higher index). Plain click selects
    /// only it; shift-click toggles; a miss clears (plain) or no-ops
    /// (shift).
    pub fn click(
        &self,
        committed: &[Ph2dVectorAsset],
        selection: &mut VectorSelection,
        pos: Vec2,
        additive: bool,
    ) -> SelectOutcome {
        match Self::network_at_point(committed, pos) {
            Some(idx) => {
                if additive {
                    selection.toggle_network(idx);
                    SelectOutcome::ToggledNetwork(idx)
                } else {
                    selection.select_only_network(idx);
                    SelectOutcome::SelectedNetwork(idx)
                }
            }
            None => {
                // Shift-click on empty keeps the selection; a plain click
                // on empty clears it (no-op if already empty).
                if additive || selection.is_empty() {
                    SelectOutcome::NoChange
                } else {
                    selection.clear();
                    SelectOutcome::Cleared
                }
            }
        }
    }

    /// Topmost network index whose filled region contains `pos`, or `None`.
    #[must_use]
    pub fn network_at_point(committed: &[Ph2dVectorAsset], pos: Vec2) -> Option<usize> {
        for (idx, asset) in committed.iter().enumerate().rev() {
            for region in &asset.network.regions {
                if asset.network.region_contains_point(region, pos) {
                    return Some(idx);
                }
            }
        }
        None
    }

    /// Begin a marquee drag at `pos`.
    pub fn begin_marquee(&mut self, pos: Vec2) {
        self.marquee_anchor = Some(pos);
        self.marquee_current = pos;
    }

    /// Update the marquee end-point.
    pub fn update_marquee(&mut self, pos: Vec2) {
        if self.marquee_anchor.is_some() {
            self.marquee_current = pos;
        }
    }

    /// `true` while a marquee drag is active.
    #[must_use]
    pub fn is_marqueeing(&self) -> bool {
        self.marquee_anchor.is_some()
    }

    /// The current marquee rect `(min, max)` for the overlay, or `None`.
    #[must_use]
    pub fn marquee_rect(&self) -> Option<(Vec2, Vec2)> {
        let a = self.marquee_anchor?;
        let b = self.marquee_current;
        Some((a.min(b), a.max(b)))
    }

    /// Finish the marquee: select every network matching the rect under
    /// `mode`, replacing the selection. Resets the marquee.
    pub fn finish_marquee(
        &mut self,
        committed: &[Ph2dVectorAsset],
        selection: &mut VectorSelection,
        mode: MarqueeMode,
    ) -> SelectOutcome {
        let Some((min, max)) = self.marquee_rect() else {
            return SelectOutcome::NoChange;
        };
        self.marquee_anchor = None;
        let hits: Vec<usize> = committed
            .iter()
            .enumerate()
            .filter_map(|(idx, asset)| {
                let bbox = asset.network.bounding_box()?;
                let matches = match mode {
                    MarqueeMode::Crossing => bbox_intersects(bbox, (min, max)),
                    MarqueeMode::Window => bbox_contains(bbox, (min, max)),
                };
                matches.then_some(idx)
            })
            .collect();
        let n = hits.len();
        selection.set_networks(hits);
        SelectOutcome::MarqueeSelected(n)
    }

    /// Abandon the in-progress marquee (Esc).
    pub fn cancel_marquee(&mut self) {
        self.marquee_anchor = None;
    }
}

/// `true` if box `b` intersects box `r` (touching counts).
fn bbox_intersects(b: (Vec2, Vec2), r: (Vec2, Vec2)) -> bool {
    b.0.x <= r.1.x && b.1.x >= r.0.x && b.0.y <= r.1.y && b.1.y >= r.0.y
}

/// `true` if box `b` is fully inside box `r`.
fn bbox_contains(b: (Vec2, Vec2), r: (Vec2, Vec2)) -> bool {
    b.0.x >= r.0.x && b.1.x <= r.1.x && b.0.y >= r.0.y && b.1.y <= r.1.y
}

impl Tool for VectorSelectTool {
    fn id(&self) -> ToolId {
        ToolId::new("vector_select")
    }

    fn label(&self) -> &str {
        "Vector Select"
    }

    fn icon_slug(&self) -> &str {
        "vector-select"
    }

    fn build_panel(&self) -> FloatingPanel {
        FloatingPanel::new(self.id(), "Vector Select")
    }

    fn on_deactivate(&mut self) {
        self.marquee_anchor = None;
    }

    fn handle_panel_event(&mut self, _event: PanelEvent) {}

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_vector_doc::{Region, Segment, StyleTable, Vertex, VectorNetwork, WindingRule};

    /// A filled square network at `[origin, origin+size]`.
    fn square_asset(origin: Vec2, size: f32) -> Ph2dVectorAsset {
        let mut net = VectorNetwork::empty();
        let corners = [
            origin,
            origin + Vec2::new(size, 0.0),
            origin + Vec2::new(size, size),
            origin + Vec2::new(0.0, size),
        ];
        for (i, &p) in corners.iter().enumerate() {
            net.vertices.push(Vertex::auto(i as u32, p));
        }
        for i in 0..4u32 {
            net.segments.push(Segment::straight(i, i, (i + 1) % 4));
        }
        let mut r = Region::new(0, WindingRule::NonZero);
        r.segments = (0..4u32).map(|i| (i, true)).collect();
        r.fill = Some(0);
        net.regions.push(r);
        Ph2dVectorAsset::from_network(net, StyleTable::default())
    }

    #[test]
    fn click_inside_selects_that_network() {
        let scene = vec![square_asset(Vec2::ZERO, 50.0)];
        let mut sel = VectorSelection::new();
        let t = VectorSelectTool::new();
        let out = t.click(&scene, &mut sel, Vec2::new(25.0, 25.0), false);
        assert_eq!(out, SelectOutcome::SelectedNetwork(0));
        assert!(sel.contains_network(0));
    }

    #[test]
    fn click_picks_topmost_overlapping_network() {
        // Two overlapping squares; the later (index 1) is on top.
        let scene = vec![
            square_asset(Vec2::ZERO, 50.0),
            square_asset(Vec2::new(20.0, 20.0), 50.0),
        ];
        let mut sel = VectorSelection::new();
        let t = VectorSelectTool::new();
        // (30,30) is inside both → topmost (1) wins.
        let out = t.click(&scene, &mut sel, Vec2::new(30.0, 30.0), false);
        assert_eq!(out, SelectOutcome::SelectedNetwork(1));
    }

    #[test]
    fn click_empty_clears_selection() {
        let scene = vec![square_asset(Vec2::ZERO, 50.0)];
        let mut sel = VectorSelection::new();
        sel.select_only_network(0);
        let t = VectorSelectTool::new();
        let out = t.click(&scene, &mut sel, Vec2::new(500.0, 500.0), false);
        assert_eq!(out, SelectOutcome::Cleared);
        assert!(sel.is_empty());
    }

    #[test]
    fn shift_click_toggles() {
        let scene = vec![
            square_asset(Vec2::ZERO, 50.0),
            square_asset(Vec2::new(100.0, 0.0), 50.0),
        ];
        let mut sel = VectorSelection::new();
        let t = VectorSelectTool::new();
        t.click(&scene, &mut sel, Vec2::new(25.0, 25.0), false); // select 0
        let out = t.click(&scene, &mut sel, Vec2::new(125.0, 25.0), true); // shift-add 1
        assert_eq!(out, SelectOutcome::ToggledNetwork(1));
        assert!(sel.contains_network(0) && sel.contains_network(1));
    }

    #[test]
    fn marquee_crossing_selects_touched_networks() {
        let scene = vec![
            square_asset(Vec2::ZERO, 50.0),          // 0..50
            square_asset(Vec2::new(200.0, 0.0), 50.0), // 200..250
        ];
        let mut sel = VectorSelection::new();
        let mut t = VectorSelectTool::new();
        t.begin_marquee(Vec2::new(-10.0, -10.0));
        t.update_marquee(Vec2::new(60.0, 60.0)); // covers square 0, touches it
        let out = t.finish_marquee(&scene, &mut sel, MarqueeMode::Crossing);
        assert_eq!(out, SelectOutcome::MarqueeSelected(1));
        assert_eq!(sel.networks, vec![0]);
        assert!(!t.is_marqueeing());
    }

    #[test]
    fn marquee_window_requires_full_containment() {
        let scene = vec![square_asset(Vec2::ZERO, 50.0)];
        let mut sel = VectorSelection::new();
        let mut t = VectorSelectTool::new();
        // Rect only partially covers the square → Window selects nothing,
        // Crossing selects it.
        t.begin_marquee(Vec2::new(-10.0, -10.0));
        t.update_marquee(Vec2::new(25.0, 25.0));
        assert_eq!(
            t.finish_marquee(&scene, &mut sel, MarqueeMode::Window),
            SelectOutcome::MarqueeSelected(0)
        );
        t.begin_marquee(Vec2::new(-10.0, -10.0));
        t.update_marquee(Vec2::new(60.0, 60.0));
        assert_eq!(
            t.finish_marquee(&scene, &mut sel, MarqueeMode::Window),
            SelectOutcome::MarqueeSelected(1)
        );
    }

    #[test]
    fn marquee_rect_normalizes_drag_direction() {
        let mut t = VectorSelectTool::new();
        t.begin_marquee(Vec2::new(60.0, 60.0));
        t.update_marquee(Vec2::new(10.0, 20.0)); // drag up-left
        let (min, max) = t.marquee_rect().expect("rect");
        assert_eq!(min, Vec2::new(10.0, 20.0));
        assert_eq!(max, Vec2::new(60.0, 60.0));
    }

    #[test]
    fn tool_identity() {
        let t = VectorSelectTool::new();
        assert_eq!(t.id(), ToolId::new("vector_select"));
        assert_eq!(t.icon_slug(), "vector-select");
    }
}
