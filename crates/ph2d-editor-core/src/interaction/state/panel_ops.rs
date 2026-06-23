//! Panel + scroll + resize + z-order methods on [`WidgetStore`].
//!
//! Extracted from [`super`] (Track D5). All the per-panel state —
//! scroll offset, painter-published rect / content-height / visible-
//! height, manual-resize delta + anchor, z-order, scrollbar drag —
//! lives in its own `impl WidgetStore` block here.

use super::WidgetStore;
use crate::interaction::drag::ScrollbarDragAnchor;
use crate::zones::Rect;
use ph2d_a11y::NodeId;

impl WidgetStore {
    /// Read the manual-resize delta `(dw, dh)` for a panel. Defaults
    /// to `(0, 0)` if no resize has happened.
    pub fn panel_resize_delta(&self, panel: NodeId) -> (f32, f32) {
        self.panel_resize_delta
            .get(&panel)
            .copied()
            .unwrap_or((0.0, 0.0))
    }

    pub fn set_panel_resize_delta(&mut self, panel: NodeId, dw: f32, dh: f32) {
        self.panel_resize_delta.insert(panel, (dw, dh));
    }

    pub fn begin_panel_resize(&mut self, panel: NodeId, cursor_x: f32, cursor_y: f32) {
        self.panel_resize_anchor = Some((panel, cursor_x, cursor_y));
    }

    pub fn panel_resize_anchor(&self) -> Option<(NodeId, f32, f32)> {
        self.panel_resize_anchor
    }

    pub fn update_panel_resize_cursor(&mut self, cursor_x: f32, cursor_y: f32) {
        if let Some((panel, _, _)) = self.panel_resize_anchor {
            self.panel_resize_anchor = Some((panel, cursor_x, cursor_y));
        }
    }

    pub fn end_panel_resize(&mut self) {
        self.panel_resize_anchor = None;
        self.panel_resize_anchor_bl = None;
    }

    /// Begin a resize from the bottom-LEFT corner. The Move handler
    /// (`dispatch::pointer`) treats this anchor differently from the
    /// bottom-right one: it both grows the panel (resize delta) AND
    /// shifts the panel offset, keeping the right edge anchored.
    pub fn begin_panel_resize_bl(&mut self, panel: NodeId, cursor_x: f32, cursor_y: f32) {
        self.panel_resize_anchor_bl = Some((panel, cursor_x, cursor_y));
    }

    /// Read the bottom-left resize anchor, if any. Sibling of
    /// [`panel_resize_anchor`](Self::panel_resize_anchor).
    pub fn panel_resize_anchor_bl(&self) -> Option<(NodeId, f32, f32)> {
        self.panel_resize_anchor_bl
    }

    pub fn update_panel_resize_cursor_bl(&mut self, cursor_x: f32, cursor_y: f32) {
        if let Some((panel, _, _)) = self.panel_resize_anchor_bl {
            self.panel_resize_anchor_bl = Some((panel, cursor_x, cursor_y));
        }
    }

    /// Move `panel_id` to the end of the z-order (= topmost). If
    /// the id isn't in the list yet, append it. Called by dispatch
    /// whenever a panel is clicked, dragged, or first opened.
    pub fn bump_panel_z(&mut self, panel_id: NodeId) {
        self.panel_z_order.retain(|id| *id != panel_id);
        self.panel_z_order.push(panel_id);
    }

    /// Read the current z-order. Bottom-first iteration (= paint
    /// order); the last element is the topmost panel.
    pub fn panel_z_order(&self) -> &[NodeId] {
        &self.panel_z_order
    }

    /// Read the vertical scroll offset for a panel (defaults to 0).
    pub fn panel_scroll(&self, panel: NodeId) -> f32 {
        self.panel_scroll.get(&panel).copied().unwrap_or(0.0)
    }

