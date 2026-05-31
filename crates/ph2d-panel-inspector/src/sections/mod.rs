//! M14 live Inspector section painters (Name / Visibility / Transform /
//! Render Source / Color & Tint / Sprite Sheet). Migrated to the panel
//! crate in ADR-0029 Phase C.1.
//!
//! Split into per-section submodules (Wave §T2.1, `architecture_panel_loc_cap`):
//! the shared import surface is re-exported `pub(crate)` here so each
//! submodule opens with a single `use super::*;`. No logic moved — every
//! section painter is verbatim from the pre-split `sections.rs`.

pub(crate) use crate::state::current_display_unit;
pub(crate) use ph2d_a11y::NodeId;
pub(crate) use ph2d_editor_core::icons::IconId;
pub(crate) use ph2d_editor_core::ids;
pub(crate) use ph2d_editor_core::interaction::{HitIndex, InteractiveState, WidgetStore};
pub(crate) use ph2d_editor_core::paint::{
    fill_rounded_rect, paint_icon, paint_text, rect_to_vello, resolve, stroke_rounded_rect,
};
pub(crate) use ph2d_editor_core::screens::hero::{InspectorSpriteInfo, InspectorSpriteSource};
pub(crate) use ph2d_editor_core::widget::panel_chrome::{
    SECTION_BOTTOM_PAD_PX, SECTION_LABEL_TO_CONTROL_PX, paint_segmented_group_adaptive,
};
pub(crate) use ph2d_editor_core::widget::showcase::{active_index, read_number_input};
pub(crate) use ph2d_editor_core::widget::{
    BitmaskGrid32, Button, ButtonKind, ButtonState, Checkbox, CheckboxState, CheckboxValue,
    ColorSwatch, IconButtonStyle, IconGlyph, NumberInput, Rect2Editor, Rect2Layout, SectionHeader,
    SliderState, SwatchSize, TabItem, Tabs, TabsVariant, TextInput, TextInputState,
    paint_bitmask_grid32, paint_button, paint_checkbox, paint_color_swatch, paint_icon_button,
    paint_number_input_with_buffer, paint_rect2_editor_with_state, paint_section_header,
    paint_slider_with_chip, paint_tabs, paint_text_input_with_buffer,
};
pub(crate) use ph2d_editor_core::zones::Rect;
pub(crate) use ph2d_text::TextSystem;
pub(crate) use ph2d_tokens::{ColorToken, ROW_H_PX, Radius, Spacing, Theme, TypeToken};
pub(crate) use ph2d_vector::{Color as VelloColor, VectorScene};

mod color_tint;
mod identity;
mod material_blend;
pub(crate) mod ordering;
mod render_source;
mod sampling;
mod sprite_sheet;
mod transform;
mod visibility;

pub(crate) use color_tint::paint_color_tint_section;
pub(crate) use identity::{paint_entity_name_row, paint_visibility_row};
pub(crate) use material_blend::paint_material_blend_section;
pub(crate) use ordering::paint_ordering_section;
pub(crate) use render_source::paint_render_source_section;
pub(crate) use sampling::paint_sampling_section;
pub(crate) use sprite_sheet::paint_sprite_sheet_section;
pub(crate) use transform::paint_transform_section;
pub(crate) use visibility::paint_visibility_section;
