//! Os ids das amostras do Rive na galeria (doc `CartoesUI/08` §1.4) — literais, para os censos.

use super::Campo;
use ph2d_a11y::NodeId;
use ph2d_tool_registry::hash_node_id;

pub const CARD_INTERP: NodeId = hash_node_id("gallery.rive.card_interp");
pub const CARD_INPUTS: NodeId = hash_node_id("gallery.rive.card_inputs");
pub const CARD_TRIM: NodeId = hash_node_id("gallery.rive.card_trim");
pub const CARD_SOLO: NodeId = hash_node_id("gallery.rive.card_solo");
pub const CARD_GRADIENT: NodeId = hash_node_id("gallery.rive.card_gradient");
pub const IN_LINEAR: NodeId = hash_node_id("gallery.rive.in_linear");
pub const IN_CUBIC: NodeId = hash_node_id("gallery.rive.in_cubic");
pub const IN_HOLD: NodeId = hash_node_id("gallery.rive.in_hold");
pub const CU_X1: NodeId = hash_node_id("gallery.rive.cu_x1");
pub const CU_Y1: NodeId = hash_node_id("gallery.rive.cu_y1");
pub const CU_X2: NodeId = hash_node_id("gallery.rive.cu_x2");
pub const CU_Y2: NodeId = hash_node_id("gallery.rive.cu_y2");
pub const CU_H1: NodeId = hash_node_id("gallery.rive.cu_h1");
pub const CU_H2: NodeId = hash_node_id("gallery.rive.cu_h2");
pub const SM_ADD: NodeId = hash_node_id("gallery.rive.sm_add");
pub const SM_ADD_NUMBER: NodeId = hash_node_id("gallery.rive.sm_add_number");
pub const SM_ADD_BOOLEAN: NodeId = hash_node_id("gallery.rive.sm_add_boolean");
pub const SM_ADD_TRIGGER: NodeId = hash_node_id("gallery.rive.sm_add_trigger");
pub const TR_START: NodeId = hash_node_id("gallery.rive.tr_start");
pub const TR_END: NodeId = hash_node_id("gallery.rive.tr_end");
pub const TR_OFFSET: NodeId = hash_node_id("gallery.rive.tr_offset");
pub const TR_MODE: NodeId = hash_node_id("gallery.rive.tr_mode");
pub const TR_SEQUENTIAL: NodeId = hash_node_id("gallery.rive.tr_sequential");
pub const TR_SYNCED: NodeId = hash_node_id("gallery.rive.tr_synced");
pub const GR_SOLID: NodeId = hash_node_id("gallery.rive.gr_solid");
pub const GR_LINEAR: NodeId = hash_node_id("gallery.rive.gr_linear");
pub const GR_RADIAL: NodeId = hash_node_id("gallery.rive.gr_radial");
pub const GR_ADD: NodeId = hash_node_id("gallery.rive.gr_add");
pub const GR_SWATCH_SOLID: NodeId = hash_node_id("gallery.rive.gr_swatch_solid");

/// Uma linha de entrada da máquina de estados: o número, a caixa, o botão de disparo e o «−».
#[derive(Clone, Copy, Debug)]
pub struct InputRow {
    pub number: NodeId,
    pub check: NodeId,
    pub fire: NodeId,
    pub remove: NodeId,
}

pub const INPUTS: [InputRow; 3] = [
    InputRow {
        number: hash_node_id("gallery.rive.input0.number"),
        check: hash_node_id("gallery.rive.input0.check"),
        fire: hash_node_id("gallery.rive.input0.fire"),
        remove: hash_node_id("gallery.rive.input0.remove"),
    },
    InputRow {
        number: hash_node_id("gallery.rive.input1.number"),
        check: hash_node_id("gallery.rive.input1.check"),
        fire: hash_node_id("gallery.rive.input1.fire"),
        remove: hash_node_id("gallery.rive.input1.remove"),
    },
    InputRow {
        number: hash_node_id("gallery.rive.input2.number"),
        check: hash_node_id("gallery.rive.input2.check"),
        fire: hash_node_id("gallery.rive.input2.fire"),
        remove: hash_node_id("gallery.rive.input2.remove"),
    },
];

/// Os filhos do grupo Solo.
pub const SOLO: [NodeId; 3] = [
    hash_node_id("gallery.rive.solo0"),
    hash_node_id("gallery.rive.solo1"),
    hash_node_id("gallery.rive.solo2"),
];

/// Uma paragem do gradiente: a pega na barra, a posição, a cor e o «−».
#[derive(Clone, Copy, Debug)]
pub struct StopRow {
    pub handle: NodeId,
    pub position: NodeId,
    pub swatch: NodeId,
    pub remove: NodeId,
}

pub const STOPS: [StopRow; 3] = [
    StopRow {
        handle: hash_node_id("gallery.rive.stop0.handle"),
        position: hash_node_id("gallery.rive.stop0.position"),
        swatch: hash_node_id("gallery.rive.stop0.swatch"),
        remove: hash_node_id("gallery.rive.stop0.remove"),
    },
    StopRow {
        handle: hash_node_id("gallery.rive.stop1.handle"),
        position: hash_node_id("gallery.rive.stop1.position"),
        swatch: hash_node_id("gallery.rive.stop1.swatch"),
        remove: hash_node_id("gallery.rive.stop1.remove"),
    },
    StopRow {
        handle: hash_node_id("gallery.rive.stop2.handle"),
        position: hash_node_id("gallery.rive.stop2.position"),
        swatch: hash_node_id("gallery.rive.stop2.swatch"),
        remove: hash_node_id("gallery.rive.stop2.remove"),
    },
];

/// Os campos numéricos.
pub const CAMPOS_RIVE: [Campo; 13] = [
    (CU_X1, 0.42, 0.01, 0.002), // LITERAL-PX-OK: dado do objeto fictício
    (CU_Y1, 0.0, 0.01, 0.002),  // LITERAL-PX-OK: dado do objeto fictício
    (CU_X2, 0.58, 0.01, 0.002), // LITERAL-PX-OK: dado do objeto fictício
    (CU_Y2, 1.0, 0.01, 0.002),  // LITERAL-PX-OK: dado do objeto fictício
    (INPUTS[0].number, 0.0, 1.0, 0.1), // LITERAL-PX-OK: dado do objeto fictício
    (INPUTS[1].number, 0.0, 1.0, 0.1), // LITERAL-PX-OK: dado do objeto fictício
    (INPUTS[2].number, 0.0, 1.0, 0.1), // LITERAL-PX-OK: dado do objeto fictício
    (TR_START, 0.0, 1.0, 0.5),  // LITERAL-PX-OK: dado do objeto fictício
    (TR_END, 100.0, 1.0, 0.5),  // LITERAL-PX-OK: dado do objeto fictício
    (TR_OFFSET, 0.0, 1.0, 1.0), // LITERAL-PX-OK: dado do objeto fictício
    (STOPS[0].position, 0.0, 1.0, 0.5), // LITERAL-PX-OK: dado do objeto fictício
    (STOPS[1].position, 100.0, 1.0, 0.5), // LITERAL-PX-OK: dado do objeto fictício
    (STOPS[2].position, 50.0, 1.0, 0.5), // LITERAL-PX-OK: dado do objeto fictício
];
