//! Os ids da galeria em cartões (doc `CartoesUI/08` §1.1) — literais, para o censo
//! `every_non_literal_hash_is_named` e o de colisões os lerem.

use ph2d_a11y::NodeId;
use ph2d_editor_core::panel::cards::estilos::botao_de;
use ph2d_tool_registry::hash_node_id;

// ── Os cartões (secções) ──
pub const CARD_IDENTITY: NodeId = hash_node_id("gallery.card.identity");
pub const CARD_ALIGN: NodeId = hash_node_id("gallery.card.align");
pub const CARD_BOOLEAN: NodeId = hash_node_id("gallery.card.boolean");
pub const CARD_TRANSFORM: NodeId = hash_node_id("gallery.card.transform");
pub const CARD_LAYOUT: NodeId = hash_node_id("gallery.card.layout");
pub const CARD_CORNERS: NodeId = hash_node_id("gallery.card.corners");
pub const CARD_CONSTRAINTS: NodeId = hash_node_id("gallery.card.constraints");
pub const CARD_APPEARANCE: NodeId = hash_node_id("gallery.card.appearance");
pub const CARD_FILL: NodeId = hash_node_id("gallery.card.fill");
pub const CARD_STROKE: NodeId = hash_node_id("gallery.card.stroke");
pub const CARD_EFFECTS: NodeId = hash_node_id("gallery.card.effects");
pub const CARD_TEXT: NodeId = hash_node_id("gallery.card.text");
pub const CARD_EXPORT: NodeId = hash_node_id("gallery.card.export");
pub const CARD_ACTIONS: NodeId = hash_node_id("gallery.card.actions");
pub const CARD_MARKS: NodeId = hash_node_id("gallery.card.marks");
pub const CARD_ORDERING: NodeId = hash_node_id("gallery.card.ordering");
pub const CARD_PHYSICS: NodeId = hash_node_id("gallery.card.physics");

// ── Identidade ──
pub const NAME: NodeId = hash_node_id("gallery.text.name");
pub const VISIBLE: NodeId = hash_node_id("gallery.object.visible");
pub const LOCK: NodeId = hash_node_id("gallery.object.lock");

// ── Align ──
pub const ALIGN_LEFT: NodeId = hash_node_id("gallery.align.left");
pub const ALIGN_HCENTER: NodeId = hash_node_id("gallery.align.hcenter");
pub const ALIGN_RIGHT: NodeId = hash_node_id("gallery.align.right");
pub const ALIGN_TOP: NodeId = hash_node_id("gallery.align.top");
pub const ALIGN_VCENTER: NodeId = hash_node_id("gallery.align.vcenter");
pub const ALIGN_BOTTOM: NodeId = hash_node_id("gallery.align.bottom");
pub const DISTRIBUTE_H: NodeId = hash_node_id("gallery.align.distribute_h");
pub const DISTRIBUTE_V: NodeId = hash_node_id("gallery.align.distribute_v");

// ── Boolean ──
pub const BOOL_UNION: NodeId = hash_node_id("gallery.boolean.union");
pub const BOOL_SUBTRACT: NodeId = hash_node_id("gallery.boolean.subtract");
pub const BOOL_INTERSECT: NodeId = hash_node_id("gallery.boolean.intersect");
pub const BOOL_EXCLUDE: NodeId = hash_node_id("gallery.boolean.exclude");
pub const FLATTEN: NodeId = hash_node_id("gallery.boolean.flatten");
pub const CONVERT: NodeId = hash_node_id("gallery.boolean.convert");

