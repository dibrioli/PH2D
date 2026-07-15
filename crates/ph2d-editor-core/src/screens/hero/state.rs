//! Wave 5 stage B — HeroScreen sub-state groups.
//!
//! Replaces the flat 30+-field god-struct with cohesive groups.
//! Cross-group access still goes through the parent `HeroScreen`
//! (`hero.view.ui_mirrored`, etc.); no inter-group method
//! dependencies — each group is plain data.
//!
//! ## Group inventory
//!
//! - [`ImageEditState`] — TopBar Image-Tools mode + undo availability.
//! - [`ViewState`] — UI mirror toggle + 3 overlay visibility flags
//!   (stats HUD / widget gallery / grid overlay) + gallery rect.
//! - [`GizmoStateGroup`] — selection + per-frame view + in-progress drag.
//! - [`GridState`] — per-frame projection view + config + snap subsystem.
//!
//! Default impls match the pre-decomp `HeroScreen::new` defaults
//! (stats + grid overlay visible; everything else off / None).
//! Pre-existing snapshot types (`InspectorSpriteInfo` etc.) keep
//! their place in `hero.rs` for reach — moving them would churn the
//! import surface for no gain.

// ADR-0029 Phase C.1: `InspectorState` migrated to
// `ph2d_panel_inspector::state::InspectorState`.
// ADR-0029 Phase C.2: `HierarchyState` migrated to
// `ph2d_panel_hierarchy::state::HierarchyState`.
// ADR-0029 Phase C.3: Widget Gallery rect migrated to
// `ph2d_panel_widget_gallery::state::WidgetGalleryState::rect`;
// `widget_gallery_visible` migrated to `HeroScreen::panel_visibility`.
// All store their retained state inside `ErasedPanel` in the typed
// registry; visibility flags moved into `HeroScreen::panel_visibility`.

/// Image-edit subsystem state — TopBar Image-Tools mode flag + a
/// read-only signal mirroring the shell's image-edit undo snapshot.
#[derive(Copy, Clone, Debug, Default)]
pub struct ImageEditState {
    /// `true` when the TopBar is in **Image Tools mode**. Right-side
    /// clusters hide; image-editing action pills surface. Toggled by
    /// `TOPBAR_IMAGE_TOOLS` clicks (handled in `apply_event` before
    /// the topbar's stub). Default `false`.
    pub mode_on: bool,
    /// Read-only signal from the host: `true` when the host has a
    /// stored image-edit snapshot that Cmd+Z would restore. Lets the
    /// UI dim the `TOOL_UNDO` chip when no undo is available. Shell
    /// writes this each frame after its drain pass.
    pub has_undoable: bool,
    /// Canonical id string of the active image-edit tool (`"painter"`,
    /// `"bgremoval"`, …), or `None` when no stateful image-edit tool is
    /// active. The active tool itself lives shell-side in the
    /// `ToolRegistry` (ADR-0040: editor-core must not depend on concrete
    /// tool crates); the shell mirrors the active tool's `id()` here each
    /// frame after its `ActivateTool`/deactivate drain. Editor-core chrome
    /// reads it as an opaque string (e.g. the left rail swaps to its
    /// painter section when this `== Some("painter")`) — a dependency-legal
    /// mode signal, not a tool handle.
    pub active_tool_id: Option<&'static str>,
}

/// View-state flags — mirror toggle + overlay visibility flags
/// (stats HUD, grid overlay). All purely UI presentation — no business
/// logic. ADR-0029 Phase C.3 removed `widget_gallery_visible` (now in
/// `HeroScreen::panel_visibility` map) and `widget_gallery_rect` (now
/// on `ph2d_panel_widget_gallery::WidgetGalleryState::rect`).
#[derive(Clone, Debug)]
pub struct ViewState {
    /// When `true`, the Inspector and Hierarchy panels swap sides
    /// (Inspector left, Hierarchy right). Toggled via the "Mirror UI"
    /// entry in the theme context menu. Default `false`.
    pub ui_mirrored: bool,
    /// Visibility of the bottom statistics HUD — toggled by the
    /// "Show Statistics" entry in the theme context menu. Default
    /// `true`.
    pub stats_visible: bool,
    /// World-space grid overlay toggle (ADR-0025 M14.4b). Default
    /// `true`. Toggled via the "Show Grid" context-menu entry and the
    /// `G` key.
    pub grid_visible: bool,
    /// Center split between the scene viewport and the Motion Nodes graph
    /// (Motion Nodes M0.T4). Default [`CenterSplit::None`] — no split for any
    /// non-Motion tool. The Motion bridge sets it to the remembered orientation
    /// (default `Horizontal { t: 0.55 }`) on tool-activate and back to `None` on
    /// deactivate; the graph toolbar's SplitH/SplitV chips flip the orientation.
    pub center_split: crate::screens::layout::CenterSplit,
}

