//! **O losango dos campos dos componentes da entidade e dos textos** — a metade de [`super`]
//! (`marcas.rs`) que o bloco 2 do plano 35 trouxe: o pincel do traço, a quebra e a família do
//! texto, o perfil de largura, o Blend/Morph/Envelope vivos, a Interface (âncoras, recorte, auto
//! layout, pele do widget), o padrão e o texto no caminho, e os filtros. O gate
//! `architecture_every_editable_field_is_a_channel` lê este ficheiro como tabela de canais.

use ph2d_ecs::FieldKey;
use ph2d_ecs::canal::familia::VETOR;
use ph2d_ecs::canal_vetor as v;
use ph2d_editor_core::NodeId;
use ph2d_panel_vector::ids as painel;
use ph2d_timeline::PropKind;
use ph2d_tool_vector::ids as ferramenta;

const fn campo(c: u16) -> PropKind {
    PropKind::Field(FieldKey::new(VETOR, c, 0, 0))
}

/// O pincel do traço de base é membro da TINTA dele (`canal_vetor::PINCEL_*`).
const PINCEL: PropKind = campo(v::TINTA_TRACO + v::OFF_SOLIDA);
const QUEBRA: PropKind = campo(v::TEXTO_QUEBRA);
const PERFIL: PropKind = campo(v::PERFIL_EXISTE);
const ENVELOPE: PropKind = campo(v::ENVELOPE_CANTO_X);
const ANCORAS: PropKind = campo(v::ANCORAS_EXISTE);
const DIR: PropKind = campo(v::LAYOUT_DIR);
const TAMANHO: PropKind = campo(v::LAYOUT_TAMANHO);
const MARGEM: PropKind = campo(v::LAYOUT_MARGEM);
const ALINHA: PropKind = campo(v::LAYOUT_ALINHA);
const DISTRIBUI: PropKind = campo(v::LAYOUT_DISTRIBUI);
const PADRAO_INICIO: PropKind = campo(v::PADRAO_CAMINHO_INICIO);
const PADRAO_FIM: PropKind = campo(v::PADRAO_CAMINHO_FIM);
const PADRAO_ESPELHO: PropKind = campo(v::PADRAO_CAMINHO_ESPELHO);
const TEXTO_ESPELHO: PropKind = campo(v::TEXTO_CAMINHO_ESPELHO);
/// O «Slide» move o início e o fim juntos: um losango, os dois.
const SLIDE: &[PropKind] = &[PADRAO_INICIO, PADRAO_FIM];

