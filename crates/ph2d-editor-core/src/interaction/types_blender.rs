//! Value types describing a hit inside the shared BlenderColorPicker.
//!
//! Split from [`super::types`] (HR-18 file-LOC cap) when the Color-Harmonies
//! variants pushed it over 700 — the picker's own sub-control vocabulary is a
//! cohesive responsibility, so it earns a sibling module (mirror of
//! [`super::types_menu`]). Re-exported by `types` so `types::BlenderHitKind`
//! and the mod-level `interaction::BlenderHitKind` paths keep working unchanged.

/// A pending palette file-I/O request the picker dispatch raises and the host (shell) services by
/// opening a file dialog: [`Import`](Self::Import) loads + REPLACES the active palette,
/// [`Export`](Self::Export) saves it. The format is chosen from the picked file's extension.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum PaletteIoKind {
    /// Load a palette file and replace the active swatches.
    Import,
    /// Save the active swatches to a palette file.
    Export,
}

/// Which sub-control of a [`super::InteractiveState::BlenderPicker`] a
/// [`super::InteractiveState::BlenderHit`] points at.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum BlenderHitKind {
    Wheel,
    ValueSlider,
    InterpolationLinear,
    InterpolationPerceptual,
    ChannelRgb,
    ChannelHsv,
    /// Switch the channel sliders to OKLCH (L/C/H/Alpha).
    ChannelOklch,
    /// One of the 4 horizontal channel sliders (R/G/B/A, H/S/V/A, or
    /// L/C/H/A). Index 0..3: 0 = R/H/L, 1 = G/S/C, 2 = B/V/H, 3 = A.
    ChannelSlider(u8),
    /// The hex `#RRGGBBAA` text input field.
    Hex,
    /// One swatch in the active palette. Index into the picker's
    /// store-side palette (see [`super::WidgetStore::blender_palette`]).
    /// Left-click picks the swatch; right-click removes it.
    PaletteSwatch(u8),
    /// "+ swatch" button at the end of the palette grid; clicking
    /// appends the picker's current value to the palette.
    AddSwatch,
    /// A palette TAB in the named-palette strip. Index into the picker's palette set
    /// ([`WidgetStore::blender_palette_set`]); clicking selects it as the active palette.
    PaletteTab(u8),
    /// "+ palette" button — appends a fresh empty "Palette N" and makes it active.
    NewPalette,
    /// "delete palette" button — removes the active palette (keeping at least one).
    DeletePalette,
    /// Palette-select dropdown chip — clicking toggles the popover listing every named
    /// palette (replaces the old cramped one-line tab strip; index = palette position).
    PaletteDropdown,
    /// A Color-Harmonies scheme segment. Index into [`crate::widget::Harmony::ALL`]; clicking
    /// selects that scheme (view-state — the partners are derived, never stored).
    HarmonyScheme(u8),
    /// A derived harmony partner swatch. Index into [`crate::widget::harmony_partners`] of the
    /// current value + scheme; clicking sets it as the picker's value.
    HarmonySwatch(u8),
    /// "Add harmony to palette" — appends every derived partner to the active palette.
    HarmonyAdd,
    /// "R" rename button next to the dropdown — toggles the inline rename field (a
    /// `TextInput` whose Enter commits the new active-palette name).
    RenamePalette,
    /// "×" close button at the top-right of the floating picker — clears the picker
    /// target (`WidgetStore::set_picker_target(None)`), dismissing the popover.
    Close,
    /// "Import" palette button — clicking flags a host file-dialog request
    /// (`WidgetStore::set_palette_io_pending`) to load a `.gpl`/`.hex`/`.ase`/
    /// `.aco` as a NEW named palette.
    ImportPalette,
    /// "Export" palette button — flags a host file-dialog request to save the
    /// active palette's swatches in the format the chosen extension selects.
    ExportPalette,
    /// Eyedropper button next to the hex field. Clicking enters
    /// pixel-pick mode (the host samples the next click's color from
    /// the rendered scene).
    Eyedropper,
    /// Drag handle bar at the top of the picker — Down begins a
    /// drag, Move updates the picker offset, Up ends it.
    DragHandle,
    /// Bottom-right resize gripper. Down begins a resize; Move
    /// adjusts the parent's stored `(dw, dh)`; Up ends it.
    ResizeHandle,
    /// Bottom-LEFT resize gripper. Mirror of [`ResizeHandle`]. Down
    /// begins a BL-mode resize; Move adjusts the parent's stored
    /// `(dw, dh)` AND `(dx, dy)` so the right edge stays put while
    /// the left edge follows the cursor; Up ends it. Lets the user
    /// grab the panel from either bottom corner.
    ResizeHandleBl,
    /// M14.6A: eye icon on a hierarchy row — toggles the entity's
    /// `Visibility` component. Parent NodeId on the `BlenderHit` is
    /// the row's id; dispatcher sets `HeroScreen.pending_visibility_toggle`
    /// for the host to drain and apply on `SimWorld`.
    VisibilityToggle,
}
