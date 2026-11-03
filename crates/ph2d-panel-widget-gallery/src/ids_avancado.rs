//! Os ids dos cartões Advanced e Render (doc `CartoesUI/08` §1.2) — literais, para os censos. Os
//! cartões guardam os ids das secções de sempre: ordem e tema sobrevivem.

use super::Campo;
use ph2d_a11y::NodeId;
use ph2d_tool_registry::hash_node_id;

pub const CARD_ADVANCED: NodeId = ph2d_editor_core::ids::INSP_SECTION_W6;
pub const CARD_RENDER: NodeId = ph2d_editor_core::ids::INSP_SECTION_STATUS;

// ── Advanced ──
pub const ADV_X: NodeId = hash_node_id("gallery.adv.x");
pub const ADV_Y: NodeId = hash_node_id("gallery.adv.y");
pub const ADV_W: NodeId = hash_node_id("gallery.adv.w");
pub const ADV_H: NodeId = hash_node_id("gallery.adv.h");
pub const ADV_ROTATION: NodeId = hash_node_id("gallery.adv.rotation");
pub const SLICE: [NodeId; 4] = [
    hash_node_id("gallery.adv.slice_simple"),
    hash_node_id("gallery.adv.slice_sliced"),
    hash_node_id("gallery.adv.slice_tiled"),
    hash_node_id("gallery.adv.slice_tiled_fit"),
];
pub const LAYERS: [NodeId; 32] = [
    hash_node_id("gallery.adv.layer00"),
    hash_node_id("gallery.adv.layer01"),
    hash_node_id("gallery.adv.layer02"),
    hash_node_id("gallery.adv.layer03"),
    hash_node_id("gallery.adv.layer04"),
    hash_node_id("gallery.adv.layer05"),
    hash_node_id("gallery.adv.layer06"),
    hash_node_id("gallery.adv.layer07"),
    hash_node_id("gallery.adv.layer08"),
    hash_node_id("gallery.adv.layer09"),
    hash_node_id("gallery.adv.layer10"),
    hash_node_id("gallery.adv.layer11"),
    hash_node_id("gallery.adv.layer12"),
    hash_node_id("gallery.adv.layer13"),
    hash_node_id("gallery.adv.layer14"),
    hash_node_id("gallery.adv.layer15"),
    hash_node_id("gallery.adv.layer16"),
    hash_node_id("gallery.adv.layer17"),
    hash_node_id("gallery.adv.layer18"),
    hash_node_id("gallery.adv.layer19"),
    hash_node_id("gallery.adv.layer20"),
    hash_node_id("gallery.adv.layer21"),
    hash_node_id("gallery.adv.layer22"),
    hash_node_id("gallery.adv.layer23"),
    hash_node_id("gallery.adv.layer24"),
    hash_node_id("gallery.adv.layer25"),
    hash_node_id("gallery.adv.layer26"),
    hash_node_id("gallery.adv.layer27"),
    hash_node_id("gallery.adv.layer28"),
    hash_node_id("gallery.adv.layer29"),
    hash_node_id("gallery.adv.layer30"),
    hash_node_id("gallery.adv.layer31"),
];

/// Uma entrada de Metadata: a caixa do tipo e as suas opções, os três valores (só o do tipo se
/// pinta) e o «−».
#[derive(Clone, Copy, Debug)]
pub struct MetaRow {
    pub kind: NodeId,
    pub options: [NodeId; 3],
    pub number: NodeId,
    pub text: NodeId,
    pub swatch: NodeId,
    pub remove: NodeId,
}

