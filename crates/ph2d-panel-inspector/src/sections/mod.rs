//! M14 live Inspector section painters (Name / Visibility / Transform /
//! Render Source / Color & Tint / Sprite Sheet). Migrated to the panel
//! crate in ADR-0029 Phase C.1.
//!
//! Split into per-section submodules (Wave §T2.1, `architecture_panel_loc_cap`):
//! the shared import surface is re-exported `pub(crate)` here so each
//! submodule opens with a single `use super::*;`. No logic moved — every
//! section painter is verbatim from the pre-split `sections.rs`.

/// ⭐⭐⭐ **A ALTURA DE UM BOTÃO DE ACÇÃO DO INSPECTOR — uma, e só uma.**
///
/// ⛔⛔ **Ela estava declarada QUINZE vezes**, cada cópia com um comentário a afirmar *«igual à das
/// irmãs»* — e **nada** que o verificasse. É a MESMA forma que o `CHECKBOX_BOX_PX = 18` pagou em
/// 2026-09-21 (cinco cópias, uma curada, quatro com a frase a ficar falsa em silêncio), e a que o
/// `SwatchSize::Md` pagou em 22/09. *Uma frase de comentário não é uma lei: só uma PORTA é.*
///
/// ⚠️ **Ela é MAIOR que a altura de uma fileira** (`ROW_H_PX = 22`), e isso é declarado e não um
/// acidente: um botão de acção não é um campo. ⛔ O que NÃO é declarado é a divergência que estava
/// ao lado — ver o [`ALTURA_DE_CAMPO`].
pub(crate) const ALTURA_DE_BOTAO: f32 = 30.0; // LITERAL-PX-OK: a ÚNICA declaração desta grandeza

/// ⭐⭐⭐ **A ALTURA DE UM CAMPO DO INSPECTOR — que é a da FILEIRA, e não um número próprio.**
///
/// ⛔⛔⛔ **Medido 2026-09-22: ela estava declarada em DUAS versões que se contradiziam**, as duas
/// com um comentário a chamar-se *«a altura de campo do Inspector»* — `24` no `emissive_row.rs` e
/// no `slice_nine.rs`, `22` no `script.rs`. *Duas respostas à mesma pergunta, e a que o artista vê
/// é a do ficheiro em que ele calhou de estar a olhar.*
///
/// ⇒ a resposta é a da CASA, e esta porta delega nela para não haver uma terceira.
pub(crate) const ALTURA_DE_CAMPO: f32 = ph2d_tokens::ROW_H_PX;

pub(crate) use crate::ids;
pub(crate) use crate::state::{current_display_angle, current_display_unit};
pub(crate) use ph2d_a11y::NodeId;
pub(crate) use ph2d_editor_core::icons::IconId;
pub(crate) use ph2d_editor_core::ids as core_ids;
pub(crate) use ph2d_editor_core::interaction::{HitIndex, InteractiveState, WidgetStore};
pub(crate) use ph2d_editor_core::paint::{
    fill_rounded_rect, paint_text, paint_text_block, resolve,
};
pub(crate) use ph2d_editor_core::screens::hero::{InspectorSpriteInfo, InspectorSpriteSource};
pub(crate) use ph2d_editor_core::widget::panel_chrome::{
    SECTION_BOTTOM_PAD_PX, SECTION_LABEL_TO_CONTROL_PX,
};
pub(crate) use ph2d_editor_core::widget::showcase::read_number_input;
pub(crate) use ph2d_editor_core::widget::{
    BitmaskGrid32, Button, ButtonKind, ButtonState, Checkbox, CheckboxState, CheckboxValue,
    IconButtonStyle, IconGlyph, NumberInput, SectionHeader, SliderState, TextInput, TextInputState,
    paint_bitmask_grid32, paint_button, paint_checkbox, paint_icon_button,
    paint_number_input_with_buffer, paint_section_header, paint_slider_with_chip,
    paint_text_input_with_buffer,
};
pub(crate) use ph2d_editor_core::zones::Rect;
use ph2d_i18n::TextKey;
pub(crate) use ph2d_text::TextSystem;
pub(crate) use ph2d_tokens::{ColorToken, ROW_H_PX, Radius, Spacing, Theme, TypeToken};
pub(crate) use ph2d_vector::{Color as VelloColor, VectorScene};