// ── Transform ──
pub const TRANSFORM_RESET: NodeId = hash_node_id("gallery.transform.reset");
pub const POS_X: NodeId = hash_node_id("gallery.field.pos_x");
pub const POS_Y: NodeId = hash_node_id("gallery.field.pos_y");
pub const ROTATION: NodeId = hash_node_id("gallery.field.rotation");
pub const ROTATE_90: NodeId = hash_node_id("gallery.transform.rotate_90");
pub const FLIP_H: NodeId = hash_node_id("gallery.transform.flip_h");
pub const FLIP_V: NodeId = hash_node_id("gallery.transform.flip_v");
pub const SCALE_X: NodeId = hash_node_id("gallery.field.scale_x");
pub const SCALE_Y: NodeId = hash_node_id("gallery.field.scale_y");
pub const SCALE_LINK: NodeId = hash_node_id("gallery.transform.scale_link");
pub const SKEW_X: NodeId = hash_node_id("gallery.field.skew_x");
pub const SKEW_Y: NodeId = hash_node_id("gallery.field.skew_y");

// ── Layout ──
pub const DIR_ROW: NodeId = hash_node_id("gallery.layout.dir_row");
pub const DIR_COL: NodeId = hash_node_id("gallery.layout.dir_col");
pub const DIR_WRAP: NodeId = hash_node_id("gallery.layout.dir_wrap");
pub const SIZE_W: NodeId = hash_node_id("gallery.field.size_w");
pub const SIZE_H: NodeId = hash_node_id("gallery.field.size_h");
pub const RESIZE_W: NodeId = hash_node_id("gallery.layout.resize_w");
pub const RESIZE_W_FIXED: NodeId = hash_node_id("gallery.layout.resize_w_fixed");
pub const RESIZE_W_HUG: NodeId = hash_node_id("gallery.layout.resize_w_hug");
pub const RESIZE_W_FILL: NodeId = hash_node_id("gallery.layout.resize_w_fill");
pub const RESIZE_H: NodeId = hash_node_id("gallery.layout.resize_h");
pub const RESIZE_H_FIXED: NodeId = hash_node_id("gallery.layout.resize_h_fixed");
pub const RESIZE_H_HUG: NodeId = hash_node_id("gallery.layout.resize_h_hug");
pub const RESIZE_H_FILL: NodeId = hash_node_id("gallery.layout.resize_h_fill");
pub const GAP: NodeId = hash_node_id("gallery.field.gap");
pub const PAD_H: NodeId = hash_node_id("gallery.field.pad_h");
pub const PAD_V: NodeId = hash_node_id("gallery.field.pad_v");
/// A grelha 3×3 do alinhamento do conteúdo, linha a linha.
pub const GRID: [NodeId; 9] = [
    hash_node_id("gallery.layout.grid_tl"),
    hash_node_id("gallery.layout.grid_tc"),
    hash_node_id("gallery.layout.grid_tr"),
    hash_node_id("gallery.layout.grid_ml"),
    hash_node_id("gallery.layout.grid_mc"),
    hash_node_id("gallery.layout.grid_mr"),
    hash_node_id("gallery.layout.grid_bl"),
    hash_node_id("gallery.layout.grid_bc"),
    hash_node_id("gallery.layout.grid_br"),
];

// ── Corners e Constraints ──
pub const RADIUS: NodeId = hash_node_id("gallery.field.radius");
pub const CORNER_EACH: NodeId = hash_node_id("gallery.corners.each");
pub const RADIUS_TL: NodeId = hash_node_id("gallery.field.radius_tl");
pub const RADIUS_TR: NodeId = hash_node_id("gallery.field.radius_tr");
pub const RADIUS_BL: NodeId = hash_node_id("gallery.field.radius_bl");
pub const RADIUS_BR: NodeId = hash_node_id("gallery.field.radius_br");
pub const CLIP: NodeId = hash_node_id("gallery.check.clip");
pub const CONS_H: NodeId = hash_node_id("gallery.constraints.h");
pub const CONS_H_LEFT: NodeId = hash_node_id("gallery.constraints.h_left");
pub const CONS_H_RIGHT: NodeId = hash_node_id("gallery.constraints.h_right");
pub const CONS_H_CENTER: NodeId = hash_node_id("gallery.constraints.h_center");
pub const CONS_H_SCALE: NodeId = hash_node_id("gallery.constraints.h_scale");
pub const CONS_V: NodeId = hash_node_id("gallery.constraints.v");
pub const CONS_V_TOP: NodeId = hash_node_id("gallery.constraints.v_top");
pub const CONS_V_BOTTOM: NodeId = hash_node_id("gallery.constraints.v_bottom");
pub const CONS_V_CENTER: NodeId = hash_node_id("gallery.constraints.v_center");
pub const CONS_V_SCALE: NodeId = hash_node_id("gallery.constraints.v_scale");

