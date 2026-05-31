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

    /// Initialize the BlenderPicker's palette swatches. Caller passes
    /// the seed colors (typically `default_palette()`).
    pub fn init_blender_palette(&mut self, parent: NodeId, swatches: Vec<ColorValue>) {
        self.blender_palettes.insert(parent, swatches);
    }

    /// Read the BlenderPicker's current palette swatches. Returns
    /// `None` if `init_blender_palette` was never called for `parent`.
    pub fn blender_palette(&self, parent: NodeId) -> Option<&[ColorValue]> {
        self.blender_palettes.get(&parent).map(|v| v.as_slice())
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

    /// Append `color` to the BlenderPicker's palette. No-op if the
    /// palette wasn't initialized OR is already at the static cap
    /// (24 entries — matches the pre-registered swatch hit slots so
    /// every visible swatch has a clickable hit rect).
    pub fn blender_palette_push(&mut self, parent: NodeId, color: ColorValue) {
        const PALETTE_CAP: usize = 27;
        if let Some(palette) = self.blender_palettes.get_mut(&parent)
            && palette.len() < PALETTE_CAP
        {
            palette.push(color);
        }
    }

    /// Remove the swatch at `idx` from the BlenderPicker's palette.
    /// Returns true if a swatch was actually removed.
    pub fn blender_palette_remove(&mut self, parent: NodeId, idx: usize) -> bool {
        if let Some(palette) = self.blender_palettes.get_mut(&parent)
            && idx < palette.len()
        {
            palette.remove(idx);
            return true;
        }
        false
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
                    // OKLCH derives directly from the current sRGB value
                    // (no retained anchor): overwrite one normalized
                    // L/C/H/A channel and convert back. `norm` is the
                    // 0..1 slider/chip value matching the painter's
                    // `oklch_norm_channels` display scale.
                    let rgba = crate::widget::oklch_set_channel(
                        value.rgba,
                        channel_idx,
                        norm.clamp(0.0, 1.0),
                    );
                    *value = ColorValue::from_rgba8(rgba[0], rgba[1], rgba[2], rgba[3]);
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