/// ⭐ A secção COUNTER WATCH — a vigia do contador.
pub(crate) mod action_trigger;
/// ⭐⭐⭐ **A secção SIGNAL ACTIONS** (TOP-20 #5, W3) — a tabela nome → acção.
pub(crate) mod actions;
/// **§12 Sockets / Named Anchors** (ADR-0072) — a seção nascida em 2026-08-21.
pub(crate) mod anchor_mount_row;
pub(crate) mod anchors;
pub(crate) mod anim;
pub(crate) mod anim_rows;
/// ⭐⭐⭐ O SOM de um objecto (TOP-20 #4) — a fonte e as orelhas.
pub(crate) mod audio;
pub(crate) mod camera;
mod color_tint;
pub(crate) mod counter_watch;
/// A linha `Emissive` (plano `docs/Sprite_projeto/18` W8) — irmã do `render_source`, que está no
/// tecto de LOC.
mod emissive_row;
pub(crate) mod factory;
/// ⭐⭐⭐ A secção PARTICLES (TOP-20 #18) — ver o cabeçalho.
/// ⭐⭐⭐ **O HUD** (TOP-20 #20).
pub(crate) mod hud;
mod identity;
/// ⭐ **A seção COMPONENT** (ADR-0164 / F5) — o que esta cópia tem de diferente da receita.
pub(crate) mod instance;
/// ⭐⭐ **O bloco das peças ACRESCENTADAS** — irmão por assunto do `instance`, ver o cabeçalho de lá.
pub(crate) mod instance_added;
/// ⭐⭐ **O bloco das excepções SEM ALVO** — irmão por assunto do `instance`, ver o cabeçalho de lá.
pub(crate) mod instance_orphans;
/// ⭐⭐ **O bloco das peças RECUSADAS** — irmão por assunto do `instance`, ver o cabeçalho de lá.
pub(crate) mod instance_removed;
pub(crate) mod joint;
mod joint_pair_rows;
pub(crate) mod lifecycle;
mod material_blend;
pub(crate) mod ordering;
pub(crate) mod particles;
/// ⭐⭐⭐ **A secção do SEGUIDOR DE CAMINHO** (suplente #23) — ver o cabeçalho.
pub(crate) mod path_follow;
mod physics;
mod physics_area_rows;
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
pub(crate) mod physics_rows;
/// ⚠️ `pub(crate)` só para o `PLAYER_ROW_COUNT` do `lib.rs` — o gate de seam
/// afirma que cobre a tabela INTEIRA, e um oráculo que itera a própria lista
/// que testa encolhe junto com ela.
pub(crate) mod player;
pub(crate) mod projectile;
pub(crate) mod properties;
/// ⭐⭐⭐ A secção RAY SENSOR (suplente #21).
pub(crate) mod ray;
mod render_source;
/// O par `Format` — irmão do `render_source` pelo cap de LOC.
mod render_source_precision;
mod render_source_regiao;
/// ⚠️ `pub(crate)` só para a régua do card (`card_pitch`), que o `lib.rs`
/// re-exporta para o gate de GEOMETRIA da §14.
pub(crate) mod rows;
mod sampling;
/// ⭐⭐⭐ A secção SCRIPT (TOP-20 #16) — ver o cabeçalho.
pub(crate) mod script;
mod script_avisos;
/// ⭐⭐⭐ A secção SEQUENCE (TOP-20 #19) — a cutscene de um objecto. Ver o cabeçalho.
pub(crate) mod sequence;
/// ⭐⭐⭐ **O ABANÃO DA CÂMERA** (suplente #25) — *como* ela treme; ver o cabeçalho.
pub(crate) mod shake;
/// ⭐⭐⭐ **O EMISSOR DE ABANÃO** (suplente #25) — *ao ouvir o quê*; ver o cabeçalho.
pub(crate) mod shake_emitter;
/// **§5 9-Slice** — a seção que a spec declarou em 2026-05 e que nasceu em 2026-08-21.
pub(crate) mod slice_grid;
pub(crate) mod slice_nine;
mod sprite_sheet;
/// ⭐⭐⭐ A secção STATE MACHINE (TOP-20 #15) — ver o cabeçalho.
pub(crate) mod statemachine;
/// ⭐⭐⭐ **A secção TAGS** (TOP-20 #9, W3a) — a que o objecto É.
pub(crate) mod tags;
/// ⭐⭐⭐ **A secção TIMERS** (TOP-20 #2, W3) — o painel do primeiro relógio autorável.
pub(crate) mod timers;
pub(crate) mod topdown;
mod transform;
/// ⭐ **A secção TWEEN** (suplente #22) — irmã da `timers`, e com o molde dela.
pub(crate) mod tween;
/// ⚠️ O EDITOR de um tween — separado da MOLDURA dele pelo tecto de LOC do painel.
pub(crate) mod tween_editor;
pub(crate) mod weapon;
// ⚠️ O DESENHO de uma linha do Transform — separado da orquestração delas pelo tecto de LOC.
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

/// ⭐⭐ **A altura que este texto vai de facto ocupar**, nunca menos que uma linha.
///
/// ⚠️ **Ela mora aqui porque tem DOIS cartões** (auditoria de 2026-08-31, achado A2): o de
/// instância mediu-a e o de propriedades ficou a contar linhas — e o título dele é
/// `Properties of "<nome do artista>"`, que quebra pela mesma razão. *Uma cura escrita num dos
/// dois irmãos deixa o outro a repetir o defeito, e foi o que aconteceu em duas horas.*
///
/// ⚠️ **`max(line)`, e não a medida crua:** o `paint_text` desenha a partir do topo e o cartão
/// espaça em `line`; uma frase curta que medisse menos encolheria o ritmo das linhas seguintes.
pub(crate) fn text_h(
    text_system: &mut TextSystem,
    text: &str,
    font_size: f32,
    max_w: f32,
    line: f32,
) -> f32 {
    let h = text_system.layout(text, font_size, max_w).height();
    h.max(line)
}

/// A espessura da moldura de uma RANHURA de asset. LITERAL-PX-OK: contorno de 1 px, o mesmo que o
/// `stroke_rect` usa por omissão em todo o chrome.
pub(crate) const SLOT_BORDER_PX: f32 = 1.0;

/// O rótulo da ranhura da textura. ⏳ Migra com os irmãos quando o Fluent chegar (HR-15).
pub(crate) const STORAGE_LABEL: TextKey = TextKey::new("panel.inspector.render_source.storage");

/// O rótulo do tamanho de origem. ⏳ Migra com os irmãos quando o Fluent chegar (HR-15).
pub(crate) const SOURCE_SIZE_LABEL: TextKey = TextKey::new("panel.inspector.render_source.source");