pub const META: [MetaRow; 6] = [
    MetaRow {
        kind: hash_node_id("gallery.adv.meta0.kind"),
        options: [
            hash_node_id("gallery.adv.meta0.float"),
            hash_node_id("gallery.adv.meta0.text"),
            hash_node_id("gallery.adv.meta0.color"),
        ],
        number: hash_node_id("gallery.adv.meta0.number"),
        text: hash_node_id("gallery.adv.meta0.value"),
        swatch: hash_node_id("gallery.adv.meta0.swatch"),
        remove: hash_node_id("gallery.adv.meta0.remove"),
    },
    MetaRow {
        kind: hash_node_id("gallery.adv.meta1.kind"),
        options: [
            hash_node_id("gallery.adv.meta1.float"),
            hash_node_id("gallery.adv.meta1.text"),
            hash_node_id("gallery.adv.meta1.color"),
        ],
        number: hash_node_id("gallery.adv.meta1.number"),
        text: hash_node_id("gallery.adv.meta1.value"),
        swatch: hash_node_id("gallery.adv.meta1.swatch"),
        remove: hash_node_id("gallery.adv.meta1.remove"),
    },
    MetaRow {
        kind: hash_node_id("gallery.adv.meta2.kind"),
        options: [
            hash_node_id("gallery.adv.meta2.float"),
            hash_node_id("gallery.adv.meta2.text"),
            hash_node_id("gallery.adv.meta2.color"),
        ],
        number: hash_node_id("gallery.adv.meta2.number"),
        text: hash_node_id("gallery.adv.meta2.value"),
        swatch: hash_node_id("gallery.adv.meta2.swatch"),
        remove: hash_node_id("gallery.adv.meta2.remove"),
    },
    MetaRow {
        kind: hash_node_id("gallery.adv.meta3.kind"),
        options: [
            hash_node_id("gallery.adv.meta3.float"),
            hash_node_id("gallery.adv.meta3.text"),
            hash_node_id("gallery.adv.meta3.color"),
        ],
        number: hash_node_id("gallery.adv.meta3.number"),
        text: hash_node_id("gallery.adv.meta3.value"),
        swatch: hash_node_id("gallery.adv.meta3.swatch"),
        remove: hash_node_id("gallery.adv.meta3.remove"),
    },
    MetaRow {
        kind: hash_node_id("gallery.adv.meta4.kind"),
        options: [
            hash_node_id("gallery.adv.meta4.float"),
            hash_node_id("gallery.adv.meta4.text"),
            hash_node_id("gallery.adv.meta4.color"),
        ],
        number: hash_node_id("gallery.adv.meta4.number"),
        text: hash_node_id("gallery.adv.meta4.value"),
        swatch: hash_node_id("gallery.adv.meta4.swatch"),
        remove: hash_node_id("gallery.adv.meta4.remove"),
    },
    MetaRow {
        kind: hash_node_id("gallery.adv.meta5.kind"),
        options: [
            hash_node_id("gallery.adv.meta5.float"),
            hash_node_id("gallery.adv.meta5.text"),
            hash_node_id("gallery.adv.meta5.color"),
        ],
        number: hash_node_id("gallery.adv.meta5.number"),
        text: hash_node_id("gallery.adv.meta5.value"),
        swatch: hash_node_id("gallery.adv.meta5.swatch"),
        remove: hash_node_id("gallery.adv.meta5.remove"),
    },
];
pub const META_ADD: NodeId = hash_node_id("gallery.adv.meta_add");
pub const META_ADD_FLOAT: NodeId = hash_node_id("gallery.adv.meta_add_float");
pub const META_ADD_TEXT: NodeId = hash_node_id("gallery.adv.meta_add_text");
pub const META_ADD_COLOR: NodeId = hash_node_id("gallery.adv.meta_add_color");

/// Uma propriedade: a chave, o valor e o lixo.
#[derive(Clone, Copy, Debug)]
pub struct PropRow {
    pub key: NodeId,
    pub value: NodeId,
    pub remove: NodeId,
}

pub const PROPS: [PropRow; 3] = [
    PropRow {
        key: hash_node_id("gallery.adv.prop0.key"),
        value: hash_node_id("gallery.adv.prop0.value"),
        remove: hash_node_id("gallery.adv.prop0.remove"),
    },
    PropRow {
        key: hash_node_id("gallery.adv.prop1.key"),
        value: hash_node_id("gallery.adv.prop1.value"),
        remove: hash_node_id("gallery.adv.prop1.remove"),
    },
    PropRow {
        key: hash_node_id("gallery.adv.prop2.key"),
        value: hash_node_id("gallery.adv.prop2.value"),
        remove: hash_node_id("gallery.adv.prop2.remove"),
    },
];
pub const PROP_ADD: NodeId = hash_node_id("gallery.adv.prop_add");

// ── Render ──
pub const RN_FORMAT: NodeId = hash_node_id("gallery.render.format");
pub const RN_FORMATS: [NodeId; 4] = [
    hash_node_id("gallery.render.format_png"),
    hash_node_id("gallery.render.format_mp4"),
    hash_node_id("gallery.render.format_webp"),
    hash_node_id("gallery.render.format_gif"),
];
pub const RN_QUALITY: [NodeId; 3] = [
    hash_node_id("gallery.render.quality_draft"),
    hash_node_id("gallery.render.quality_good"),
    hash_node_id("gallery.render.quality_best"),
];
pub const RN_RUN: NodeId = hash_node_id("gallery.render.run");
pub const RN_CANCEL: NodeId = hash_node_id("gallery.render.cancel");

/// Os campos destes dois cartões.
pub const CAMPOS_AVANCADO: [Campo; 11] = [
    (ADV_X, 0.0, 1.0, 1.0),
    (ADV_Y, 0.0, 1.0, 1.0),
    (ADV_W, 100.0, 1.0, 1.0),       // LITERAL-PX-OK: dado do objeto fictício
    (ADV_H, 50.0, 1.0, 1.0),        // LITERAL-PX-OK: dado do objeto fictício
    (ADV_ROTATION, 90.0, 1.0, 1.0), // LITERAL-PX-OK: dado do objeto fictício
    (META[0].number, 0.25, 0.01, 0.01), // LITERAL-PX-OK: dado do objeto fictício
    (META[1].number, 0.0, 0.01, 0.01), // LITERAL-PX-OK: dado do objeto fictício
    (META[2].number, 0.0, 0.01, 0.01), // LITERAL-PX-OK: dado do objeto fictício
    (META[3].number, 0.0, 0.01, 0.01), // LITERAL-PX-OK: dado do objeto fictício
    (META[4].number, 0.0, 0.01, 0.01), // LITERAL-PX-OK: dado do objeto fictício
    (META[5].number, 0.0, 0.01, 0.01), // LITERAL-PX-OK: dado do objeto fictício
];
