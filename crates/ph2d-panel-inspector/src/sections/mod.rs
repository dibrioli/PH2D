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
    fill_rounded_rect, paint_icon, paint_text, paint_text_block, rect_to_vello, resolve,
    stroke_rounded_rect,
};
pub(crate) use ph2d_editor_core::screens::hero::{InspectorSpriteInfo, InspectorSpriteSource};
pub(crate) use ph2d_editor_core::widget::panel_chrome::{
    SECTION_BOTTOM_PAD_PX, SECTION_LABEL_TO_CONTROL_PX, paint_segmented_group_adaptive,
};
pub(crate) use ph2d_editor_core::widget::showcase::{paint_section_separator, read_number_input};
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

/// **§12 Sockets / Named Anchors** (ADR-0072) — a seção nascida em 2026-08-21.
pub(crate) mod anchor_mount_row;
pub(crate) mod anchors;
pub(crate) mod anim;
pub(crate) mod anim_rows;
mod color_tint;
/// A linha `Emissive` (plano `docs/Sprite_projeto/18` W8) — irmã do `render_source`, que está no
/// tecto de LOC.
mod emissive_row;
mod identity;
pub(crate) mod joint;
mod joint_pair_rows;
mod material_blend;
pub(crate) mod ordering;
mod physics;
/// A face de CORPO do §11 — as rows de quem tem `RigidBody` + `Collider`.
/// Irmã do `physics.rs` pelos caps de painel: com TRÊS faces, `physics.rs` fica
/// sendo o cabeçalho e o roteador.
mod physics_body;
/// As portas da face VAZIA do §11 (docs dele) — irmão pelo cap de 600 LOC.
mod physics_doors;
mod physics_join_rows;
/// W-PartFace: a 3ª face do §11 — o que se mostra de um `Collider` que não é
/// corpo (uma PEÇA de um corpo composto).
mod physics_part;
mod physics_rows;
/// ⚠️ `pub(crate)` só para o `PLAYER_ROW_COUNT` do `lib.rs` — o gate de seam
/// afirma que cobre a tabela INTEIRA, e um oráculo que itera a própria lista
/// que testa encolhe junto com ela.
pub(crate) mod player;
mod render_source;
/// O par `Format` — irmão do `render_source` pelo cap de LOC.
mod render_source_precision;
/// ⚠️ `pub(crate)` só para a régua do card (`card_pitch`), que o `lib.rs`
/// re-exporta para o gate de GEOMETRIA da §14.
pub(crate) mod rows;
mod sampling;
/// **§5 9-Slice** — a seção que a spec declarou em 2026-05 e que nasceu em 2026-08-21.
pub(crate) mod slice_grid;
pub(crate) mod slice_nine;
mod sprite_sheet;
mod transform;
mod visibility;
mod wheel;

pub(crate) use color_tint::paint_color_tint_section;
pub(crate) use identity::{paint_entity_name_row, paint_visibility_row};
pub(crate) use joint::paint_joint_section;
pub use joint::paste_label;
pub(crate) use material_blend::paint_material_blend_section;
pub(crate) use ordering::paint_ordering_section;
pub use physics::bake_label;
pub(crate) use physics::paint_physics_section;
pub use physics_join_rows::rig_button_label;
pub(crate) use player::paint_player_section;
pub(crate) use render_source::paint_render_source_section;
/// ⚠️ Público para o gate `the_filter_segmented_tells_the_truth_about_what_renders` (shell), que é
/// a única crate que vê o painel e o `ph2d-render` ao mesmo tempo.
pub use sampling::FILTER_LABELS;
pub(crate) use sampling::paint_sampling_section;
pub(crate) use sprite_sheet::paint_sprite_sheet_section;
pub(crate) use transform::paint_transform_section;
pub(crate) use visibility::paint_visibility_section;
pub(crate) use wheel::paint_wheel_section;

/// **A PORTA ÚNICA do cabeçalho de uma secção do Inspector.**
///
/// As doze secções construíam a mesma cadeia à mão (`new` + `collapsible` + agora o `open_t`), e
/// a terceira metade — o `t` VIVO da dobra — é precisamente a que um sítio novo esquece: ele
/// compila, pinta, responde ao rato, e fica **silenciosamente discreto** no meio de onze vizinhas
/// que rodam. Com uma porta, a secção treze nasce viva.
///
/// ⚠️ **Lê o `is_collapsed` outra vez de propósito.** Quase todo chamador já tem um `collapsed`
/// local (ele decide se o CORPO é pintado, que é outra pergunta) — passá-lo aqui seria pedir ao
/// chamador que mantivesse duas respostas coerentes, e é assim que nasce um cabeçalho a dizer
/// aberto sobre um corpo escondido. As duas leituras são puras e do mesmo quadro.
pub(crate) fn section_header(store: &WidgetStore, id: NodeId, label: &str) -> SectionHeader {
    SectionHeader::new(id, label)
        .collapsible(!store.is_collapsed(id))
        .open_t(store.section_open_live(id))
}