// ── Appearance ──
pub const OPACITY: NodeId = hash_node_id("gallery.field.opacity");
pub const BLEND: NodeId = hash_node_id("gallery.dropdown.blend");
pub const BLEND_NORMAL: NodeId = hash_node_id("gallery.dropdown.blend_normal");
pub const BLEND_MULTIPLY: NodeId = hash_node_id("gallery.dropdown.blend_multiply");
pub const BLEND_SCREEN: NodeId = hash_node_id("gallery.dropdown.blend_screen");
pub const BLEND_OVERLAY: NodeId = hash_node_id("gallery.dropdown.blend_overlay");

// ── Fill e Stroke ──
/// Uma linha da pilha de tinta: amostra + código, opacidade, olho, menos.
#[derive(Clone, Copy, Debug)]
pub struct PaintRow {
    pub swatch: NodeId,
    pub opacity: NodeId,
    pub eye: NodeId,
    pub remove: NodeId,
}

pub const FILL_STYLES: NodeId = botao_de(CARD_FILL);
pub const FILL_ADD: NodeId = hash_node_id("gallery.fill.add");
pub const FILLS: [PaintRow; 3] = [
    PaintRow {
        swatch: hash_node_id("gallery.fill0.swatch"),
        opacity: hash_node_id("gallery.fill0.opacity"),
        eye: hash_node_id("gallery.fill0.eye"),
        remove: hash_node_id("gallery.fill0.remove"),
    },
    PaintRow {
        swatch: hash_node_id("gallery.fill1.swatch"),
        opacity: hash_node_id("gallery.fill1.opacity"),
        eye: hash_node_id("gallery.fill1.eye"),
        remove: hash_node_id("gallery.fill1.remove"),
    },
    PaintRow {
        swatch: hash_node_id("gallery.fill2.swatch"),
        opacity: hash_node_id("gallery.fill2.opacity"),
        eye: hash_node_id("gallery.fill2.eye"),
        remove: hash_node_id("gallery.fill2.remove"),
    },
];
pub const STROKE_STYLES: NodeId = botao_de(CARD_STROKE);
pub const STROKE_ADD: NodeId = hash_node_id("gallery.stroke.add");
pub const STROKES: [PaintRow; 3] = [
    PaintRow {
        swatch: hash_node_id("gallery.stroke0.swatch"),
        opacity: hash_node_id("gallery.stroke0.opacity"),
        eye: hash_node_id("gallery.stroke0.eye"),
        remove: hash_node_id("gallery.stroke0.remove"),
    },
    PaintRow {
        swatch: hash_node_id("gallery.stroke1.swatch"),
        opacity: hash_node_id("gallery.stroke1.opacity"),
        eye: hash_node_id("gallery.stroke1.eye"),
        remove: hash_node_id("gallery.stroke1.remove"),
    },
    PaintRow {
        swatch: hash_node_id("gallery.stroke2.swatch"),
        opacity: hash_node_id("gallery.stroke2.opacity"),
        eye: hash_node_id("gallery.stroke2.eye"),
        remove: hash_node_id("gallery.stroke2.remove"),
    },
];
pub const STROKE_POS: NodeId = hash_node_id("gallery.stroke.position");
pub const STROKE_POS_CENTER: NodeId = hash_node_id("gallery.stroke.position_center");
pub const STROKE_POS_INSIDE: NodeId = hash_node_id("gallery.stroke.position_inside");
pub const STROKE_POS_OUTSIDE: NodeId = hash_node_id("gallery.stroke.position_outside");
pub const WEIGHT: NodeId = hash_node_id("gallery.field.weight");
pub const CAP_BUTT: NodeId = hash_node_id("gallery.choice.cap_butt");
pub const CAP_ROUND: NodeId = hash_node_id("gallery.choice.cap_round");
pub const CAP_SQUARE: NodeId = hash_node_id("gallery.choice.cap_square");
pub const JOIN_MITER: NodeId = hash_node_id("gallery.choice.join_miter");
pub const JOIN_ROUND: NodeId = hash_node_id("gallery.choice.join_round");
pub const JOIN_BEVEL: NodeId = hash_node_id("gallery.choice.join_bevel");

