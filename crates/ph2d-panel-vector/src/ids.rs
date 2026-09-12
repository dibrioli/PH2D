//! Widget `NodeId`s for the Vector Style panel.
//!
//! Like the other panel crates, the ids stay defined in editor-core
//! (`ph2d_editor_core::ids`) — the layout + z-order walk + `node_id_collisions`
//! arch test all reference them, and re-defining them here would fork the
//! source of truth. This module is a convenience re-export so the panel's
//! internal modules (and the tool, via `handle_panel_event`) can write
//! `crate::ids::VECTOR_*`.

/// O 6º pill de modo: **Connect** — a linha que gruda em duas formas e as segue.
/// ⭐ **A família da secção *Pattern*** (plano 35, wave F): um id por `(tinta, controlo)`, para as
/// duas secções — a do preenchimento e a do traço — nascerem da MESMA lista.
pub use ph2d_editor_core::ids::TexPatKnob;
/// ⭐⭐⭐ **A secção BRUSH** (plano 36, W4) — knobs PRÓPRIOS, e não mais um alvo da família do padrão:
/// um pincel tem avanço e escala relativa, um padrão tem reticulado, fase e modo de repetição.
pub use ph2d_editor_core::ids::VECTOR_BRUSH_PICK_SHAPE;
/// **A SIMETRIA de desenho** (plano 25 W6.3): a seção + o par que arma + os quatro tipos + os
/// controles que só existem onde têm o que fazer (Segments no Radial, Fuse nos espelhos) + o
/// Apply. Os ids dos TIPOS não estão nomeados aqui: quem os resolve é a porta única
/// `ph2d_tool_vector::params::symmetry_kind_id`, e o `glob` do `ids` os traz.
pub use ph2d_editor_core::ids::VECTOR_SYM_APPLY;
pub use ph2d_editor_core::ids::{
    MAX_ENVELOPE_PRESETS, MAX_WIDTH_PRESETS, vector_envelope_preset_id, vector_width_preset_id,
};
pub use ph2d_editor_core::ids::{MAX_MORPH_STATES, VECTOR_MORPH_PREVIEW};
/// ⭐ **O ESQUELETO** (estudo 42 item 5): o 17º pill, o cabeçalho da seção e os controlos dela (conte-os nas tabelas abaixo, nunca aqui: esta linha dizia **cinco** com 29 ids re-exportados).
pub use ph2d_editor_core::ids::{
    MAX_SMART_CLIPS, VECTOR_BONE_ACT_CREATE, VECTOR_BONE_ACT_TRANSFORM, VECTOR_BONE_BEND_IDS,
    VECTOR_BONE_BIND, VECTOR_BONE_DEFORM_FAST, VECTOR_BONE_DEFORM_SMOOTH, VECTOR_BONE_EXPAND,
    VECTOR_BONE_FIELDS, VECTOR_BONE_IK_ADD, VECTOR_BONE_IK_CHAIN, VECTOR_BONE_IK_MIX,
    VECTOR_BONE_IK_REMOVE, VECTOR_BONE_IK_SOFTNESS, VECTOR_BONE_LENGTH, VECTOR_BONE_LIMIT_ADD,
    VECTOR_BONE_LIMIT_MAX, VECTOR_BONE_LIMIT_MIN, VECTOR_BONE_LIMIT_REMOVE, VECTOR_BONE_RELEASE,
    VECTOR_BONE_SMART_ADD, VECTOR_BONE_SMART_CLIP_IDS, VECTOR_BONE_SMART_FROM,
    VECTOR_BONE_SMART_PICK, VECTOR_BONE_SMART_REMOVE, VECTOR_BONE_SMART_TO, VECTOR_BONE_STRENGTH,
    VECTOR_BONE_VERBS, VECTOR_MODE_BONE, VECTOR_SECTION_BONE,
};
pub use ph2d_editor_core::ids::{MAX_TEXT_VARIATION_AXES, vector_text_axis_id};
/// **AS ÂNCORAS** (plano UI/UX W3): a seção + as duas fileiras de quatro chips.
pub use ph2d_editor_core::ids::{
    VECTOR_ANCHOR_H_CENTER, VECTOR_ANCHOR_H_END, VECTOR_ANCHOR_H_START, VECTOR_ANCHOR_H_STRETCH,
    VECTOR_ANCHOR_V_CENTER, VECTOR_ANCHOR_V_END, VECTOR_ANCHOR_V_START, VECTOR_ANCHOR_V_STRETCH,
};
pub use ph2d_editor_core::ids::{
    VECTOR_ARRANGE_DUPLICATE, VECTOR_ARRANGE_Z, VECTOR_BLEND_EXPAND, VECTOR_BLEND_RELEASE,
    VECTOR_BLEND_RESET_SPINE, VECTOR_BLEND_RUN, VECTOR_BLEND_STEPS, VECTOR_BOOL_APPLY,
    VECTOR_BOOL_LIVE_OFF, VECTOR_BOOL_LIVE_ON, VECTOR_CLOSE, VECTOR_COMPOUND_MAKE,
    VECTOR_COMPOUND_RELEASE, VECTOR_CONVERT_TO_CURVES, VECTOR_ENVELOPE_BEND,
    VECTOR_ENVELOPE_CLEAR_PINS, VECTOR_ENVELOPE_EXPAND, VECTOR_ENVELOPE_MESH,
    VECTOR_ENVELOPE_PERSPECTIVE, VECTOR_ENVELOPE_PINS, VECTOR_ENVELOPE_RELEASE,
    VECTOR_ENVELOPE_RUN, VECTOR_EXPAND_OFFSET, VECTOR_EXPAND_W_END, VECTOR_EXPAND_W_MID,
    VECTOR_EXPAND_W_POS, VECTOR_EXPAND_W_START, VECTOR_FILL_RULE_EVENODD, VECTOR_FILL_RULE_NONZERO,
    VECTOR_GRAD_ADD_POINT, VECTOR_GRAD_ADD_STOP, VECTOR_GRAD_ANGLE, VECTOR_GRAD_INFLUENCE,
    VECTOR_GRAD_JITTER, VECTOR_GRAD_REMOVE_POINT, VECTOR_GRAD_REMOVE_STOP, VECTOR_MORPH_RUN,
    VECTOR_MORPH_T, VECTOR_PANEL, VECTOR_PATH_CLOSE, VECTOR_PATH_JOIN, VECTOR_PATH_REVERSE,
    VECTOR_PATH_WELD, VECTOR_PIVOT_EDIT, VECTOR_RULERS_OFF, VECTOR_RULERS_ON,
    VECTOR_SNAP_CROSS_OFF, VECTOR_SNAP_CROSS_ON, VECTOR_SNAP_GUIDES_OFF, VECTOR_SNAP_GUIDES_ON,
    VECTOR_SNAP_OFF, VECTOR_SNAP_ON, VECTOR_SNAP_PATH_OFF, VECTOR_SNAP_PATH_ON,
    VECTOR_STROKE_PRESENT, VECTOR_TEXT_ALIGN_CENTER, VECTOR_TEXT_ALIGN_LEFT,
    VECTOR_TEXT_ALIGN_RIGHT, VECTOR_TEXT_FONT_DD, VECTOR_TEXT_FONT_IMPORT, VECTOR_TEXT_FONT_NEXT,
    VECTOR_TEXT_FONT_PREV, VECTOR_TEXT_LINE_HEIGHT, VECTOR_TEXT_SIZE, VECTOR_TEXT_TRACKING,
    VECTOR_TEXT_WEIGHT, VECTOR_TEXT_WRAP_AUTO, VECTOR_TEXT_WRAP_FIXED, VECTOR_TEXT_WRAP_W,
    VECTOR_TRANSFORM_R, VECTOR_TRANSFORM_RESIZE_BOX, VECTOR_VERT_AVERAGE, VECTOR_VERT_DELETE,
    VECTOR_VERT_SEL_SAME, VECTOR_VERT_SEL_SUBPATH, VECTOR_VERT_X, VECTOR_VERT_Y,
};
/// O 9º e 10º pills de modo: **Fillet** / **Chamfer** — arredondar / chanfrar quina por
/// clicar-e-arrastar (consolidam a alça do Node + o toggle da seção Vertex numa dupla).
pub use ph2d_editor_core::ids::{VECTOR_CUT_APPLY, VECTOR_CUT_DISCARD};
/// ⭐ **A APARÊNCIA do OBJECTO** (estudo 42 item 2): a seção, o slider de opacidade e o chip de
/// mistura + as linhas do popover dele.
pub use ph2d_editor_core::ids::{VECTOR_OBJ_BLEND, VECTOR_OBJ_OPACITY};
/// ⭐⭐⭐ **A PILHA DE APARÊNCIA** (estudo 42 item 4): os dois botões que acrescentam uma camada, os
/// cinco controlos de cada LINHA e as **quatro** propriedades da camada ABERTA (v21 acrescentou
/// ONDE ela desenha).
///
/// ⚠️ **Esta lista é CURADA** — um id que exista no `editor-core` e não venha aqui simplesmente não
/// se compila no painel. Foi a armadilha da wave anterior, e é a mesma família.
pub use ph2d_editor_core::ids::{
    VECTOR_PAINT_BLEND, VECTOR_PAINT_DILATE, VECTOR_PAINT_DX, VECTOR_PAINT_DY,
    VECTOR_PAINT_OPACITY, VECTOR_PAINT_WIDTH,
};

