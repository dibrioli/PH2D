//! BlenderColorPicker methods on [`WidgetStore`].
//!
//! Extracted from [`super`] (Track D4). The picker's retained state
//! (palette swatches, drag offset / anchor, HSV anchor, eyedropper
//! pending target) and the per-channel mutators all live here as a
//! separate `impl WidgetStore` block, so the main `state/mod.rs`
//! impl stays focused on core widget store ops.
//!
//! Requires `WidgetStore`'s fields to be `pub(super)` (bumped by D4
//! prep) so this file can access them from a sibling submodule.

use super::WidgetStore;
use crate::interaction::util::hsv_to_color_value;
use crate::interaction::{InteractiveState, types::BlenderHitKind};
use crate::widget::{ChannelMode, InterpolationMode};
use ph2d_a11y::NodeId;
use ph2d_tokens::ColorValue;

#[allow(dead_code)]
const _: () = ();

impl WidgetStore {
    /// Tag a hex `TextInput` widget as belonging to a `BlenderPicker`.
    /// Caller is responsible for both ids being pre-registered.
    pub fn link_blender_hex(&mut self, parent: NodeId, hex: NodeId) {
        self.hex_to_blender_parent.insert(hex, parent);
    }

    pub fn blender_hex_parent(&self, hex: NodeId) -> Option<NodeId> {
        self.hex_to_blender_parent.get(&hex).copied()
    }

    /// Tag a channel `NumberInput` chip as belonging to a
    /// `BlenderPicker` at channel index `idx` (0..=3). On commit,
    /// dispatch reads `idx` to know which RGBA / HSVA dimension to
    /// rewrite.
    pub fn link_blender_channel(&mut self, parent: NodeId, chip: NodeId, idx: u8) {
        self.blender_channel_chip.insert(chip, (parent, idx));
    }

    pub fn blender_channel_chip(&self, chip: NodeId) -> Option<(NodeId, u8)> {
        self.blender_channel_chip.get(&chip).copied()
    }

    /// Read the BlenderPicker state at `id`. Returns `None` for
    /// non-picker widgets.
    pub fn blender_picker(
        &self,
        id: NodeId,
    ) -> Option<(ColorValue, ChannelMode, InterpolationMode, usize)> {
        match self.states.get(&id) {
            Some(InteractiveState::BlenderPicker {
                value,
                channel_mode,
                interpolation,
                active_palette,
                ..
            }) => Some((*value, *channel_mode, *interpolation, *active_palette)),
            _ => None,
        }
    }

    /// Initialize the BlenderPicker's palette set with a single palette named "Palette" holding the
    /// seed `swatches` (typically `default_palette()`). The user adds more via the New-palette button.
    pub fn init_blender_palette(&mut self, parent: NodeId, swatches: Vec<ColorValue>) {
        self.blender_palettes.insert(
            parent,
            vec![super::NamedPalette {
                name: "Palette".to_string(),
                swatches,
            }],
        );
    }

    /// The active palette index for `parent`, clamped to the palette set so a stale index (after a
    /// delete) never points past the end. `0` when the picker has no state yet.
    fn active_palette_idx(&self, parent: NodeId) -> usize {
        let count = self.blender_palettes.get(&parent).map_or(0, Vec::len);
        self.blender_picker(parent)
            .map_or(0, |t| t.3)
            .min(count.saturating_sub(1))
    }

    /// Read the ACTIVE palette's swatches. `None` if the picker was never seeded.
    pub fn blender_palette(&self, parent: NodeId) -> Option<&[ColorValue]> {
        let active = self.active_palette_idx(parent);
        self.blender_palettes
            .get(&parent)?
            .get(active)
            .map(|p| p.swatches.as_slice())
    }

    /// Read the full named-palette SET (for the tab strip). `None` if never seeded.
    pub fn blender_palette_set(&self, parent: NodeId) -> Option<&[super::NamedPalette]> {
        self.blender_palettes.get(&parent).map(Vec::as_slice)
    }

    /// Append a fresh empty palette named "Palette N" and make it active. No-op if never seeded.
    pub fn blender_new_palette(&mut self, parent: NodeId) {
        let new_idx = self.blender_palettes.get_mut(&parent).map(|set| {
            let n = set.len() + 1;
            set.push(super::NamedPalette {
                name: format!("Palette {n}"),
                swatches: Vec::new(),
            });
            set.len() - 1
        });
        if let Some(idx) = new_idx {
            self.set_blender_active_palette(parent, idx);
        }
    }

