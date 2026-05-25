//! Editor chrome state on [`WidgetStore`] — clipboard outbox, scene
//! name, tool toggles, tooltips, collapsed flags, context menu,
//! section outlines, notes, picker target, widget colors,
//! radius scale.
//!
//! Extracted from [`super`] (Track D6). All these are short
//! key/value mutators that don't fit the Blender / panel / widget-
//! accessor / drag clusters but share the "side-table on WidgetStore"
//! pattern.

use super::WidgetStore;
use crate::interaction::types::{ContextMenuRequest, NoteData};
use ph2d_a11y::NodeId;

impl WidgetStore {
    /// Drain the pending clipboard-copy text (set by a Cmd+C / Cmd+X
    /// dispatch); shell should call once per frame and write to OS
    /// clipboard when non-None.
    pub fn take_clipboard_copy(&mut self) -> Option<String> {
        self.pending_clipboard_copy.take()
    }

    pub fn set_clipboard_copy(&mut self, text: String) {
        self.pending_clipboard_copy = Some(text);
    }

    /// Drain the pending clipboard-paste request; shell should read
    /// the OS clipboard and call `apply_clipboard_paste` with the
    /// result when non-None.
    pub fn take_clipboard_paste_request(&mut self) -> Option<NodeId> {
        self.pending_clipboard_paste.take()
    }

    pub fn set_clipboard_paste_request(&mut self, id: NodeId) {
        self.pending_clipboard_paste = Some(id);
    }

    pub fn current_scene_name(&self) -> &str {
        &self.current_scene_name
    }

    pub fn set_current_scene_name(&mut self, name: impl Into<String>) {
        self.current_scene_name = name.into();
    }

    pub fn tool_space_local(&self) -> bool {
        self.tool_space_local
    }

    pub fn set_tool_space_local(&mut self, local: bool) {
        self.tool_space_local = local;
    }

    pub fn tool_view_mode(&self) -> u8 {
        self.tool_view_mode
    }

    pub fn set_tool_view_mode(&mut self, mode: u8) {
        self.tool_view_mode = mode % 3;
    }

    /// Register a tooltip string for `id`. Called by `populate`
    /// passes or directly inside painters; read by the hover-tooltip
    /// pass via [`Self::tooltip_for`]. Empty strings are treated as
    /// no-tooltip and removed from the table.
    pub fn set_tooltip(&mut self, id: NodeId, text: impl Into<String>) {
        let s = text.into();
        if s.is_empty() {
            self.tooltips.remove(&id);
        } else {
            self.tooltips.insert(id, s);
        }
    }

    /// Lookup the tooltip for a widget id. Returns `None` if the id
    /// has no registered tooltip.
    pub fn tooltip_for(&self, id: NodeId) -> Option<&str> {
        self.tooltips.get(&id).map(|s| s.as_str())
    }

    /// `true` iff the section/panel at `id` is currently collapsed.
    /// Missing entries default to expanded — newly-registered
    /// sections start open without any setup.
    pub fn is_collapsed(&self, id: NodeId) -> bool {
        self.collapsed.get(&id).copied().unwrap_or(false)
    }

    /// Set the collapsed state for a section/panel. `true` collapses,
    /// `false` expands.
    pub fn set_collapsed(&mut self, id: NodeId, collapsed: bool) {
        self.collapsed.insert(id, collapsed);
    }

    /// Flip the collapsed state for `id`. Convenience for click
    /// handlers — equivalent to
    /// `set_collapsed(id, !is_collapsed(id))`.
    pub fn toggle_collapsed(&mut self, id: NodeId) {
        let was = self.is_collapsed(id);
        self.collapsed.insert(id, !was);
    }

    /// Open a right-click context menu at `(x, y)` with the given
    /// `kind`. Replaces any previously-open menu (only one menu
    /// can be visible at a time).
    pub fn open_context_menu(&mut self, request: ContextMenuRequest) {
        self.context_menu = Some(request);
    }

    /// Close any currently-open context menu. Snapshots the request
    /// into `last_context_menu` so `apply_event` can still read the
    /// menu's `kind` when handling the item-click that triggered
    /// the close (the click → Click event arrives AFTER the close).
    pub fn close_context_menu(&mut self) {
        if let Some(req) = self.context_menu.take() {
            self.last_context_menu = Some(req);
        }
    }

    /// Read the currently-open context menu request, if any.
    pub fn context_menu(&self) -> Option<ContextMenuRequest> {
        self.context_menu
    }

    /// Read the most recently closed context-menu request — used by
    /// `apply_event` to recover the original `kind` when routing an
    /// item Click into a side-table mutation. Cleared by
    /// `consume_last_context_menu` once the click has been applied.
    pub fn last_context_menu(&self) -> Option<ContextMenuRequest> {
        self.last_context_menu
    }