/// **Text on Path** (plano 22): a seção + os quatro controles + o par do offset.
pub use ph2d_editor_core::ids::{
    VECTOR_TEXTPATH_DETACH, VECTOR_TEXTPATH_FLIP, VECTOR_TEXTPATH_FLIP_OFF, VECTOR_TEXTPATH_LINK,
    VECTOR_TEXTPATH_OFFSET, VECTOR_TEXTPATH_PICK,
};

/// **Contour** (pesquisa `20_*` #9): a seção + os três comandos + os pares Steps/Offset/Accel + a
/// swatch da cor-alvo + os dois trios exclusivos (Corner / Side).
pub use ph2d_editor_core::ids::{
    VECTOR_CONTOUR_ACCEL, VECTOR_CONTOUR_ACCEL_NUM, VECTOR_CONTOUR_ADD, VECTOR_CONTOUR_EXPAND,
    VECTOR_CONTOUR_OFFSET, VECTOR_CONTOUR_OFFSET_NUM, VECTOR_CONTOUR_REMOVE, VECTOR_CONTOUR_STEPS,
    VECTOR_CONTOUR_STEPS_NUM, VECTOR_CONTOUR_TO,
};

/// **Filters** (a pilha de FX raster, plano 24): a seção + os "Add" + o bloco de controles de cada
/// LINHA (card / ✕ / ↑ / ↓ / 👁 / Radius / OffX / OffY / Color / Opacity). Distinta de EFFECTS
/// (deformadores vetoriais).
pub use ph2d_editor_core::ids::{MAX_FILTER_ROWS, filter_ramp_id};