    /// Delete the active palette (keeping at least one) and clamp the active index onto the survivor.
    pub fn blender_delete_active_palette(&mut self, parent: NodeId) {
        let active = self.active_palette_idx(parent);
        let new_active = self.blender_palettes.get_mut(&parent).and_then(|set| {
            (set.len() > 1).then(|| {
                set.remove(active);
                active.min(set.len() - 1)
            })
        });
        if let Some(idx) = new_active {
            self.set_blender_active_palette(parent, idx);
        }
    }

    /// Rename the active palette (empty names are ignored — the tab needs a label).
    pub fn blender_rename_active_palette(&mut self, parent: NodeId, name: &str) {
        let active = self.active_palette_idx(parent);
        if !name.trim().is_empty()
            && let Some(p) = self
                .blender_palettes
                .get_mut(&parent)
                .and_then(|s| s.get_mut(active))
        {
            p.name = name.trim().to_string();
        }
    }

    /// Select palette `idx` as active (clamped to the set).
    pub fn blender_select_palette(&mut self, parent: NodeId, idx: usize) {
        let count = self.blender_palettes.get(&parent).map_or(0, Vec::len);
        if count > 0 {
            self.set_blender_active_palette(parent, idx.min(count - 1));
        }
    }

    /// Write the active-palette index into the picker's [`InteractiveState::BlenderPicker`] state.
    fn set_blender_active_palette(&mut self, parent: NodeId, idx: usize) {
        if let Some(InteractiveState::BlenderPicker { active_palette, .. }) =
            self.states.get_mut(&parent)
        {
            *active_palette = idx;
        }
    }

    /// Read the BlenderPicker's drag offset (dx, dy). Defaults to
    /// (0, 0) if no drag has happened yet.
    pub fn blender_picker_offset(&self, parent: NodeId) -> (f32, f32) {
        self.blender_picker_offset
            .get(&parent)
            .copied()
            .unwrap_or((0.0, 0.0))
    }

    pub fn set_blender_picker_offset(&mut self, parent: NodeId, dx: f32, dy: f32) {
        self.blender_picker_offset.insert(parent, (dx, dy));
    }

    /// Begin a picker drag at cursor `(px, py)`. Snapshots the
    /// current offset so Move events can compute new_offset =
    /// offset_at_down + (cursor − down_cursor).
    pub fn begin_blender_drag(&mut self, parent: NodeId, cursor_x: f32, cursor_y: f32) {
        let (off_x, off_y) = self.blender_picker_offset(parent);
        self.blender_drag_anchor = Some((parent, cursor_x, cursor_y, off_x, off_y));
        // Dragged panel → topmost in z-order.
        self.bump_panel_z(parent);
    }

    pub fn blender_drag_anchor(&self) -> Option<(NodeId, f32, f32, f32, f32)> {
        self.blender_drag_anchor
    }

    /// Update only the cursor coordinates in the drag anchor (used by
    /// the incremental drag model — each move re-anchors so the next
    /// move applies a fresh delta to the post-clamp offset).
    pub fn update_blender_drag_cursor(&mut self, cursor_x: f32, cursor_y: f32) {
        if let Some((parent, _, _, off_x, off_y)) = self.blender_drag_anchor {
            self.blender_drag_anchor = Some((parent, cursor_x, cursor_y, off_x, off_y));
        }
    }

    pub fn end_blender_drag(&mut self) {
        self.blender_drag_anchor = None;
    }

    pub fn eyedropper_pending(&self) -> Option<NodeId> {
        self.eyedropper_pending
    }

    pub fn set_eyedropper_pending(&mut self, parent: Option<NodeId>) {
        self.eyedropper_pending = parent;
    }

    /// Flag a palette import/export request for the host to service (set by the picker dispatch on an
    /// Import/Export button click).
    pub fn set_palette_io_pending(
        &mut self,
        parent: NodeId,
        kind: crate::interaction::PaletteIoKind,
    ) {
        self.palette_io_pending = Some((parent, kind));
    }