// ── Effects ──
/// Uma linha de efeito: a linha que abre o painel do lado, a lista do tipo (caixa + 4 opções, no
/// painel), olho, menos, os campos e a cor.
#[derive(Clone, Copy, Debug)]
pub struct EffectRow {
    pub open: NodeId,
    pub kind: NodeId,
    pub options: [NodeId; 4],
    pub eye: NodeId,
    pub remove: NodeId,
    pub x: NodeId,
    pub y: NodeId,
    pub blur: NodeId,
    pub spread: NodeId,
    pub swatch: NodeId,
}

pub const FX_ADD: NodeId = hash_node_id("gallery.fx.add");
/// As opções do «+»: o tipo do efeito novo (F43).
pub const FX_ADD_KINDS: [NodeId; 4] = [
    hash_node_id("gallery.fx.add_drop"),
    hash_node_id("gallery.fx.add_inner"),
    hash_node_id("gallery.fx.add_layer"),
    hash_node_id("gallery.fx.add_background"),
];
/// O ✕ do painel de propriedades ao lado.
pub const SIDE_CLOSE: NodeId = hash_node_id("gallery.side.close");
pub const EFFECTS: [EffectRow; 3] = [
    EffectRow {
        open: hash_node_id("gallery.fx0.open"),
        kind: hash_node_id("gallery.fx0.kind"),
        options: [
            hash_node_id("gallery.fx0.drop"),
            hash_node_id("gallery.fx0.inner"),
            hash_node_id("gallery.fx0.layer"),
            hash_node_id("gallery.fx0.background"),
        ],
        eye: hash_node_id("gallery.fx0.eye"),
        remove: hash_node_id("gallery.fx0.remove"),
        x: hash_node_id("gallery.fx0.x"),
        y: hash_node_id("gallery.fx0.y"),
        blur: hash_node_id("gallery.fx0.blur"),
        spread: hash_node_id("gallery.fx0.spread"),
        swatch: hash_node_id("gallery.fx0.swatch"),
    },
    EffectRow {
        open: hash_node_id("gallery.fx1.open"),
        kind: hash_node_id("gallery.fx1.kind"),
        options: [
            hash_node_id("gallery.fx1.drop"),
            hash_node_id("gallery.fx1.inner"),
            hash_node_id("gallery.fx1.layer"),
            hash_node_id("gallery.fx1.background"),
        ],
        eye: hash_node_id("gallery.fx1.eye"),
        remove: hash_node_id("gallery.fx1.remove"),
        x: hash_node_id("gallery.fx1.x"),
        y: hash_node_id("gallery.fx1.y"),
        blur: hash_node_id("gallery.fx1.blur"),
        spread: hash_node_id("gallery.fx1.spread"),
        swatch: hash_node_id("gallery.fx1.swatch"),
    },
    EffectRow {
        open: hash_node_id("gallery.fx2.open"),
        kind: hash_node_id("gallery.fx2.kind"),
        options: [
            hash_node_id("gallery.fx2.drop"),
            hash_node_id("gallery.fx2.inner"),
            hash_node_id("gallery.fx2.layer"),
            hash_node_id("gallery.fx2.background"),
        ],
        eye: hash_node_id("gallery.fx2.eye"),
        remove: hash_node_id("gallery.fx2.remove"),
        x: hash_node_id("gallery.fx2.x"),
        y: hash_node_id("gallery.fx2.y"),
        blur: hash_node_id("gallery.fx2.blur"),
        spread: hash_node_id("gallery.fx2.spread"),
        swatch: hash_node_id("gallery.fx2.swatch"),
    },
];