impl Default for ViewState {
    fn default() -> Self {
        Self {
            ui_mirrored: false,
            stats_visible: true,
            grid_visible: true,
            center_split: crate::screens::layout::CenterSplit::None,
        }
    }
}

/// Canvas gizmo state — current selection bits, per-frame projection
/// view, in-progress drag. Wave 2.5 promoted these from scattered
/// fields; Wave 5 groups them. Fase 0a (image-tools multi-select):
/// `selection` is the **primary** sprite (drives gizmo + inspector
/// mirror, identical to pre-Fase-0 contract); `extra_selection` holds
/// additional sprites for batch image-tool operations. Invariants
/// enforced by the mutating API methods below: `extra_selection` is
/// empty when `selection` is `None`, and never contains the primary
/// nor duplicates. Direct field mutation is still allowed for
/// single-select call-sites — to clear or replace the *whole*
/// selection, call [`replace_selection`] or [`clear_all_selection`].
#[derive(Clone, Debug, Default)]
pub struct GizmoStateGroup {
    /// M14.7 A: sim-entity bits of the sprite currently selected for
    /// gizmo manipulation. Host's canvas-click handler runs
    /// `pick_sprite_at_world` against PresentWorld and writes here.
    /// `None` = nothing selected. With multi-select, this is the
    /// **primary** of the selection.
    pub selection: Option<u64>,
    /// Fase 0a: additional sprites in the multi-selection, beyond
    /// `selection` (the primary). Image-tools drains iterate the
    /// full selection via [`Self::iter_selected`]. Empty in single-
    /// select flows, which remain the default.
    pub extra_selection: Vec<u64>,
    /// M14.7 B: per-frame projection input for the gizmo painter.
    /// Host computes from `selection_bbox_world(present, selection)`
    /// plus current camera/window and pushes here just before
    /// `paint_hero_screen`. `None` ⇒ no gizmo painted this frame.
    pub view: Option<crate::gizmo::GizmoView>,
    /// Onda 2: per-frame views of the multi-selection extras. The
    /// shell rebuilds this from `extra_selection` × PresentWorld each
    /// frame. The painter draws an outline-only gizmo (no interactive
    /// handles) for each — visual confirmation that those sprites are
    /// part of the active selection. Empty when `extra_selection` is
    /// empty or no sprites resolved.
    /// `(entity bits, view)` pairs — the bits travel WITH the view so the
    /// gizmo painter registers each handle under the correct sprite's
    /// identity. Previously this was a bare `Vec<GizmoView>` zipped against
    /// `extra_selection` at paint time; any drift between the two parallel
    /// lists (e.g. primary promotion trimming one but not the other) made a
    /// sprite's handles register under a *different* sprite's bits, so
    /// grabbing it rotated around the wrong sprite (Enio 2026-06-08).
    pub extra_views: Vec<(u64, crate::gizmo::GizmoView)>,
    /// Onda 2: per-frame "global" view covering every selected sprite
    /// — the union of all individual bboxes, no rotation. `Some` only
    /// when `selected_len() > 1`. The painter draws a distinctive
    /// outline.
    pub global_view: Option<crate::gizmo::GizmoView>,
    /// Flip W7.5: a view do gizmo da **POSE da chave** (modo Edit da tool Flip,
    /// quadro instanciado). Publicada pelo shell a cada frame; o painter a desenha
    /// como um gizmo keyed (`GizmoTarget::FlipPose`) — handles de rotate/scale,
    /// SEM interior (o translate da instância é o arrasto de canvas do Edit, e um
    /// interior aqui roubaria o clique da seleção de traço). `None` ⇒ nada pintado.
    pub pose_view: Option<crate::gizmo::GizmoView>,
    /// Onda 2C: reverse lookup from a hit NodeId to which gizmo (and
    /// which handle of it) was clicked. The painters populate this
    /// map every frame for the primary, every extra, and the global
    /// gizmo. The shell's `on_mouse_input` Down reads it after the
    /// `hit_index` lookup to decide which group-transform mode to
    /// open. Cleared at the top of `paint_hero_screen` to keep the
    /// map fresh each frame (no stale entries from sprites that left
    /// the selection).
    pub gizmo_hit_map: std::collections::BTreeMap<ph2d_a11y::NodeId, crate::gizmo::GizmoHit>,
    /// Onda 2C polish: snapshot of the global view taken at the start
    /// of a `GizmoTarget::Global` drag. While the drag is alive, the
    /// shell's `snapshots::publish` derives the live global view from
    /// this start (centre + half-extents + rotation deltas tracked
    /// against `drag.start_transform`) instead of recomputing the
    /// union of every sprite's AABB — that way the global gizmo
    /// **visually rotates** with the group during a Global Rotate
    /// (axis-aligned-union would just grow / shrink the AABB) and
    /// scales rigidly. Cleared on PointerUp.
    pub global_view_start: Option<crate::gizmo::GizmoView>,
    /// M14.7 C: in-progress drag on the gizmo. Host's MouseInput
    /// handler fills on Down landing on a handle; Move advances
    /// `cursor_screen`, calls `compute_gizmo_transform`, writes back
    /// to SimWorld; Up clears the field.
    pub drag: Option<crate::gizmo::GizmoDragState>,
}