    /// Take the pending palette import/export request (the host drains it once, opens a file dialog,
    /// then applies via [`Self::blender_palette_replace`] / reads [`Self::blender_palette`]).
    pub fn take_palette_io_pending(
        &mut self,
    ) -> Option<(NodeId, crate::interaction::PaletteIoKind)> {
        self.palette_io_pending.take()
    }

    /// Append `color` to the ACTIVE palette. No-op if never seeded OR already at the static cap (27 —
    /// matches the pre-registered swatch hit slots so every visible swatch has a clickable hit rect).
    pub fn blender_palette_push(&mut self, parent: NodeId, color: ColorValue) {
        const PALETTE_CAP: usize = 27;
        let active = self.active_palette_idx(parent);
        if let Some(p) = self
            .blender_palettes
            .get_mut(&parent)
            .and_then(|s| s.get_mut(active))
            && p.swatches.len() < PALETTE_CAP
        {
            p.swatches.push(color);
        }
    }

    /// Remove the swatch at `idx` from the ACTIVE palette. Returns true if one was removed.
    pub fn blender_palette_remove(&mut self, parent: NodeId, idx: usize) -> bool {
        let active = self.active_palette_idx(parent);
        if let Some(p) = self
            .blender_palettes
            .get_mut(&parent)
            .and_then(|s| s.get_mut(active))
            && idx < p.swatches.len()
        {
            p.swatches.remove(idx);
            return true;
        }
        false
    }

    /// IMPORT a palette: append a NEW named palette (file name + swatches, capped to the 27 hit slots)
    /// and make it active — so an import adds to the set rather than clobbering the current palette.
    /// Seeds the set if the picker was never initialised.
    pub fn blender_import_palette(
        &mut self,
        parent: NodeId,
        name: &str,
        mut swatches: Vec<ColorValue>,
    ) {
        const PALETTE_CAP: usize = 27;
        swatches.truncate(PALETTE_CAP);
        let name = if name.trim().is_empty() {
            "Imported".to_string()
        } else {
            name.trim().to_string()
        };
        let last = {
            let set = self.blender_palettes.entry(parent).or_default();
            set.push(super::NamedPalette { name, swatches });
            set.len() - 1
        };
        self.set_blender_active_palette(parent, last);
    }

    /// Read the retained HSV anchor (h, s) the picker uses to
    /// preserve hue + saturation across V→0 transitions where the
    /// RGBA representation would otherwise lose them. Both in 0..1.
    pub fn blender_hsv_anchor(&self, id: NodeId) -> Option<(f32, f32)> {
        match self.states.get(&id) {
            Some(InteractiveState::BlenderPicker { hsv_h, hsv_s, .. }) => Some((*hsv_h, *hsv_s)),
            _ => None,
        }
    }

    /// Mutate the BlenderPicker's value. Auto-updates the retained
    /// (h, s) anchor when the new color is chromatic (S>0, V>0); for
    /// gray/black inputs the anchor is preserved so the user's chosen
    /// hue doesn't reset to red on a V=0 click.
    pub fn set_blender_value(&mut self, id: NodeId, new_value: ColorValue) {
        if let Some(InteractiveState::BlenderPicker {
            value,
            hsv_h,
            hsv_s,
            ..
        }) = self.states.get_mut(&id)
        {
            *value = new_value;
            let (h, s, v, _) = crate::widget::rgba_to_hsv(new_value.rgba);
            if s > 1e-3 && v > 1e-3 {
                *hsv_h = h;
                *hsv_s = s;
            }
        }
    }

    /// Mutate the BlenderPicker's value AND override the retained
    /// (h, s) anchor explicitly. Used by the SV-rect / hue-strip
    /// dispatchers, which know the canonical H or S even when the
    /// resulting RGBA collapses (e.g. picking V=0 → all-zero RGBA).
    pub fn set_blender_value_with_hsv(
        &mut self,
        id: NodeId,
        new_value: ColorValue,
        h: f32,
        s: f32,
    ) {
        if let Some(InteractiveState::BlenderPicker {
            value,
            hsv_h,
            hsv_s,
            ..
        }) = self.states.get_mut(&id)
        {
            *value = new_value;
            // Clamp instead of `rem_euclid`: the user-picked H from
            // a hue-strip click may equal 1.0 at the right edge; we
            // want the thumb to stay at the right rather than
            // wrapping to 0.0 (left edge).
            *hsv_h = h.clamp(0.0, 1.0);
            *hsv_s = s.clamp(0.0, 1.0);
        }
    }