/// ⭐ **A tabela campo → canal** dos componentes vivos e da Interface (ids fixos).
pub(super) const CAMPOS: &[(NodeId, &[PropKind])] = &[
    (painel::VECTOR_BRUSH_SCALE, &[PINCEL]),
    (painel::VECTOR_BRUSH_SCALE_NUM, &[PINCEL]),
    (painel::VECTOR_BRUSH_SPACING, &[PINCEL]),
    (painel::VECTOR_BRUSH_SPACING_NUM, &[PINCEL]),
    (painel::VECTOR_BRUSH_ROTATION, &[PINCEL]),
    (painel::VECTOR_BRUSH_ROTATION_NUM, &[PINCEL]),
    (painel::VECTOR_BRUSH_OFFSET, &[PINCEL]),
    (painel::VECTOR_BRUSH_OFFSET_NUM, &[PINCEL]),
    (painel::VECTOR_BRUSH_FLIP, &[PINCEL]),
    (painel::VECTOR_TEXT_WRAP_AUTO, &[QUEBRA]),
    (painel::VECTOR_TEXT_WRAP_FIXED, &[QUEBRA]),
    (painel::VECTOR_TEXT_FONT_DD, &[campo(v::TEXTO_FAMILIA)]),
    (painel::VECTOR_BAR_PROFILE, &[PERFIL]),
    (painel::VECTOR_EXPAND_W_START, &[PERFIL]),
    (painel::VECTOR_EXPAND_W_MID, &[PERFIL]),
    (painel::VECTOR_EXPAND_W_END, &[PERFIL]),
    (painel::VECTOR_EXPAND_W_POS, &[PERFIL]),
    (ferramenta::VECTOR_EXPAND_W_START_NUM, &[PERFIL]),
    (ferramenta::VECTOR_EXPAND_W_MID_NUM, &[PERFIL]),
    (ferramenta::VECTOR_EXPAND_W_END_NUM, &[PERFIL]),
    (ferramenta::VECTOR_EXPAND_W_POS_NUM, &[PERFIL]),
    (painel::VECTOR_BLEND_STEPS, &[campo(v::BLEND_PASSOS)]),
    (
        ferramenta::VECTOR_BLEND_STEPS_NUM,
        &[campo(v::BLEND_PASSOS)],
    ),
    // O `t` do Morph vivo é o `PropKind::Morph` de sempre (um dono só para o número).
    (painel::VECTOR_MORPH_T, &[PropKind::Morph]),
    (ferramenta::VECTOR_MORPH_T_NUM, &[PropKind::Morph]),
    (painel::VECTOR_ENVELOPE_PERSPECTIVE, &[ENVELOPE]),
    (painel::VECTOR_ENVELOPE_MESH, &[ENVELOPE]),
    (painel::VECTOR_ENVELOPE_PINS, &[ENVELOPE]),
    (painel::VECTOR_ENVELOPE_BEND, &[ENVELOPE]),
    (ferramenta::VECTOR_ENVELOPE_BEND_NUM, &[ENVELOPE]),
    (painel::VECTOR_ANCHOR_H_START, &[ANCORAS]),
    (painel::VECTOR_ANCHOR_H_CENTER, &[ANCORAS]),
    (painel::VECTOR_ANCHOR_H_END, &[ANCORAS]),
    (painel::VECTOR_ANCHOR_H_STRETCH, &[ANCORAS]),
    (painel::VECTOR_ANCHOR_V_START, &[ANCORAS]),
    (painel::VECTOR_ANCHOR_V_CENTER, &[ANCORAS]),
    (painel::VECTOR_ANCHOR_V_END, &[ANCORAS]),
    (painel::VECTOR_ANCHOR_V_STRETCH, &[ANCORAS]),
    (painel::VECTOR_FRAME_CLIP_OFF, &[campo(v::MOLDURA_RECORTA)]),
    (painel::VECTOR_FRAME_CLIP_ON, &[campo(v::MOLDURA_RECORTA)]),
    (painel::VECTOR_LAYOUT_DIR_OFF, &[DIR]),
    (painel::VECTOR_LAYOUT_DIR_ROW, &[DIR]),
    (painel::VECTOR_LAYOUT_DIR_COL, &[DIR]),
    (painel::VECTOR_LAYOUT_DIR_WRAP, &[DIR]),
    (painel::VECTOR_LAYOUT_DIR_GRID, &[DIR]),
    (painel::VECTOR_LAYOUT_SIZE_W_FIXED, &[TAMANHO]),
    (painel::VECTOR_LAYOUT_SIZE_W_HUG, &[TAMANHO]),
    (painel::VECTOR_LAYOUT_SIZE_H_FIXED, &[TAMANHO]),
    (painel::VECTOR_LAYOUT_SIZE_H_HUG, &[TAMANHO]),
    (painel::VECTOR_LAYOUT_MIN_W, &[campo(v::LAYOUT_MIN)]),
    (painel::VECTOR_LAYOUT_MIN_H, &[campo(v::LAYOUT_MIN)]),
    (painel::VECTOR_LAYOUT_MAX_W, &[campo(v::LAYOUT_MAX)]),
    (painel::VECTOR_LAYOUT_MAX_H, &[campo(v::LAYOUT_MAX)]),
    (painel::VECTOR_LAYOUT_COLUMNS, &[campo(v::LAYOUT_COLUNAS)]),
    (painel::VECTOR_LAYOUT_GAP_MAIN, &[campo(v::LAYOUT_VAO_X)]),
    (painel::VECTOR_LAYOUT_GAP_CROSS, &[campo(v::LAYOUT_VAO_X)]),
    (painel::VECTOR_LAYOUT_PAD_ALL, &[MARGEM]),
    (painel::VECTOR_LAYOUT_PAD_T, &[MARGEM]),
    (painel::VECTOR_LAYOUT_PAD_R, &[MARGEM]),
    (painel::VECTOR_LAYOUT_PAD_B, &[MARGEM]),
    (painel::VECTOR_LAYOUT_PAD_L, &[MARGEM]),
    (painel::VECTOR_LAYOUT_ALIGN_START, &[ALINHA]),
    (painel::VECTOR_LAYOUT_ALIGN_CENTER, &[ALINHA]),
    (painel::VECTOR_LAYOUT_ALIGN_END, &[ALINHA]),
    (painel::VECTOR_LAYOUT_ALIGN_STRETCH, &[ALINHA]),
    (painel::VECTOR_LAYOUT_JUSTIFY_START, &[DISTRIBUI]),
    (painel::VECTOR_LAYOUT_JUSTIFY_CENTER, &[DISTRIBUI]),
    (painel::VECTOR_LAYOUT_JUSTIFY_END, &[DISTRIBUI]),
    (painel::VECTOR_LAYOUT_JUSTIFY_BETWEEN, &[DISTRIBUI]),
    (painel::VECTOR_LAYOUT_JUSTIFY_AROUND, &[DISTRIBUI]),
    (
        painel::VECTOR_LAYOUT_ITEM_ABSOLUTE,
        &[campo(v::LAYOUT_ABSOLUTO)],
    ),
    (painel::VECTOR_LAYOUT_ITEM_GROW, &[campo(v::LAYOUT_CRESCE)]),
    (
        painel::VECTOR_LAYOUT_ITEM_SHRINK,
        &[campo(v::LAYOUT_ENCOLHE)],
    ),
    (painel::VECTOR_WIDGET_ICON_DD, &[campo(v::WIDGET_ICONE)]),
    (
        painel::VECTOR_PATTERNPATH_SPACING,
        &[campo(v::PADRAO_CAMINHO_ESPACO)],
    ),
    (
        painel::VECTOR_PATTERNPATH_SPACING_NUM,
        &[campo(v::PADRAO_CAMINHO_ESPACO)],
    ),
    (painel::VECTOR_PATTERNPATH_START, &[PADRAO_INICIO]),
    (painel::VECTOR_PATTERNPATH_START_NUM, &[PADRAO_INICIO]),
    (painel::VECTOR_PATTERNPATH_END, &[PADRAO_FIM]),
    (painel::VECTOR_PATTERNPATH_END_NUM, &[PADRAO_FIM]),
    (painel::VECTOR_PATTERNPATH_SLIDE, SLIDE),
    (painel::VECTOR_PATTERNPATH_SLIDE_NUM, SLIDE),
    (
        painel::VECTOR_PATTERNPATH_OFFSET,
        &[campo(v::PADRAO_CAMINHO_DESLOCA)],
    ),
    (
        painel::VECTOR_PATTERNPATH_OFFSET_NUM,
        &[campo(v::PADRAO_CAMINHO_DESLOCA)],
    ),
    (
        painel::VECTOR_PATTERNPATH_ROTATION,
        &[campo(v::PADRAO_CAMINHO_ROTACAO)],
    ),
    (
        painel::VECTOR_PATTERNPATH_ROTATION_NUM,
        &[campo(v::PADRAO_CAMINHO_ROTACAO)],
    ),
    (painel::VECTOR_PATTERNPATH_FLIP, &[PADRAO_ESPELHO]),
    (painel::VECTOR_PATTERNPATH_FLIP_OFF, &[PADRAO_ESPELHO]),
    (
        painel::VECTOR_TEXTPATH_OFFSET,
        &[campo(v::TEXTO_CAMINHO_INICIO)],
    ),
    (
        painel::VECTOR_TEXTPATH_OFFSET_NUM,
        &[campo(v::TEXTO_CAMINHO_INICIO)],
    ),
    (painel::VECTOR_TEXTPATH_FLIP, &[TEXTO_ESPELHO]),
    (painel::VECTOR_TEXTPATH_FLIP_OFF, &[TEXTO_ESPELHO]),
];