// ── Text ──
pub const FONT: NodeId = hash_node_id("gallery.text.font");
pub const FONT_INTER: NodeId = hash_node_id("gallery.text.font_inter");
pub const FONT_ROBOTO: NodeId = hash_node_id("gallery.text.font_roboto");
pub const FONT_NOTO: NodeId = hash_node_id("gallery.text.font_noto");
pub const FONT_WEIGHT: NodeId = hash_node_id("gallery.text.weight");
pub const FONT_REGULAR: NodeId = hash_node_id("gallery.text.weight_regular");
pub const FONT_MEDIUM: NodeId = hash_node_id("gallery.text.weight_medium");
pub const FONT_BOLD: NodeId = hash_node_id("gallery.text.weight_bold");
pub const FONT_SIZE: NodeId = hash_node_id("gallery.field.font_size");
pub const LINE_HEIGHT: NodeId = hash_node_id("gallery.field.line_height");
pub const LETTER_SPACING: NodeId = hash_node_id("gallery.field.letter_spacing");
pub const TEXT_LEFT: NodeId = hash_node_id("gallery.text.align_left");
pub const TEXT_CENTER: NodeId = hash_node_id("gallery.text.align_center");
pub const TEXT_RIGHT: NodeId = hash_node_id("gallery.text.align_right");

// ── Export ──
pub const EXPORT_SCALE: NodeId = hash_node_id("gallery.export.scale");
pub const EXPORT_X1: NodeId = hash_node_id("gallery.export.scale_x1");
pub const EXPORT_X2: NodeId = hash_node_id("gallery.export.scale_x2");
pub const EXPORT_X3: NodeId = hash_node_id("gallery.export.scale_x3");
pub const SUFFIX: NodeId = hash_node_id("gallery.export.suffix");
pub const FORMAT: NodeId = hash_node_id("gallery.export.format");
pub const FORMAT_PNG: NodeId = hash_node_id("gallery.export.format_png");
pub const FORMAT_JPG: NodeId = hash_node_id("gallery.export.format_jpg");
pub const FORMAT_SVG: NodeId = hash_node_id("gallery.export.format_svg");
pub const FORMAT_PDF: NodeId = hash_node_id("gallery.export.format_pdf");
pub const EXPORT: NodeId = hash_node_id("gallery.export.run");

// ── Actions, marcas, Ordering, Physics ──
pub const PRIMARY: NodeId = hash_node_id("gallery.action.primary");
pub const SECONDARY: NodeId = hash_node_id("gallery.action.secondary");
pub const RESET: NodeId = hash_node_id("gallery.action.reset");
pub const MARK_FREE: NodeId = hash_node_id("gallery.field.mark_free");
pub const MARK_ANIMATED: NodeId = hash_node_id("gallery.field.mark_animated");
pub const MARK_KEY: NodeId = hash_node_id("gallery.field.mark_key");
pub const MARK_CHANGED: NodeId = hash_node_id("gallery.field.mark_changed");
pub const Z_ORDER: NodeId = hash_node_id("gallery.field.z_order");
pub const MASS: NodeId = hash_node_id("gallery.field.mass");
pub const PHYSICS_ADD: NodeId = hash_node_id("gallery.action.physics_add");
pub const PHYSICS_REMOVE: NodeId = hash_node_id("gallery.action.physics_remove");

