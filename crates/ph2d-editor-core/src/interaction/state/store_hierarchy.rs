use super::*;

impl WidgetStore {
    /// Read the hierarchy display order (empty = use fixture's
    /// default order).
    pub fn hierarchy_order(&self) -> &[NodeId] {
        &self.hierarchy_order
    }

    /// Seed the hierarchy order with the fixture's default. Called
    /// at populate time if not already populated.
    pub fn init_hierarchy_order(&mut self, ids: Vec<NodeId>) {
        if self.hierarchy_order.is_empty() {
            self.hierarchy_order = ids;
        }
    }

    /// Forcibly overwrite the hierarchy order. Used by the live-data
    /// bridge (ADR-0025 M14.4a, [`crate::screens::hero::HeroScreen::sync_from_hierarchy`])
    /// to swap the fixture's `[HIER_PLAYER]` placeholder for the
    /// host's per-frame entity list. Also clears any drag-induced
    /// `hierarchy_parent` re-parents — the host owns the tree shape
    /// when live mode is active.
    pub fn set_hierarchy_order(&mut self, ids: Vec<NodeId>) {
        self.hierarchy_order = ids;
        self.hierarchy_parent.clear();
    }

    /// Move `dragged` to land just before `target` (or at the end
    /// when `target == None`). No-op when `dragged` isn't in the
    /// order list.
    pub fn hierarchy_move(&mut self, dragged: NodeId, target: Option<NodeId>) {
        let Some(from) = self.hierarchy_order.iter().position(|i| *i == dragged) else {
            return;
        };
        let item = self.hierarchy_order.remove(from);
        let to = match target {
            Some(t) => self
                .hierarchy_order
                .iter()
                .position(|i| *i == t)
                .unwrap_or(self.hierarchy_order.len()),
            None => self.hierarchy_order.len(),
        };
        self.hierarchy_order
            .insert(to.min(self.hierarchy_order.len()), item);
    }

    pub fn hierarchy_drag(&self) -> Option<HierarchyDragState> {
        self.hierarchy_drag
    }

    pub fn begin_hierarchy_drag(
        &mut self,
        dragged: NodeId,
        down_x: f32,
        down_y: f32,
        timestamp_ns: u128,
    ) {
        self.hierarchy_drag = Some(HierarchyDragState {
            dragged,
            down_x,
            down_y,
            cursor_x: down_x,
            cursor_y: down_y,
            active: false,
            down_timestamp_ns: timestamp_ns,
        });
    }

    pub fn update_hierarchy_drag(&mut self, cursor_x: f32, cursor_y: f32) {
        if let Some(d) = self.hierarchy_drag.as_mut() {
            d.cursor_x = cursor_x;
            d.cursor_y = cursor_y;
            let dx = cursor_x - d.down_x;
            let dy = cursor_y - d.down_y;
            if (dx * dx + dy * dy) > 25.0 {
                // 5 px threshold²
                d.active = true;
            }
        }
    }

    pub fn end_hierarchy_drag(&mut self) -> Option<HierarchyDragState> {
        self.hierarchy_drag.take()
    }

    /// Parent NodeId of `child` if it's been re-parented via DnD;
    /// `None` for root rows.
    pub fn hierarchy_parent_of(&self, child: NodeId) -> Option<NodeId> {
        self.hierarchy_parent.get(&child).copied()
    }

    /// M14.6C: true when `id`'s subtree is collapsed in the panel.
    /// The ECS hierarchy is untouched — this is purely a view filter
    /// applied by `paint_hierarchy`.
    pub fn is_hierarchy_collapsed(&self, id: NodeId) -> bool {
        self.hierarchy_collapsed.contains(&id)
    }

    /// Flip the collapsed flag for `id`. Called by `apply_event` when
    /// the chevron companion NodeId is clicked.
    pub fn toggle_hierarchy_collapsed(&mut self, id: NodeId) {
        if !self.hierarchy_collapsed.insert(id) {
            self.hierarchy_collapsed.remove(&id);
        }
    }

    /// M14.6B: republish the set of NodeIds currently displayed as
    /// hierarchy rows. The painter calls this once per frame after
    /// registering its row hit-rects. Cleared and replaced wholesale
    /// — no merge — so stale ids from the previous frame (e.g. an
    /// entity that despawned) drop out automatically.
    pub fn set_hierarchy_row_ids(&mut self, ids: std::collections::BTreeSet<NodeId>) {
        self.hierarchy_row_ids = ids;
    }

