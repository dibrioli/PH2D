//! TopBar chip → human name map, split out of `topbar/mod.rs` so the painter
//! module stays under its LOC cap. Pure id→label lookup, used by the hover
//! tooltip and any debug/a11y readout of a TopBar pill.

use crate::ids;
use ph2d_a11y::NodeId;
use ph2d_i18n::tr;

/// The display name for a TopBar chip id, or `None` if the id isn't a chip.
pub(super) fn topbar_chip_name(id: NodeId) -> Option<&'static str> {
    Some(match id {
        x if x == ids::TOPBAR_THEME => tr("chrome.topbar.name.theme_ph2d"),
        x if x == ids::TOPBAR_PROJECT => tr("chrome.topbar.name.project"),
        x if x == ids::TOPBAR_SAVE => tr("chrome.topbar.name.save"),
        x if x == ids::TOPBAR_SAVE_AS => tr("chrome.topbar.name.save_as"),
        x if x == ids::TOPBAR_OPEN => tr("chrome.topbar.name.open"),
        x if x == ids::TOPBAR_IMAGE_TOOLS => tr("chrome.topbar.name.image_tools"),
        x if x == ids::TOPBAR_AUDIO_MIXER => tr("chrome.topbar.name.audio_mixer"),
        x if x == ids::TOPBAR_AUDIO_EDITOR => tr("chrome.topbar.name.audio_editor"),
        x if x == ids::TOPBAR_PLAY_BUTTON => tr("chrome.topbar.name.play"),
        x if x == ids::TOPBAR_PAUSE => tr("chrome.topbar.name.pause"),
        x if x == ids::TOPBAR_RESET => tr("chrome.topbar.name.reset"),
        x if x == ids::TOPBAR_RIGHT_LAYERS => tr("chrome.topbar.name.layers"),
        x if x == ids::TOPBAR_RIGHT_ASSETS => tr("chrome.topbar.name.assets"),
        x if x == ids::TOPBAR_RIGHT_SCRIPT => tr("chrome.topbar.name.script"),
        x if x == ids::TOPBAR_WIDGET_GALLERY => tr("chrome.topbar.name.widget_gallery"),
        x if x == ids::TOPBAR_PHYSICS => tr("chrome.topbar.name.physics"),
        x if x == ids::TOPBAR_TOKENS => tr("chrome.topbar.name.tokens"),
        x if x == ids::TOPBAR_AUTHORED => tr("chrome.topbar.name.authored_ui"),
        x if x == ids::TOPBAR_SCULPT3D => tr("chrome.topbar.name.sculpt_3d"),
        x if x == ids::TOPBAR_MODEL3D => tr("chrome.topbar.name.n3d_model"),
        x if x == ids::TOPBAR_GRID_SETTINGS => tr("chrome.topbar.name.grid_settings"),
        x if x == ids::TOPBAR_SETTINGS => tr("chrome.topbar.name.settings"),
        x if x == ids::IMAGE_ACTION_TRIM => tr("chrome.topbar.name.trim_transparency"),
        x if x == ids::IMAGE_ACTION_MAKE_SQUARE => tr("chrome.topbar.name.make_square"),
        x if x == ids::IMAGE_ACTION_BGREMOVAL => tr("chrome.topbar.name.bg_removal"),
        x if x == ids::IMAGE_ACTION_REAL_SIZE => tr("chrome.topbar.name.real_size"),
        x if x == ids::IMAGE_ACTION_PADDING => tr("chrome.topbar.name.padding"),
        x if x == ids::IMAGE_ACTION_COLOR_EQUALIZATION => {
            tr("chrome.topbar.name.color_equalization")
        }
        x if x == ids::IMAGE_ACTION_EQUALIZE_SIZES => tr("chrome.topbar.name.equalize_sizes"),
        x if x == ids::IMAGE_ACTION_RASTERIZE => tr("chrome.topbar.name.rasterize"),
        x if x == ids::IMAGE_ACTION_UPSCALE => tr("chrome.topbar.name.upscale"),
        x if x == ids::IMAGE_ACTION_PAINTER => tr("chrome.topbar.name.painter"),
        x if x == ids::TOPBAR_LEFT_BACKDROP => tr("chrome.topbar.name.left_backdrop"),
        x if x == ids::TOPBAR_RIGHT_BACKDROP => tr("chrome.topbar.name.right_backdrop"),
        x if x == ids::TOPBAR_IMAGE_TOOLS_BACKDROP => tr("chrome.topbar.name.image_tools_backdrop"),
        _ => return None,
    })
}
