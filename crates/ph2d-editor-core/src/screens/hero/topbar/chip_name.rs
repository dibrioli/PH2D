//! TopBar chip → human name map, split out of `topbar/mod.rs` so the painter
//! module stays under its LOC cap. Pure id→label lookup, used by the hover
//! tooltip and any debug/a11y readout of a TopBar pill.

use crate::screens::hero::ids;
use ph2d_a11y::NodeId;

/// The display name for a TopBar chip id, or `None` if the id isn't a chip.
pub(super) fn topbar_chip_name(id: NodeId) -> Option<&'static str> {
    Some(match id {
        x if x == ids::TOPBAR_THEME => "Theme (PH2D)",
        x if x == ids::TOPBAR_PROJECT => "Project",
        x if x == ids::TOPBAR_SAVE => "Save",
        x if x == ids::TOPBAR_SAVE_AS => "Save As",
        x if x == ids::TOPBAR_OPEN => "Open",
        x if x == ids::TOPBAR_IMAGE_TOOLS => "Image Tools",
        x if x == ids::TOPBAR_AUDIO_MIXER => "Audio Mixer",
        x if x == ids::TOPBAR_PLAY_BUTTON => "Play",
        x if x == ids::TOPBAR_PAUSE => "Pause",
        x if x == ids::TOPBAR_RESET => "Reset",
        x if x == ids::TOPBAR_RIGHT_LAYERS => "Layers",
        x if x == ids::TOPBAR_RIGHT_ASSETS => "Assets",
        x if x == ids::TOPBAR_RIGHT_SCRIPT => "Script",
        x if x == ids::TOPBAR_WIDGET_GALLERY => "Widget Gallery",
        x if x == ids::TOPBAR_GRID_SETTINGS => "Grid Settings",
        x if x == ids::TOPBAR_SETTINGS => "Settings",
        x if x == ids::IMAGE_ACTION_TRIM => "Trim Transparency",
        x if x == ids::IMAGE_ACTION_MAKE_SQUARE => "Make Square",
        x if x == ids::IMAGE_ACTION_BGREMOVAL => "BG Removal",
        x if x == ids::IMAGE_ACTION_REAL_SIZE => "Real Size",
        x if x == ids::IMAGE_ACTION_PADDING => "Padding",
        x if x == ids::IMAGE_ACTION_COLOR_EQUALIZATION => "Color Equalization",
        x if x == ids::IMAGE_ACTION_EQUALIZE_SIZES => "Equalize Sizes",
        x if x == ids::IMAGE_ACTION_RASTERIZE => "Rasterize",
        x if x == ids::IMAGE_ACTION_UPSCALE => "Upscale",
        x if x == ids::IMAGE_ACTION_PAINTER => "Painter",
        x if x == ids::TOPBAR_LEFT_BACKDROP => "Left Backdrop",
        x if x == ids::TOPBAR_RIGHT_BACKDROP => "Right Backdrop",
        x if x == ids::TOPBAR_IMAGE_TOOLS_BACKDROP => "Image Tools Backdrop",
        _ => return None,
    })
}