    /// Set the vertical scroll offset for a panel. Caller is
    /// responsible for clamping to `[0, content_h - visible_h]`.
    pub fn set_panel_scroll(&mut self, panel: NodeId, y: f32) {
        self.panel_scroll.insert(panel, y);
    }

    /// Painter publishes its panel rect each frame so wheel
    /// dispatch can find which panel is under the cursor.
    pub fn set_panel_rect(&mut self, panel: NodeId, rect: Rect) {
        self.panel_rects.insert(panel, rect);
    }

    /// Read the published rect of a panel. Returns `None` when no
    /// painter has registered it this frame.
    pub fn panel_rect(&self, panel: NodeId) -> Option<Rect> {
        self.panel_rects.get(&panel).copied()
    }

    /// Drop the published rect for a panel. Used by transient
    /// panels (e.g. the floating BlenderColorPicker) when they're
    /// not currently visible — so dispatch's "is the click inside
    /// this panel?" tests aren't fooled by a stale rect from a
    /// previous frame.
    pub fn clear_panel_rect(&mut self, panel: NodeId) {
        self.panel_rects.remove(&panel);
    }

    /// Total height of all painted content in a panel (sum of
    /// section heights + separators). Set by the painter each
    /// frame; read by `dispatch_wheel` to clamp at the upper
    /// bound. Missing entry = unknown content height = no upper
    /// clamp (treated as infinite).
    pub fn set_panel_content_h(&mut self, panel: NodeId, content_h: f32) {
        self.panel_content_h.insert(panel, content_h);
    }

    /// Read the total content height for a panel. Returns `None`
    /// when the painter hasn't published one yet (e.g. on the
    /// first frame).
    pub fn panel_content_h(&self, panel: NodeId) -> Option<f32> {
        self.panel_content_h.get(&panel).copied()
    }

    /// Set the exact visible body height for a panel. Painters
    /// publish this each frame; `dispatch_wheel` uses it (instead
    /// of a `panel.h - 60` heuristic) to compute `max_scroll`.
    pub fn set_panel_visible_h(&mut self, panel: NodeId, visible_h: f32) {
        self.panel_visible_h.insert(panel, visible_h);
    }

    /// Read the visible body height for a panel.
    pub fn panel_visible_h(&self, panel: NodeId) -> Option<f32> {
        self.panel_visible_h.get(&panel).copied()
    }

    /// Find the panel whose rect contains `(x, y)`. Walks all
    /// registered panels and returns the first match. Acceptable
    /// because there are only a handful of panels (~3-5); for
    /// dozens, switch to the same back-to-front Vec approach as
    /// [`crate::interaction::HitIndex`].
    pub fn panel_at(&self, x: f32, y: f32) -> Option<NodeId> {
        for (id, rect) in &self.panel_rects {
            if rect.contains(x, y) {
                return Some(*id);
            }
        }
        None
    }

    /// Begin a scrollbar drag at the given anchor. Stores the
    /// cursor / scroll / metrics snapshot until `end_scrollbar_drag`
    /// is called. Move events while this is set update
    /// `panel_scroll` proportionally.
    pub fn begin_scrollbar_drag(&mut self, anchor: ScrollbarDragAnchor) {
        self.scrollbar_drag = Some(anchor);
    }

    pub fn scrollbar_drag(&self) -> Option<ScrollbarDragAnchor> {
        self.scrollbar_drag
    }

    pub fn end_scrollbar_drag(&mut self) {
        self.scrollbar_drag = None;
    }

    /// Publish the open dropdown popover that owns scroll (id + popover rect). Called each frame the
    /// popover paints; lets the wheel + `DROPDOWN_SCROLLBAR_ID` drag route to it.
    pub fn set_dropdown_popover(&mut self, id: NodeId, rect: Rect) {
        self.dropdown_popover = Some((id, rect));
    }

    /// The open dropdown popover `(id, rect)`, if any. Stale after close — callers gate on `open`.
    pub fn dropdown_popover(&self) -> Option<(NodeId, Rect)> {
        self.dropdown_popover
    }
}