fn em_i(c: u16, i: usize) -> PropKind {
    PropKind::Field(FieldKey::new(
        VETOR,
        c,
        u16::try_from(i).unwrap_or(u16::MAX),
        0,
    ))
}

/// ⭐ Os campos de id DERIVADO de um índice: cada degrau `r` dos filtros (o olho é um grupo, o
/// resto do degrau outro — `canal_vetor::FILTRO_*`), os perfis do catálogo e as peles do widget.
pub(super) fn derivados(t: &mut Vec<(NodeId, PropKind)>) {
    for r in 0..painel::MAX_FILTER_ROWS {
        t.push((painel::filter_hide_id(r), em_i(v::FILTRO_LIGADO, r)));
        let g = em_i(v::FILTRO_TIPO, r);
        t.extend((0..painel::MAX_FILTER_MODES).map(|m| (painel::filter_mode_id(r, m), g)));
        t.extend(
            [
                painel::filter_radius_id(r),
                painel::filter_radius_num_id(r),
                painel::filter_offx_id(r),
                painel::filter_offx_num_id(r),
                painel::filter_offy_id(r),
                painel::filter_offy_num_id(r),
                painel::filter_color_id(r),
                painel::filter_color_b_id(r),
                painel::filter_stop_color_id(r),
                painel::filter_blend_id(r),
                painel::filter_opacity_id(r),
                painel::filter_opacity_num_id(r),
                painel::filter_scale_id(r),
                painel::filter_scale_num_id(r),
                painel::filter_detail_id(r),
                painel::filter_detail_num_id(r),
                painel::filter_seed_id(r),
                painel::filter_seed_num_id(r),
                painel::filter_grow_id(r),
                painel::filter_grow_num_id(r),
                painel::filter_hue_id(r),
                painel::filter_hue_num_id(r),
                painel::filter_sat_id(r),
                painel::filter_sat_num_id(r),
                painel::filter_bright_id(r),
                painel::filter_bright_num_id(r),
            ]
            .map(|id| (id, g)),
        );
    }
    t.extend((0..painel::MAX_WIDTH_PRESETS).map(|i| (painel::vector_width_preset_id(i), PERFIL)));
    t.extend(
        (0..painel::MAX_WIDGET_KINDS)
            .map(|i| (painel::vector_widget_kind_id(i), campo(v::WIDGET_PELE))),
    );
}
