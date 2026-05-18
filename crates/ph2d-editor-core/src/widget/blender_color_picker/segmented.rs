//! RGB/HSV segmented toggle (Linear/Perceptual was removed when the
//! picker was simplified to the web-standard SV-rect + hue-strip
//! layout — `InterpolationMode` is still in the state model but
//! has no UI).

use super::state::{BlenderColorPicker, ChannelMode};
use crate::widget::{RadioGroup, RadioOption, RadioOrientation, paint_radio_group_with_labels};
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_text::TextSystem;
use ph2d_tokens::Theme;
use ph2d_vector::VectorScene;

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