/// **Pattern on Path** (plano 23): a seção + os quatro botões + os pares Spacing/Start/End/Slide/Offset.
pub use ph2d_editor_core::ids::{
    VECTOR_PATTERNPATH_DETACH, VECTOR_PATTERNPATH_END, VECTOR_PATTERNPATH_FLIP,
    VECTOR_PATTERNPATH_FLIP_OFF, VECTOR_PATTERNPATH_LINK, VECTOR_PATTERNPATH_OFFSET,
    VECTOR_PATTERNPATH_PICK, VECTOR_PATTERNPATH_ROTATION, VECTOR_PATTERNPATH_SLIDE,
    VECTOR_PATTERNPATH_SPACING, VECTOR_PATTERNPATH_START,
};

/// **A MOLDURA** (plano UI/UX W0): o 14º pill, a seção, os dois chips de recorte e os quatro
/// presets de dispositivo.
pub use ph2d_editor_core::ids::{
    VECTOR_FRAME_CLIP_OFF, VECTOR_FRAME_CLIP_ON, VECTOR_FRAME_PANEL_OFF, VECTOR_FRAME_PANEL_ON,
};

/// **O AUTO LAYOUT** (plano UI/UX W2, ADR-0153): a seção, o rádio de direção, os vãos, o recuo
/// (modo + cinco campos), as duas fileiras de alinhamento, o par Grow/Shrink do filho, o
/// vocabulário de TAMANHO (Fixed/Hug por eixo + os quatro limites) e o fora-do-fluxo.
pub use ph2d_editor_core::ids::{
    VECTOR_LAYOUT_ALIGN_CENTER, VECTOR_LAYOUT_ALIGN_END, VECTOR_LAYOUT_ALIGN_START,
    VECTOR_LAYOUT_ALIGN_STRETCH, VECTOR_LAYOUT_COLUMNS, VECTOR_LAYOUT_DIR_COL,
    VECTOR_LAYOUT_DIR_GRID, VECTOR_LAYOUT_DIR_OFF, VECTOR_LAYOUT_DIR_ROW, VECTOR_LAYOUT_DIR_WRAP,
    VECTOR_LAYOUT_GAP_CROSS, VECTOR_LAYOUT_GAP_MAIN, VECTOR_LAYOUT_ITEM_ABSOLUTE,
    VECTOR_LAYOUT_ITEM_GROW, VECTOR_LAYOUT_ITEM_SHRINK, VECTOR_LAYOUT_JUSTIFY_AROUND,
    VECTOR_LAYOUT_JUSTIFY_BETWEEN, VECTOR_LAYOUT_JUSTIFY_CENTER, VECTOR_LAYOUT_JUSTIFY_END,
    VECTOR_LAYOUT_JUSTIFY_START, VECTOR_LAYOUT_MAX_H, VECTOR_LAYOUT_MAX_W, VECTOR_LAYOUT_MIN_H,
    VECTOR_LAYOUT_MIN_W, VECTOR_LAYOUT_PAD_ALL, VECTOR_LAYOUT_PAD_B, VECTOR_LAYOUT_PAD_L,
    VECTOR_LAYOUT_PAD_R, VECTOR_LAYOUT_PAD_T, VECTOR_LAYOUT_SIZE_H_FIXED, VECTOR_LAYOUT_SIZE_H_HUG,
    VECTOR_LAYOUT_SIZE_W_FIXED, VECTOR_LAYOUT_SIZE_W_HUG,
};