/// `(id, valor de fábrica, passo, arrasto por px)` de um campo numérico.
pub type Campo = (NodeId, f64, f64, f64);

#[path = "ids_avancado.rs"]
mod avancado;
#[path = "ids_reais.rs"]
mod reais;
#[path = "ids_rive.rs"]
mod rive;
pub use avancado::*;
pub use reais::*;
pub use rive::*;

/// Junta as tabelas de campos num array só (o tamanho conta-se, nunca se escolhe).
const fn junta<const N: usize>(partes: [&[Campo]; 4]) -> [Campo; N] {
    let mut out = [partes[0][0]; N];
    let (mut p, mut n) = (0, 0);
    while p < partes.len() {
        let mut i = 0;
        while i < partes[p].len() {
            out[n] = partes[p][i];
            n += 1;
            i += 1;
        }
        p += 1;
    }
    assert!(n == N);
    out
}

/// A tabela que o `populate` regista, o `prepare` marca e o ↺ repõe. Os números são dados do objeto
/// fictício.
pub const FIELDS: [Campo; BASE.len() + CAMPOS.len() + CAMPOS_RIVE.len() + CAMPOS_AVANCADO.len()] =
    junta([&BASE, &CAMPOS, &CAMPOS_RIVE, &CAMPOS_AVANCADO]);