impl GizmoStateGroup {
    /// Iterates every selected sprite — primary first, then extras
    /// in insertion order. Empty iterator when nothing is selected.
    pub fn iter_selected(&self) -> impl Iterator<Item = u64> + '_ {
        self.selection
            .into_iter()
            .chain(self.extra_selection.iter().copied())
    }

    /// Number of selected sprites (0..=1 + extras.len()).
    pub fn selected_len(&self) -> usize {
        self.selection.map_or(0, |_| 1) + self.extra_selection.len()
    }

    /// `true` when `bits` is currently selected (primary OR extra).
    pub fn is_selected(&self, bits: u64) -> bool {
        self.selection == Some(bits) || self.extra_selection.contains(&bits)
    }

    /// Drops both primary and extras. Equivalent to clicking an empty
    /// area on the canvas in single-select mode.
    pub fn clear_all_selection(&mut self) {
        self.selection = None;
        self.extra_selection.clear();
    }

    /// Replaces the whole selection with a single primary (or none),
    /// discarding any extras. Drives single-click without modifier
    /// and deselect-on-toggle.
    pub fn replace_selection(&mut self, primary: Option<u64>) {
        self.selection = primary;
        self.extra_selection.clear();
    }

    /// Adds `bits` to the selection without unsetting current ones.
    /// If nothing is selected, `bits` becomes primary. If `bits` is
    /// already selected (primary or extra), no-op. Drives Shift-click.
    pub fn add_to_selection(&mut self, bits: u64) {
        if self.selection.is_none() {
            self.selection = Some(bits);
            return;
        }
        if self.is_selected(bits) {
            return;
        }
        self.extra_selection.push(bits);
    }

    /// Toggles `bits` in the selection. If it was the primary, demote
    /// the oldest extra to primary (or clear the selection if no
    /// extras). If it was an extra, remove it. Otherwise add it as
    /// an extra (or as primary if nothing was selected). Drives
    /// Cmd/Ctrl-click on macOS / Windows.
    pub fn toggle_in_selection(&mut self, bits: u64) {
        if self.selection == Some(bits) {
            self.selection = if self.extra_selection.is_empty() {
                None
            } else {
                Some(self.extra_selection.remove(0))
            };
            return;
        }
        if let Some(pos) = self.extra_selection.iter().position(|&b| b == bits) {
            self.extra_selection.remove(pos);
            return;
        }
        self.add_to_selection(bits);
    }
}

/// Grid subsystem state — per-frame projection view + paint config +
/// snap state (overlay + per-kind config + snap policy). `grid_visible`
/// stays on [`ViewState`] since it's an overlay toggle rather than a
/// grid-subsystem field.
#[derive(Clone, Debug, Default)]
pub struct GridState {
    /// Per-frame grid projection. `None` means host hasn't supplied a
    /// view yet → grid stays hidden even if `ViewState::grid_visible`
    /// is `true`. Set each frame via
    /// [`super::HeroScreen::set_grid_view`].
    pub view: Option<crate::grid::GridView>,
    /// Spacing + color config for the grid painter. Mutate via
    /// [`super::HeroScreen::grid_config_mut`] for project-level
    /// customization.
    pub config: crate::grid::GridConfig,
    /// Grid-snap subsystem state — kind selector, per-kind config,
    /// snap policy, overlay display + opacity. Canonical source for
    /// the canvas grid overlay (paints via
    /// [`crate::grid_snap::render::paint`]) and snapping world
    /// positions (via [`crate::grid_snap::GridSnapState::snap_world`]).
    /// Panel opens/closes via `TOPBAR_GRID_SETTINGS`.
    pub snap_state: crate::grid_snap::GridSnapState,
}