    /// True iff `id` is currently displayed as a hierarchy row.
    /// Covers both fixture HIER_* ids and live ECS-bridge ids in
    /// one query — dispatch uses this to decide whether to start a
    /// drag candidate on Primary Down (replaces the static range
    /// check that used to silently reject every live row).
    /// Mark `id` as a multi-line text widget. Enter on it inserts a
    /// literal newline instead of submitting + blurring (the default
    /// behavior for single-line TextInputs). Idempotent; callers
    /// (TextArea widget, note body populators) re-mark every frame
    /// during populate.
    pub fn mark_multiline_text(&mut self, id: NodeId) {
        self.multiline_text_ids.insert(id);
    }

    /// True iff `id` was previously marked via `mark_multiline_text`.
    pub fn is_multiline_text(&self, id: NodeId) -> bool {
        self.multiline_text_ids.contains(&id)
    }

    pub fn is_hierarchy_row(&self, id: NodeId) -> bool {
        self.hierarchy_row_ids.contains(&id)
    }

    // ── Painter layers-panel row drag (W3 T3.8) — mirror of the hierarchy
    //    drag, but the dispatch never mutates structure (the painter tool
    //    owns the LayerStack and resolves the emitted `PainterLayerReparent`).

    /// Snapshot of the in-progress painter layer-row drag, if any.
    pub fn painter_layer_drag(&self) -> Option<HierarchyDragState> {
        self.painter_layer_drag
    }

    /// Begin a painter layer-row drag (Primary Down on a row). `active`
    /// flips once the cursor passes the 5px threshold (see
    /// [`Self::update_painter_layer_drag`]).
    pub fn begin_painter_layer_drag(
        &mut self,
        dragged: NodeId,
        down_x: f32,
        down_y: f32,
        timestamp_ns: u128,
    ) {
        self.painter_layer_drag = Some(HierarchyDragState {
            dragged,
            down_x,
            down_y,
            cursor_x: down_x,
            cursor_y: down_y,
            active: false,
            down_timestamp_ns: timestamp_ns,
        });
    }

    /// Advance the drag cursor; flips `active` once past the 5px threshold.
    pub fn update_painter_layer_drag(&mut self, cursor_x: f32, cursor_y: f32) {
        if let Some(d) = self.painter_layer_drag.as_mut() {
            d.cursor_x = cursor_x;
            d.cursor_y = cursor_y;
            let dx = cursor_x - d.down_x;
            let dy = cursor_y - d.down_y;
            if (dx * dx + dy * dy) > 25.0 {
                d.active = true;
            }
        }
    }

    /// Take the in-progress drag (cleared on Up).
    pub fn end_painter_layer_drag(&mut self) -> Option<HierarchyDragState> {
        self.painter_layer_drag.take()
    }

    /// W4 §3 — record a Curves/Levels control-point drag computed by the
    /// dispatch (`x`/`y` already normalized to `[0, 1]`). The panel drains it
    /// with [`Self::take_curve_point_drag`] and forwards to its tool's
    /// `set_curve_point`.
    pub fn set_curve_point_drag(&mut self, parent: NodeId, channel: u8, index: u8, x: f32, y: f32) {
        self.curve_point_drag =
            Some((parent, channel, index, x.clamp(0.0, 1.0), y.clamp(0.0, 1.0)));
    }

    /// Take the pending curve-point drag `(parent, channel, index, x01, y01)`,
    /// if any. Drained once per frame by the curve editor panel.
    pub fn take_curve_point_drag(&mut self) -> Option<(NodeId, u8, u8, f32, f32)> {
        self.curve_point_drag.take()
    }

    /// Republish the set of `NodeId`s that are currently painter layer rows.
    /// The layers panel calls this each frame with `painter_layer_widget_id(
    /// layer, Row)` for every visible row.
    pub fn set_painter_layer_row_ids(&mut self, ids: std::collections::BTreeSet<NodeId>) {
        self.painter_layer_row_ids = ids;
    }

    /// Is `id` a draggable painter layer row?
    pub fn is_painter_layer_row(&self, id: NodeId) -> bool {
        self.painter_layer_row_ids.contains(&id)
    }

