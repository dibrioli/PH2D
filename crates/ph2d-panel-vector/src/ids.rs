//! Widget `NodeId`s for the Vector Style panel.
//!
//! Like the other panel crates, the ids stay defined in editor-core
//! (`ph2d_editor_core::ids`) — the layout + z-order walk + `node_id_collisions`
//! arch test all reference them, and re-defining them here would fork the
//! source of truth. This module is a convenience re-export so the panel's
//! internal modules (and the tool, via `handle_panel_event`) can write
//! `crate::ids::VECTOR_*`.

/// O 6º pill de modo: **Connect** — a linha que gruda em duas formas e as segue.
pub use ph2d_editor_core::ids::VECTOR_MODE_BUILD;
pub use ph2d_editor_core::ids::VECTOR_MODE_CONNECT;
/// O **lápis** — o modo de mão livre, ao lado da caneta na fileira TOOL.
pub use ph2d_editor_core::ids::VECTOR_MODE_PENCIL;
/// O 8º pill de modo: **Pick Shapes** (Blend) — coleta as formas na ordem de clique.
pub use ph2d_editor_core::ids::VECTOR_MODE_PICKBLEND;
/// Pontas de traço (arrowheads): os dois chips + as opções do popover, por `slot`
/// (0 = começo, 1 = fim) e índice em `ph2d_vec_scene::ALL_MARKERS` — mais o TAMANHO da
/// cabeça, o ARREDONDAMENTO das quinas dela e a DUPLA VIA (estado derivado das duas pontas).
pub use ph2d_editor_core::ids::{
    MARKER_SLOTS, MAX_MARKER_OPTIONS, VECTOR_MARKER_BOTH, VECTOR_MARKER_END_DD,
    VECTOR_MARKER_ROUND, VECTOR_MARKER_SCALE, VECTOR_MARKER_START_DD, vector_marker_option_id,
};
pub use ph2d_editor_core::ids::{
    MAX_ENVELOPE_PRESETS, MAX_WIDTH_PRESETS, vector_envelope_preset_id, vector_width_preset_id,
};
pub use ph2d_editor_core::ids::{
    MAX_FX_KINDS, MAX_FX_ROW_PARAMS, MAX_FX_ROWS, VECTOR_FX_APPLY, vector_fx_add_id,
    vector_fx_card_id, vector_fx_down_id, vector_fx_hide_id, vector_fx_param_id,
    vector_fx_param_num_id, vector_fx_remove_id, vector_fx_toggle_id, vector_fx_up_id,
};
pub use ph2d_editor_core::ids::{
    MAX_SHAPE_FIELD_SLOTS, vector_shape_choice_id, vector_shape_field_id, vector_shape_group_id,
    vector_shape_id,
};
pub use ph2d_editor_core::ids::{
    MAX_TEXT_VARIATION_AXES, vector_text_axis_id, vector_text_font_option_id,
};
pub use ph2d_editor_core::ids::{
    VECTOR_ALIGN_BOTTOM, VECTOR_ALIGN_HCENTER, VECTOR_ALIGN_LEFT, VECTOR_ALIGN_RIGHT,
    VECTOR_ALIGN_TOP, VECTOR_ALIGN_VCENTER, VECTOR_ARRANGE_BACKWARD, VECTOR_ARRANGE_DUPLICATE,
    VECTOR_ARRANGE_FLIP_H, VECTOR_ARRANGE_FLIP_V, VECTOR_ARRANGE_FORWARD,
    VECTOR_ARRANGE_ROTATE_CCW, VECTOR_ARRANGE_ROTATE_CW, VECTOR_ARRANGE_TO_BACK,
    VECTOR_ARRANGE_TO_FRONT, VECTOR_BLEND_EXPAND, VECTOR_BLEND_RELEASE, VECTOR_BLEND_RESET_SPINE,
    VECTOR_BLEND_RUN, VECTOR_BLEND_STACK_UP, VECTOR_BLEND_STEPS, VECTOR_BLEND_STEPS_NUM,
    VECTOR_BOOL_EXCLUDE, VECTOR_BOOL_INTERSECT, VECTOR_BOOL_SUBTRACT, VECTOR_BOOL_UNION,
    VECTOR_CAP_BUTT, VECTOR_CAP_ROUND, VECTOR_CAP_SQUARE, VECTOR_CLOSE, VECTOR_COMPOUND_MAKE,
    VECTOR_COMPOUND_RELEASE, VECTOR_CONVERT_TO_CURVES, VECTOR_DASH, VECTOR_DASH_NUM,
    VECTOR_DISTRIBUTE_H, VECTOR_DISTRIBUTE_V, VECTOR_ENVELOPE_BEND, VECTOR_ENVELOPE_BEND_NUM,
    VECTOR_ENVELOPE_CLEAR_PINS, VECTOR_ENVELOPE_EXPAND, VECTOR_ENVELOPE_MESH,
    VECTOR_ENVELOPE_PERSPECTIVE, VECTOR_ENVELOPE_PINS, VECTOR_ENVELOPE_RELEASE,
    VECTOR_ENVELOPE_RUN, VECTOR_EXPAND_JOIN_BEVEL, VECTOR_EXPAND_JOIN_MITER,
    VECTOR_EXPAND_JOIN_ROUND, VECTOR_EXPAND_OFFSET, VECTOR_EXPAND_OFFSET_NUM,
    VECTOR_EXPAND_OFFSET_PATH, VECTOR_EXPAND_OUTLINE_STROKE, VECTOR_EXPAND_POWER_STROKE,
    VECTOR_EXPAND_SIDE_BOTH, VECTOR_EXPAND_SIDE_INNER, VECTOR_EXPAND_SIDE_OUTER,
    VECTOR_EXPAND_W_END, VECTOR_EXPAND_W_END_NUM, VECTOR_EXPAND_W_MID, VECTOR_EXPAND_W_MID_NUM,
    VECTOR_EXPAND_W_POS, VECTOR_EXPAND_W_POS_NUM, VECTOR_EXPAND_W_START, VECTOR_EXPAND_W_START_NUM,
    VECTOR_FILL_KIND_LINEAR, VECTOR_FILL_KIND_MULTI, VECTOR_FILL_KIND_RADIAL,
    VECTOR_FILL_KIND_SOLID, VECTOR_FILL_OPACITY, VECTOR_FILL_OPACITY_NUM, VECTOR_FILL_RULE_EVENODD,
    VECTOR_FILL_RULE_NONZERO, VECTOR_FILL_SWATCH, VECTOR_GAP, VECTOR_GAP_NUM,
    VECTOR_GRAD_ADD_POINT, VECTOR_GRAD_ADD_STOP, VECTOR_GRAD_ANGLE, VECTOR_GRAD_ANGLE_NUM,
    VECTOR_GRAD_INFLUENCE, VECTOR_GRAD_INFLUENCE_NUM, VECTOR_GRAD_JITTER, VECTOR_GRAD_JITTER_NUM,
    VECTOR_GRAD_REMOVE_POINT, VECTOR_GRAD_REMOVE_STOP, VECTOR_JOIN_BEVEL, VECTOR_JOIN_MITER,
    VECTOR_JOIN_ROUND, VECTOR_MODE_NODE, VECTOR_MODE_PEN, VECTOR_MODE_SELECT, VECTOR_MODE_TEXT,
    VECTOR_MORPH_RUN, VECTOR_MORPH_T, VECTOR_MORPH_T_NUM, VECTOR_PANEL, VECTOR_PATH_CLOSE,
    VECTOR_PATH_SHARPEN, VECTOR_PATH_SIMPLIFY, VECTOR_PATH_SMOOTH, VECTOR_PATH_SUBDIVIDE,
    VECTOR_PIVOT_EDIT, VECTOR_SNAP_OFF, VECTOR_SNAP_ON, VECTOR_STROKE_OPACITY,
    VECTOR_STROKE_OPACITY_NUM, VECTOR_STROKE_SWATCH, VECTOR_TEXT_ALIGN_CENTER,
    VECTOR_TEXT_ALIGN_LEFT, VECTOR_TEXT_ALIGN_RIGHT, VECTOR_TEXT_FONT_DD, VECTOR_TEXT_FONT_IMPORT,
    VECTOR_TEXT_FONT_NEXT, VECTOR_TEXT_FONT_PREV, VECTOR_TEXT_LINE_HEIGHT,
    VECTOR_TEXT_LINE_HEIGHT_NUM, VECTOR_TEXT_SIZE, VECTOR_TEXT_SIZE_NUM, VECTOR_TEXT_TRACKING,
    VECTOR_TEXT_TRACKING_NUM, VECTOR_TEXT_WEIGHT, VECTOR_TEXT_WEIGHT_NUM, VECTOR_TRANSFORM_H,
    VECTOR_TRANSFORM_R, VECTOR_TRANSFORM_W, VECTOR_TRANSFORM_X, VECTOR_TRANSFORM_Y,
    VECTOR_VERT_CORNER, VECTOR_VERT_DELETE, VECTOR_VERT_SMOOTH, VECTOR_VERT_SYMMETRIC,
    VECTOR_WIDTH, VECTOR_WIDTH_NUM,
};
/// Os campos da RELAÇÃO do conector (a seção só existe com um conector na seleção).
pub use ph2d_editor_core::ids::{
    VECTOR_CONNECTOR_CORNER, VECTOR_CONNECTOR_CURVE, VECTOR_CONNECTOR_JETTY,
    VECTOR_CONNECTOR_ROUTE, VECTOR_CONNECTOR_SPREAD, VECTOR_SECTION_CONNECTOR,
};
/// O 9º e 10º pills de modo: **Fillet** / **Chamfer** — arredondar / chanfrar quina por
/// clicar-e-arrastar (consolidam a alça do Node + o toggle da seção Vertex numa dupla).
pub use ph2d_editor_core::ids::{VECTOR_MODE_CHAMFER, VECTOR_MODE_FILLET, VECTOR_MODE_WIDTH};
/// Os cabeçalhos COLAPSÁVEIS (canon `section_header.md`) + o 5º pill de modo + o chip de
/// categoria do catálogo. `VECTOR_SECTIONS` é a lista que o `populate` marca como
/// colapsável — uma seção fora dela pinta um chevron que não dobra.
pub use ph2d_editor_core::ids::{
    VECTOR_MODE_SHAPE, VECTOR_SECTION_ALIGN, VECTOR_SECTION_ARRANGE, VECTOR_SECTION_AXES,
    VECTOR_SECTION_BLEND, VECTOR_SECTION_BOOLEAN, VECTOR_SECTION_EFFECTS, VECTOR_SECTION_ENVELOPE,
    VECTOR_SECTION_EXPAND, VECTOR_SECTION_FILL, VECTOR_SECTION_FILL_TYPE, VECTOR_SECTION_FONT,
    VECTOR_SECTION_MORPH, VECTOR_SECTION_PARAGRAPH, VECTOR_SECTION_PATH, VECTOR_SECTION_SHAPE,
    VECTOR_SECTION_SHAPE_PARAMS, VECTOR_SECTION_SNAP, VECTOR_SECTION_STROKE, VECTOR_SECTION_TEXT,
    VECTOR_SECTION_TOOL, VECTOR_SECTION_TRANSFORM, VECTOR_SECTION_VERTEX, VECTOR_SECTIONS,
    VECTOR_SHAPE_GROUP_DD,
};
/// **O LÁPIS** (plano 25 W1): a seção + os dois knobs da mão livre, cada um com o seu chip.
pub use ph2d_editor_core::ids::{
    VECTOR_PENCIL_FIDELITY, VECTOR_PENCIL_FIDELITY_NUM, VECTOR_PENCIL_STABILIZER,
    VECTOR_PENCIL_STABILIZER_NUM, VECTOR_PENCIL_W_PRESSURE, VECTOR_PENCIL_W_SPEED,
    VECTOR_PENCIL_W_UNIFORM, VECTOR_SECTION_PENCIL,
};
/// **Text on Path** (plano 22): a seção + os quatro controles + o par do offset.
pub use ph2d_editor_core::ids::{
    VECTOR_SECTION_TEXTPATH, VECTOR_TEXTPATH_DETACH, VECTOR_TEXTPATH_FLIP,
    VECTOR_TEXTPATH_FLIP_OFF, VECTOR_TEXTPATH_LINK, VECTOR_TEXTPATH_OFFSET,
    VECTOR_TEXTPATH_OFFSET_NUM, VECTOR_TEXTPATH_PICK,
};