    /// Take + clear the last-context-menu snapshot. Called by
    /// `apply_event` after it has routed the click.
    pub fn consume_last_context_menu(&mut self) -> Option<ContextMenuRequest> {
        self.last_context_menu.take()
    }

    /// Read the outline-color index for a section header id (0..4
    /// referencing the highlighter palette). `None` = no outline.
    pub fn section_outline_color(&self, section: NodeId) -> Option<u8> {
        self.section_outline_color.get(&section).copied()
    }

    /// Set the outline-color index for a section. Pass `None` to
    /// clear (the "No outline" menu item).
    pub fn set_section_outline_color(&mut self, section: NodeId, color: Option<u8>) {
        match color {
            Some(c) => {
                self.section_outline_color.insert(section, c);
            }
            None => {
                self.section_outline_color.remove(&section);
            }
        }
    }

    /// Read the per-panel note list. Returns an empty slice when no
    /// notes have been created for the panel.
    pub fn notes_for_panel(&self, panel: NodeId) -> &[NoteData] {
        self.notes_per_panel
            .get(&panel)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    /// Append a new sticky-note to the panel's list. Caller passes the
    /// highlighter color index + the optional section-anchor to the
    /// panel's note list. Cap at 12 notes per panel
    /// to keep paint bounded. `before_section: Some(i)` makes the
    /// painter slot the note immediately above `SECTION_IDS[i]`;
    /// `None` appends at the bottom.
    pub fn notes_push(&mut self, panel: NodeId, color_idx: u8, before_section: Option<u8>) {
        const CAP: usize = 12;
        let list = self.notes_per_panel.entry(panel).or_default();
        if list.len() >= CAP {
            return;
        }
        list.push(NoteData {
            color_idx,
            title: format!("Note {}", list.len() + 1),
            body: String::new(),
            before_section,
        });
    }

    /// Update an existing note's color index.
    pub fn note_set_color(&mut self, panel: NodeId, index: usize, color_idx: u8) {
        if let Some(list) = self.notes_per_panel.get_mut(&panel)
            && let Some(note) = list.get_mut(index)
        {
            note.color_idx = color_idx.min(4);
        }
    }

    /// Currently-active color picker target. The floating
    /// BlenderColorPicker is hidden when this is `None`.
    pub fn picker_target(&self) -> Option<NodeId> {
        self.picker_target
    }

    /// Open the picker editing the color at `target`. Pass `None`
    /// to hide the picker. The caller is responsible for seeding
    /// `widget_colors[target]` if it doesn't yet have a value.
    pub fn set_picker_target(&mut self, target: Option<NodeId>) {
        self.picker_target = target;
        // Opening the picker → it's the most recently summoned panel.
        if target.is_some() {
            // Picker is keyed by INSP_BLENDER_PICKER in the z-order
            // (canonical floating-panel id). Hard-coded here rather
            // than re-exporting the screens::hero::ids const into the
            // interaction crate, since the picker is the single
            // floating panel for the whole editor.
            const INSP_BLENDER_PICKER: NodeId = NodeId(380);
            self.bump_panel_z(INSP_BLENDER_PICKER);
        }
    }

    /// Read the current color of a color-target widget. Returns
    /// `None` when no color has been stored for `id`. Default
    /// colors are seeded lazily by callers (e.g. `apply_event`
    /// inserts a neutral gray when first opening the picker).
    pub fn widget_color(&self, id: NodeId) -> Option<[u8; 4]> {
        self.widget_colors.get(&id).copied()
    }

    /// Set the current color of a color-target widget. Called by
    /// the picker each frame to mirror its edits into the
    /// target's stored color, and by `apply_event` to seed the
    /// default when the picker first opens for that target.
    pub fn set_widget_color(&mut self, id: NodeId, rgba: [u8; 4]) {
        self.widget_colors.insert(id, rgba);
    }

    /// Editor-wide corner-radius scale (1.0 = canonical, 0.0 =
    /// sharp). Painters that want to honor the user's theme preset
    /// multiply their `Radius::*.px()` by this factor.
    pub fn radius_scale(&self) -> f32 {
        self.radius_scale
    }

    pub fn set_radius_scale(&mut self, scale: f32) {
        self.radius_scale = scale.max(0.0);
    }

    /// Current rail-button size preset (Small / Medium / Large) — set
    /// from the Themes menu (2026-05-24).
    pub fn rail_button_size(&self) -> crate::widget::RailButtonSize {
        self.rail_button_size
    }

    pub fn set_rail_button_size(&mut self, size: crate::widget::RailButtonSize) {
        self.rail_button_size = size;
    }
}