    /// Mark `id` as a picker swatch — a [`crate::widget::ColorSwatch`] whose
    /// Down opens the Blender picker seeded with its `widget_color`. Panels
    /// call this as they paint each picker swatch (idempotent; ids are stable).
    pub fn register_picker_swatch(&mut self, id: NodeId) {
        self.picker_swatch_ids.insert(id);
    }

    /// Does a Down on `id` open the canonical color picker?
    pub fn is_picker_swatch(&self, id: NodeId) -> bool {
        self.picker_swatch_ids.contains(&id)
    }

    /// Depth in the parent tree (0 = root). Capped at 32 levels as
    /// a cycle guard — re-parent operations already reject cycles,
    /// so the cap should be unreachable in practice.
    pub fn hierarchy_depth_of(&self, id: NodeId) -> u32 {
        let mut depth = 0u32;
        let mut cur = id;
        for _ in 0..32 {
            match self.hierarchy_parent.get(&cur).copied() {
                Some(p) => {
                    depth += 1;
                    cur = p;
                }
                None => return depth,
            }
        }
        depth
    }

    /// True if `candidate` is a (strict or non-strict) descendant of
    /// `ancestor`. Used to reject DnD operations that would create a
    /// cycle (you can't drop a parent inside its own child).
    pub fn hierarchy_is_descendant_of(&self, candidate: NodeId, ancestor: NodeId) -> bool {
        if candidate == ancestor {
            return true;
        }
        let mut cur = candidate;
        for _ in 0..32 {
            match self.hierarchy_parent.get(&cur).copied() {
                Some(p) if p == ancestor => return true,
                Some(p) => cur = p,
                None => return false,
            }
        }
        false
    }

    /// Re-parent `child` under `parent`. Pass `None` to detach the
    /// child to the root level. Returns `false` (no change) when the
    /// operation would create a cycle (parent is a descendant of
    /// child).
    pub fn hierarchy_set_parent(&mut self, child: NodeId, parent: Option<NodeId>) -> bool {
        match parent {
            None => {
                self.hierarchy_parent.remove(&child);
                true
            }
            Some(p) => {
                if p == child || self.hierarchy_is_descendant_of(p, child) {
                    return false;
                }
                self.hierarchy_parent.insert(child, p);
                true
            }
        }
    }

    /// Mutate a NumberInput's committed value programmatically (e.g.
    /// from a linked Slider drag). Re-syncs the buffer to the new
    /// formatted value when the input is **not** focused; if it is
    /// focused, the user's edit is preserved.
    pub fn set_number_value(&mut self, id: NodeId, new_value: f64) {
        let focused = self.focus_id == Some(id);
        // Audit re-pass fix: while a drag-slider is actively
        // scrubbing this very NumberInput, do NOT overwrite
        // `last_committed` from external (snapshot) writes. The drag's
        // Up handler commits `last_committed = value` at release — if
        // an in-flight `set_number_value` clobbered the rollback
        // anchor on every frame, Esc mid-drag would revert to the
        // last-applied dragged value (effectively a no-op), defeating
        // audit fix #2.
        let dragging_this = matches!(self.number_input_drag.as_ref(), Some(d) if d.id == id);
        if let Some(InteractiveState::NumberInput {
            value,
            buffer,
            last_committed,
            ..
        }) = self.states.get_mut(&id)
        {
            *value = new_value;
            if !dragging_this {
                *last_committed = new_value;
            }
            if !focused {
                buffer.clear();
                use std::fmt::Write;
                let _ = write!(buffer, "{}", format_number(new_value));
            }
        }
    }

    /// BulkSelect (T2.0): render a NumberInput as "Mixed" by blanking its
    /// displayed buffer (the underlying `value` / `last_committed` stay
    /// the primary's, so a focus-then-blur with no typing reverts cleanly
    /// — `commit_number_buffer` parses the empty buffer, fails, and
    /// restores `last_committed` WITHOUT emitting `ValueChanged`, so the
    /// diverging values are never stomped). No-op while this input is
    /// focused or being drag-scrubbed so it doesn't fight live input.
    pub fn blank_number_input(&mut self, id: NodeId) {
        if self.focus_id == Some(id) {
            return;
        }
        if matches!(self.number_input_drag.as_ref(), Some(d) if d.id == id) {
            return;
        }
        if let Some(InteractiveState::NumberInput { buffer, .. }) = self.states.get_mut(&id) {
            buffer.clear();
        }
    }
}