/// **Contour** (pesquisa `20_*` #9): a seção + os três comandos + os pares Steps/Offset/Accel + a
/// swatch da cor-alvo + os dois trios exclusivos (Corner / Side).
pub use ph2d_editor_core::ids::{
    VECTOR_CONTOUR_ACCEL, VECTOR_CONTOUR_ACCEL_NUM, VECTOR_CONTOUR_ADD, VECTOR_CONTOUR_EXPAND,
    VECTOR_CONTOUR_JOIN_BEVEL, VECTOR_CONTOUR_JOIN_MITER, VECTOR_CONTOUR_JOIN_ROUND,
    VECTOR_CONTOUR_OFFSET, VECTOR_CONTOUR_OFFSET_NUM, VECTOR_CONTOUR_REMOVE,
    VECTOR_CONTOUR_SIDE_BOTH, VECTOR_CONTOUR_SIDE_INNER, VECTOR_CONTOUR_SIDE_OUTER,
    VECTOR_CONTOUR_STEPS, VECTOR_CONTOUR_STEPS_NUM, VECTOR_CONTOUR_TO, VECTOR_SECTION_CONTOUR,
};

/// **Filters** (a pilha de FX raster, plano 24): a seção + os "Add" + o bloco de controles de cada
/// LINHA (card / ✕ / ↑ / ↓ / 👁 / Radius / OffX / OffY / Color / Opacity). Distinta de EFFECTS
/// (deformadores vetoriais).
pub use ph2d_editor_core::ids::{
    MAX_FILTER_BLENDS, MAX_FILTER_KINDS, MAX_FILTER_MODES, MAX_FILTER_ROWS, MAX_FILTER_STOPS,
    VECTOR_SECTION_FILTERS, filter_add_id, filter_blend_id, filter_blend_option_id,
    filter_bright_id, filter_bright_num_id, filter_card_id, filter_color_b_id, filter_color_id,
    filter_detail_id, filter_detail_num_id, filter_down_id, filter_grow_id, filter_grow_num_id,
    filter_hide_id, filter_hue_id, filter_hue_num_id, filter_mode_id, filter_offx_id,
    filter_offx_num_id, filter_offy_id, filter_offy_num_id, filter_opacity_id,
    filter_opacity_num_id, filter_radius_id, filter_radius_num_id, filter_ramp_id,
    filter_remove_id, filter_sat_id, filter_sat_num_id, filter_scale_id, filter_scale_num_id,
    filter_seed_id, filter_seed_num_id, filter_stop_add_id, filter_stop_color_id, filter_stop_id,
    filter_stop_remove_id, filter_up_id,
};

/// **Pattern on Path** (plano 23): a seção + os quatro botões + os pares Spacing/Start/End/Slide/Offset.
pub use ph2d_editor_core::ids::{
    VECTOR_PATTERNPATH_DETACH, VECTOR_PATTERNPATH_END, VECTOR_PATTERNPATH_END_NUM,
    VECTOR_PATTERNPATH_FLIP, VECTOR_PATTERNPATH_FLIP_OFF, VECTOR_PATTERNPATH_LINK,
    VECTOR_PATTERNPATH_OFFSET, VECTOR_PATTERNPATH_OFFSET_NUM, VECTOR_PATTERNPATH_PICK,
    VECTOR_PATTERNPATH_ROTATION, VECTOR_PATTERNPATH_ROTATION_NUM, VECTOR_PATTERNPATH_SLIDE,
    VECTOR_PATTERNPATH_SLIDE_NUM, VECTOR_PATTERNPATH_SPACING, VECTOR_PATTERNPATH_SPACING_NUM,
    VECTOR_PATTERNPATH_START, VECTOR_PATTERNPATH_START_NUM, VECTOR_SECTION_PATTERNPATH,
};