const BASE: [Campo; 46] = [
    (POS_X, 120.0, 1.0, 1.0), // LITERAL-PX-OK: dado do objeto fictício
    (POS_Y, 80.0, 1.0, 1.0),  // LITERAL-PX-OK: dado do objeto fictício
    (ROTATION, 0.0, 1.0, 1.0),
    (SCALE_X, 1.0, 0.1, 0.01), // LITERAL-PX-OK: dado do objeto fictício
    (SCALE_Y, 1.0, 0.1, 0.01), // LITERAL-PX-OK: dado do objeto fictício
    (SKEW_X, 0.0, 1.0, 1.0),
    (SKEW_Y, 0.0, 1.0, 1.0),
    (SIZE_W, 64.0, 1.0, 1.0),   // LITERAL-PX-OK: dado do objeto fictício
    (SIZE_H, 48.0, 1.0, 1.0),   // LITERAL-PX-OK: dado do objeto fictício
    (GAP, 8.0, 1.0, 1.0),       // LITERAL-PX-OK: dado do objeto fictício
    (PAD_H, 16.0, 1.0, 1.0),    // LITERAL-PX-OK: dado do objeto fictício
    (PAD_V, 12.0, 1.0, 1.0),    // LITERAL-PX-OK: dado do objeto fictício
    (RADIUS, 8.0, 1.0, 1.0),    // LITERAL-PX-OK: dado do objeto fictício
    (RADIUS_TL, 8.0, 1.0, 1.0), // LITERAL-PX-OK: dado do objeto fictício
    (RADIUS_TR, 8.0, 1.0, 1.0), // LITERAL-PX-OK: dado do objeto fictício
    (RADIUS_BL, 8.0, 1.0, 1.0), // LITERAL-PX-OK: dado do objeto fictício
    (RADIUS_BR, 8.0, 1.0, 1.0), // LITERAL-PX-OK: dado do objeto fictício
    (OPACITY, 100.0, 1.0, 1.0), // LITERAL-PX-OK: dado do objeto fictício
    (FILLS[0].opacity, 100.0, 1.0, 1.0), // LITERAL-PX-OK: dado do objeto fictício
    (FILLS[1].opacity, 100.0, 1.0, 1.0), // LITERAL-PX-OK: dado do objeto fictício
    (FILLS[2].opacity, 100.0, 1.0, 1.0), // LITERAL-PX-OK: dado do objeto fictício
    (STROKES[0].opacity, 100.0, 1.0, 1.0), // LITERAL-PX-OK: dado do objeto fictício
    (STROKES[1].opacity, 100.0, 1.0, 1.0), // LITERAL-PX-OK: dado do objeto fictício
    (STROKES[2].opacity, 100.0, 1.0, 1.0), // LITERAL-PX-OK: dado do objeto fictício
    (WEIGHT, 3.0, 1.0, 0.1),    // LITERAL-PX-OK: dado do objeto fictício
    (EFFECTS[0].x, 0.0, 1.0, 1.0),
    (EFFECTS[0].y, 4.0, 1.0, 1.0), // LITERAL-PX-OK: dado do objeto fictício
    (EFFECTS[0].blur, 12.0, 1.0, 1.0), // LITERAL-PX-OK: dado do objeto fictício
    (EFFECTS[0].spread, 0.0, 1.0, 1.0),
    (EFFECTS[1].x, 0.0, 1.0, 1.0),
    (EFFECTS[1].y, 4.0, 1.0, 1.0), // LITERAL-PX-OK: dado do objeto fictício
    (EFFECTS[1].blur, 12.0, 1.0, 1.0), // LITERAL-PX-OK: dado do objeto fictício
    (EFFECTS[1].spread, 0.0, 1.0, 1.0),
    (EFFECTS[2].x, 0.0, 1.0, 1.0),
    (EFFECTS[2].y, 4.0, 1.0, 1.0), // LITERAL-PX-OK: dado do objeto fictício
    (EFFECTS[2].blur, 12.0, 1.0, 1.0), // LITERAL-PX-OK: dado do objeto fictício
    (EFFECTS[2].spread, 0.0, 1.0, 1.0),
    (FONT_SIZE, 16.0, 1.0, 1.0), // LITERAL-PX-OK: dado do objeto fictício
    (LINE_HEIGHT, 24.0, 1.0, 1.0), // LITERAL-PX-OK: dado do objeto fictício
    (LETTER_SPACING, 0.0, 1.0, 0.1), // LITERAL-PX-OK: dado do objeto fictício
    (MARK_FREE, 0.0, 1.0, 1.0),
    (MARK_ANIMATED, 0.0, 1.0, 1.0),
    (MARK_KEY, 0.0, 1.0, 1.0),
    (MARK_CHANGED, 0.0, 1.0, 1.0),
    (Z_ORDER, 3.0, 1.0, 0.1), // LITERAL-PX-OK: dado do objeto fictício
    (MASS, 1.0, 0.1, 0.01),   // LITERAL-PX-OK: dado do objeto fictício
];

/// `(id, chave do texto de fábrica)` dos campos de texto.
pub const TEXTS: [(NodeId, &str); 16] = [
    (NAME, "panel.widget_gallery.name_default"),
    (SUFFIX, "panel.widget_gallery.suffix_default"),
    (CAM_TARGET, "panel.widget_gallery.name_default"),
    (CS_TARGET, "panel.widget_gallery.real.object.enemy"),
    (META[0].text, "panel.widget_gallery.adv.text_default"),
    (META[1].text, "panel.widget_gallery.adv.label_default"),
    (META[2].text, "panel.widget_gallery.adv.text_default"),
    (META[3].text, "panel.widget_gallery.adv.text_default"),
    (META[4].text, "panel.widget_gallery.adv.text_default"),
    (META[5].text, "panel.widget_gallery.adv.text_default"),
    (PROPS[0].key, "panel.widget_gallery.adv.prop_key_default"),
    (
        PROPS[0].value,
        "panel.widget_gallery.adv.prop_value_default",
    ),
    (PROPS[1].key, "panel.widget_gallery.adv.prop_new_key"),
    (PROPS[1].value, "panel.widget_gallery.adv.prop_new_value"),
    (PROPS[2].key, "panel.widget_gallery.adv.prop_new_key"),
    (PROPS[2].value, "panel.widget_gallery.adv.prop_new_value"),
];