    /// Mutate the BlenderPicker's channel mode (RGB↔HSV).
    pub fn set_blender_channel_mode(&mut self, id: NodeId, mode: ChannelMode) {
        if let Some(InteractiveState::BlenderPicker { channel_mode, .. }) = self.states.get_mut(&id)
        {
            *channel_mode = mode;
        }
    }

    /// Mutate the BlenderPicker's interpolation (Linear↔Perceptual).
    pub fn set_blender_interpolation(&mut self, id: NodeId, mode: InterpolationMode) {
        if let Some(InteractiveState::BlenderPicker { interpolation, .. }) =
            self.states.get_mut(&id)
        {
            *interpolation = mode;
        }
    }

    /// Mutate a single channel of the BlenderPicker's RGBA value.
    /// `channel_idx` 0..=3 = R/G/B/A (or H/S/V/A in HSV mode — caller
    /// is responsible for converting before calling). `norm` must be in
    /// [0.0, 1.0].
    pub fn set_blender_channel(&mut self, id: NodeId, channel_idx: u8, norm: f32) {
        if let Some(InteractiveState::BlenderPicker {
            value,
            channel_mode,
            hsv_h,
            hsv_s,
            ..
        }) = self.states.get_mut(&id)
        {
            let byte = (norm.clamp(0.0, 1.0) * 255.0).round() as u8;
            match *channel_mode {
                ChannelMode::Rgb => {
                    if let Some(slot) = value.rgba.get_mut(channel_idx as usize) {
                        *slot = byte;
                    }
                    let [r, g, b, a] = value.rgba;
                    *value = ColorValue::from_rgba8(r, g, b, a);
                    // Refresh retained anchor when the new RGB is
                    // chromatic (else keep what we had so the H chip
                    // doesn't spuriously reset on RGB-mode edits).
                    let (h, s, v, _) = crate::widget::rgba_to_hsv(value.rgba);
                    if s > 1e-3 && v > 1e-3 {
                        *hsv_h = h;
                        *hsv_s = s;
                    }
                }
                ChannelMode::Hsv => {
                    // Use retained (h, s) as the canonical HSV basis
                    // — see `apply_blender_channel_value` for the
                    // why. V + A from RGBA are recoverable.
                    let (_, _, v_rgba, a_rgba) = crate::widget::rgba_to_hsv(value.rgba);
                    let mut h = *hsv_h;
                    let mut s = *hsv_s;
                    let mut v = v_rgba;
                    let mut a = a_rgba;
                    match channel_idx {
                        0 => h = norm.clamp(0.0, 1.0),
                        1 => s = norm.clamp(0.0, 1.0),
                        2 => v = norm.clamp(0.0, 1.0),
                        3 => a = norm.clamp(0.0, 1.0),
                        _ => {}
                    }
                    *value = hsv_to_color_value(h, s, v, a);
                    *hsv_h = h;
                    *hsv_s = s;
                }
                ChannelMode::Oklch => {
                    // Edit one OKLCH channel off the value's STORED oklch
                    // (round-trip-free) and rebuild via `from_oklch`, so
                    // the chroma/hue the user dialed in persist even when
                    // the sRGB form is gray or gamut-clamped (no slider
                    // snap-back; hue survives chroma≈0).
                    *value = crate::widget::oklch_set_channel(
                        value.oklch,
                        channel_idx,
                        norm.clamp(0.0, 1.0),
                    );
                    // Keep the HSV anchor in sync so switching back to
                    // HSV/SV-rect doesn't snap the hue.
                    let (h, s, v, _) = crate::widget::rgba_to_hsv(value.rgba);
                    if s > 1e-3 && v > 1e-3 {
                        *hsv_h = h;
                        *hsv_s = s;
                    }
                }
            }
        }
    }
}

// Silence unused-imports warning when only some types are referenced
// in this submodule's impl block.
#[allow(dead_code)]
const _BLENDER_HIT_KIND_REFERENCE: Option<BlenderHitKind> = None;
