//! Linear/Perceptual + RGB/HSV segmented toggles.

use super::state::{BlenderColorPicker, ChannelMode, InterpolationMode};
use crate::widget::{RadioGroup, RadioOption, RadioOrientation, paint_radio_group_with_labels};
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_text::TextSystem;
use ph2d_tokens::Theme;
use ph2d_vector::VectorScene;

pub fn paint_interpolation_toggle(
    cp: &BlenderColorPicker,
    rect: Rect,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) {
    let group = RadioGroup::new(
        NodeId(0),
        "Interpolation",
        vec![
            RadioOption::new(NodeId(0), "linear", "Linear"),
            RadioOption::new(NodeId(0), "perceptual", "Perceptual"),
        ],
    )
    .orientation(RadioOrientation::Segmented)
    .selected(match cp.interpolation {
        InterpolationMode::Linear => "linear",
        InterpolationMode::Perceptual => "perceptual",
    });
    paint_radio_group_with_labels(&group, rect, scene, text_system, theme);
}

pub fn paint_channel_toggle(
    cp: &BlenderColorPicker,
    rect: Rect,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) {
    let group = RadioGroup::new(
        NodeId(0),
        "Channel mode",
        vec![
            RadioOption::new(NodeId(0), "rgb", "RGB"),
            RadioOption::new(NodeId(0), "hsv", "HSV"),
        ],
    )
    .orientation(RadioOrientation::Segmented)
    .selected(match cp.channel_mode {
        ChannelMode::Rgb => "rgb",
        ChannelMode::Hsv => "hsv",
    });
    paint_radio_group_with_labels(&group, rect, scene, text_system, theme);
}
