//! [`BlenderSubIds`] — the bundle of sub-control [`NodeId`]s the picker painter
//! registers in the [`crate::interaction::HitIndex`].
//!
//! Pure data, split out of [`super::paint`] (HR-18 file-LOC cap) so the painter
//! orchestrator stays focused on layout. `NodeId(0)` slots are skipped (no hit
//! rect registered), so callers can opt sub-controls in/out by leaving them zero.

use ph2d_a11y::NodeId;

/// All sub-control [`NodeId`]s that
/// [`super::paint::paint_blender_color_picker_with_store`] needs to register in
/// the [`crate::interaction::HitIndex`]. Stack-allocated; passed by reference so
/// callers don't pay for extra arguments. `NodeId(0)` slots are skipped (no hit
/// rect registered).
///
/// Build via [`BlenderSubIds::zeroed`] then fill the fields the caller wants.
#[derive(Clone, Copy, Debug)]
pub struct BlenderSubIds {
    pub parent: NodeId,
    pub wheel: NodeId,
    pub value_slider: NodeId,
    pub interp_linear: NodeId,
    pub interp_perceptual: NodeId,
    pub channel_rgb: NodeId,
    pub channel_hsv: NodeId,
    pub channel_oklch: NodeId,
    /// Channel slider ids 0..4 (R/H/L, G/S/C, B/V/H, A).
    pub channels: [NodeId; 4],
    /// Channel value chip `NumberInput` ids 0..4 (mirror channels).
    pub channels_num: [NodeId; 4],
    /// Hex TextInput id.
    pub hex: NodeId,
    /// "+ swatch" button id (appends current value to palette).
    pub add_swatch: NodeId,
    /// Palette Import / Export button ids (host file dialog → `.gpl`/`.hex`/`.ase`/`.aco`).
    pub import_palette: NodeId,
    pub export_palette: NodeId,
    /// Palette-select dropdown chip id (click → toggle the popover).
    pub palette_dropdown: NodeId,
    /// Named-palette popover-option ids (up to 8; index = palette position). Reused as the
    /// dropdown's row ids. Entries with id == 0 are skipped.
    pub palette_tabs: [NodeId; 8],
    /// "+ palette" (New), "R" (Rename) and "delete palette" (×) button ids.
    pub new_palette: NodeId,
    pub rename_palette: NodeId,
    pub delete_palette: NodeId,
    /// Active-palette rename `TextInput` field id (Enter commits the new name).
    pub palette_name: NodeId,
    /// Eyedropper button id.
    pub eyedropper: NodeId,
    /// Drag-handle bar id (at top of picker — drag to reposition).
    pub drag_handle: NodeId,
    /// Close (×) button id — dismisses the floating picker.
    pub close: NodeId,
    /// Palette swatch ids (up to 27). Entries with id == 0 are
    /// skipped. The first 12 cover the default palette; the rest
    /// cover user "+ swatch" additions (capped to keep the array
    /// fixed-size).
    pub swatches: [NodeId; 27],
    /// Color-Harmonies scheme selector segment ids (index = [`super::Harmony::ALL`] position).
    pub harmony_schemes: [NodeId; 7],
    /// Derived harmony partner swatch ids (up to [`super::Harmony::MAX_COLORS`]).
    pub harmony_swatches: [NodeId; 4],
    /// "Add harmony to palette" button id.
    pub harmony_add: NodeId,
}

impl BlenderSubIds {
    /// Construct with all zero (disabled) ids.
    pub const fn zeroed() -> Self {
        Self {
            parent: NodeId(0),
            wheel: NodeId(0),
            value_slider: NodeId(0),
            interp_linear: NodeId(0),
            interp_perceptual: NodeId(0),
            channel_rgb: NodeId(0),
            channel_hsv: NodeId(0),
            channel_oklch: NodeId(0),
            channels: [NodeId(0); 4],
            channels_num: [NodeId(0); 4],
            hex: NodeId(0),
            add_swatch: NodeId(0),
            import_palette: NodeId(0),
            export_palette: NodeId(0),
            palette_dropdown: NodeId(0),
            palette_tabs: [NodeId(0); 8],
            new_palette: NodeId(0),
            rename_palette: NodeId(0),
            delete_palette: NodeId(0),
            palette_name: NodeId(0),
            eyedropper: NodeId(0),
            drag_handle: NodeId(0),
            close: NodeId(0),
            swatches: [NodeId(0); 27],
            harmony_schemes: [NodeId(0); 7],
            harmony_swatches: [NodeId(0); 4],
            harmony_add: NodeId(0),
        }
    }
}