#[cfg(test)]
#[allow(clippy::field_reassign_with_default)]
mod selection_tests {
    use super::GizmoStateGroup;

    const A: u64 = 0x_AAAA_AAAA;
    const B: u64 = 0x_BBBB_BBBB;
    const C: u64 = 0x_CCCC_CCCC;

    #[test]
    fn default_is_empty() {
        let g = GizmoStateGroup::default();
        assert!(g.selection.is_none());
        assert!(g.extra_selection.is_empty());
        assert_eq!(g.selected_len(), 0);
        assert_eq!(g.iter_selected().count(), 0);
        assert!(!g.is_selected(A));
    }

    #[test]
    fn replace_selection_clears_extras() {
        let mut g = GizmoStateGroup::default();
        g.selection = Some(A);
        g.extra_selection = vec![B, C];
        g.replace_selection(Some(B));
        assert_eq!(g.selection, Some(B));
        assert!(g.extra_selection.is_empty());
        assert_eq!(g.selected_len(), 1);
    }

    #[test]
    fn add_to_empty_sets_primary() {
        let mut g = GizmoStateGroup::default();
        g.add_to_selection(A);
        assert_eq!(g.selection, Some(A));
        assert!(g.extra_selection.is_empty());
    }

    #[test]
    fn add_extra_appends_after_primary() {
        let mut g = GizmoStateGroup::default();
        g.add_to_selection(A);
        g.add_to_selection(B);
        g.add_to_selection(C);
        assert_eq!(g.selection, Some(A));
        assert_eq!(g.extra_selection, vec![B, C]);
        assert_eq!(g.selected_len(), 3);
        let all: Vec<u64> = g.iter_selected().collect();
        assert_eq!(all, vec![A, B, C]);
    }

    #[test]
    fn add_duplicate_is_noop() {
        let mut g = GizmoStateGroup::default();
        g.add_to_selection(A);
        g.add_to_selection(B);
        g.add_to_selection(A); // primary already selected
        g.add_to_selection(B); // extra already selected
        assert_eq!(g.selection, Some(A));
        assert_eq!(g.extra_selection, vec![B]);
    }

    #[test]
    fn toggle_off_primary_promotes_first_extra() {
        let mut g = GizmoStateGroup::default();
        g.add_to_selection(A);
        g.add_to_selection(B);
        g.add_to_selection(C);
        g.toggle_in_selection(A);
        assert_eq!(g.selection, Some(B));
        assert_eq!(g.extra_selection, vec![C]);
    }

    #[test]
    fn toggle_off_primary_with_no_extras_clears() {
        let mut g = GizmoStateGroup::default();
        g.selection = Some(A);
        g.toggle_in_selection(A);
        assert!(g.selection.is_none());
        assert!(g.extra_selection.is_empty());
    }

    #[test]
    fn toggle_off_extra_removes_it() {
        let mut g = GizmoStateGroup::default();
        g.add_to_selection(A);
        g.add_to_selection(B);
        g.add_to_selection(C);
        g.toggle_in_selection(B);
        assert_eq!(g.selection, Some(A));
        assert_eq!(g.extra_selection, vec![C]);
    }

    #[test]
    fn toggle_unselected_adds() {
        let mut g = GizmoStateGroup::default();
        g.toggle_in_selection(A); // empty → primary
        assert_eq!(g.selection, Some(A));
        g.toggle_in_selection(B); // primary present → extra
        assert_eq!(g.extra_selection, vec![B]);
    }

    #[test]
    fn clear_all_drops_everything() {
        let mut g = GizmoStateGroup::default();
        g.add_to_selection(A);
        g.add_to_selection(B);
        g.clear_all_selection();
        assert!(g.selection.is_none());
        assert!(g.extra_selection.is_empty());
    }

    #[test]
    fn is_selected_checks_both_buckets() {
        let mut g = GizmoStateGroup::default();
        g.add_to_selection(A);
        g.add_to_selection(B);
        assert!(g.is_selected(A));
        assert!(g.is_selected(B));
        assert!(!g.is_selected(C));
    }
}