/// **OS ESTADOS de UI** (plano UI/UX W7): a seção, os três verbos por papel e a duração.
pub use ph2d_editor_core::ids::{
    VECTOR_STATE_DAMPING, VECTOR_STATE_DURATION, VECTOR_STATE_MOVE_ALL, VECTOR_STATE_PREVIEW,
    VECTOR_STATE_SPRING, VECTOR_STATE_STIFFNESS,
};

mod vector_anchors;
pub use vector_anchors::*;
mod vector_appearance;
pub use vector_appearance::*;
mod vector_bool;
pub use vector_bool::*;
mod vector_components;
pub use vector_components::*;
mod vector_contour;
pub use vector_contour::*;
mod vector_filters;
pub use vector_filters::*;
mod vector_layout;
pub use vector_layout::*;
mod vector_morph;
pub use vector_morph::*;
mod vector_patternpath;
pub use vector_patternpath::*;
mod vector_sections;
pub use vector_sections::*;
mod vector_states;
pub use vector_states::*;
mod vector_text;
pub use vector_text::*;
mod vector_textpath;
pub use vector_textpath::*;
mod vector_texture_pattern;
pub use vector_texture_pattern::*;
mod vector_tokens;
pub use vector_tokens::*;
mod vector_widget;
pub use vector_widget::*;
