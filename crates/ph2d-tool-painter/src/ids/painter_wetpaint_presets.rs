//! The **Wet Painter preset** widgets (W1, `docs/Painter/NewFeaturesWetPaint/02 §5`): the preset chip,
//! the Favorite checkbox, Save as New / Save as Default / Reset, the Size px volume (Water is
//! [`super::PAINTER_WETPAINT_WATER`]), the quick-value chips of the two live volumes, and the
//! volume-policy chip. Opacity, Pressure and Length have NO reader in Wet Paint yet (gate
//! `a_opacidade_nao_tem_leitor_no_wet_paint`; W6/W5): no control until they do. The disk work behind the preset buttons is the shell bridge's
//! (`ph2d-app-painter/src/painter_bridge_presets.rs`): the tool only records the request.

use ph2d_a11y::NodeId;
use ph2d_tool_registry::{hash_node_id, hash_node_id_runtime};

/// The **Preset** chip (Wet Paint replaces the media-preset row with it). `SelectOption(_, relativo)`.
pub const PAINTER_WETPAINT_PRESET: NodeId = hash_node_id("painter_brush.wetpaint_preset");

/// One preset in the open Preset popover, keyed by its library path (stable across list changes) and
/// by the popover section it is listed in (`0` Favorites · `2` its group). Since 09/10 a preset is listed
/// ONCE (a favourite leaves its group; there is no Recent section — the owner: the menu repeated names).
#[must_use]
pub fn painter_wetpaint_preset_option_id(secao: u8, relativo: &str) -> NodeId {
    hash_node_id_runtime(&format!(
        "painter_brush.wetpaint_preset_opt.{secao}.{relativo}"
    ))
}

/// A section title in the open Preset popover (Favorites / a group) — painted disabled.
#[must_use]
pub fn painter_wetpaint_preset_title_id(k: usize) -> NodeId {
    hash_node_id_runtime(&format!("painter_brush.wetpaint_preset_title.{k}"))
}

/// **Favorite** checkbox — the active preset in/out of the Favorites list.
pub const PAINTER_WETPAINT_PRESET_FAVORITE: NodeId =
    hash_node_id("painter_brush.wetpaint_preset_favorite");
/// **Save as New** — the brush as it is now, as a new preset in the user folder, same group.
pub const PAINTER_WETPAINT_PRESET_SAVE_NEW: NodeId =
    hash_node_id("painter_brush.wetpaint_preset_save_new");
/// **Save as Default** — the brush as it is now replaces the active preset (a factory preset gets a
/// user copy on top; Reset removes it).
pub const PAINTER_WETPAINT_PRESET_SAVE_DEFAULT: NodeId =
    hash_node_id("painter_brush.wetpaint_preset_save_default");
/// **Reset** — the active preset back to its file (and to the factory file when the user's copy
/// covered one).
pub const PAINTER_WETPAINT_PRESET_RESET: NodeId =
    hash_node_id("painter_brush.wetpaint_preset_reset");

/// The preset buttons (Click → the tool records the request; the bridge does the disk work).
pub const PAINTER_WETPAINT_PRESET_BUTTONS: [NodeId; 4] = [
    PAINTER_WETPAINT_PRESET_FAVORITE,
    PAINTER_WETPAINT_PRESET_SAVE_NEW,
    PAINTER_WETPAINT_PRESET_SAVE_DEFAULT,
    PAINTER_WETPAINT_PRESET_RESET,
];

/// **Size px** volume — the DIAMETER the artist sees (the cursor ring); the tool stores half of it.
pub const PAINTER_WETPAINT_VOL_SIZE: NodeId = hash_node_id("painter_brush.wetpaint_vol_size");
/// The quick-value chip of each live volume, in volume order: Size px · Water.
pub const PAINTER_WETPAINT_VOL_QUICK: [NodeId; 2] = [
    hash_node_id("painter_brush.wetpaint_vol_quick_size"),
    hash_node_id("painter_brush.wetpaint_vol_quick_water"),
];

/// Quick values of the Size px volume (diameter, px) — Rebelle's quick list, in px, from the smallest
/// diameter the engine paints (`2 × BRUSH_SIZE_MIN_PX`: a «1» would be a promise it cannot keep).
pub const VOL_QUICK_SIZE_PX: [f32; 12] = [
    2.0, 3.0, 5.0, 10.0, 20.0, 40.0, 60.0, 100.0, 150.0, 200.0, 300.0, 500.0,
];
/// Quick values of the 0..1 volume (Water), shown as percent.
pub const VOL_QUICK_UNIT: [f32; 10] = [0.01, 0.03, 0.05, 0.1, 0.2, 0.3, 0.5, 0.7, 0.85, 1.0];

/// The quick values of volume `v` (index into [`PAINTER_WETPAINT_VOL_QUICK`]).
#[must_use]
pub fn vol_quick_values(v: usize) -> &'static [f32] {
    if v == 0 {
        &VOL_QUICK_SIZE_PX
    } else {
        &VOL_QUICK_UNIT
    }
}

/// Option `i` of volume `v`'s quick-value popover.
#[must_use]
pub fn painter_wetpaint_vol_quick_option_id(v: usize, i: u8) -> NodeId {
    hash_node_id_runtime(&format!("painter_brush.wetpaint_vol_quick_opt.{v}.{i}"))
}

/// **On preset change** chip — the volume policy (`ph2d_brush_presets::Politica`, wire = its index in
/// `Politica::TODAS`).
pub const PAINTER_WETPAINT_PRESET_POLICY: NodeId =
    hash_node_id("painter_brush.wetpaint_preset_policy");
/// Number of policies.
pub const PAINTER_WETPAINT_PRESET_POLICY_COUNT: u8 = 3;

/// Option `i` of the policy popover.
#[must_use]
pub fn painter_wetpaint_policy_option_id(i: u8) -> NodeId {
    hash_node_id_runtime(&format!("painter_brush.wetpaint_preset_policy_opt.{i}"))
}

/// Every dropdown chip of the block (registered as `Dropdown` by the panel's populate).
pub const PAINTER_WETPAINT_PRESET_DROPDOWNS: [NodeId; 4] = [
    PAINTER_WETPAINT_PRESET,
    PAINTER_WETPAINT_PRESET_POLICY,
    PAINTER_WETPAINT_VOL_QUICK[0],
    PAINTER_WETPAINT_VOL_QUICK[1],
];
