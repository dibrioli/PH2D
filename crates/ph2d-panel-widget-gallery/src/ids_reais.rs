//! Os ids dos cartões de CASOS REAIS da galeria (doc `CartoesUI/08` §1.3) — literais, para os censos.

use super::Campo;
use ph2d_a11y::NodeId;
use ph2d_tool_registry::hash_node_id;

// ── Os cartões ──
pub const CARD_IMAGE: NodeId = hash_node_id("gallery.real.card_image");
pub const CARD_RIGID_BODY: NodeId = hash_node_id("gallery.real.card_rigid_body");
pub const CARD_COLLIDER: NodeId = hash_node_id("gallery.real.card_collider");
pub const CARD_HEALTH: NodeId = hash_node_id("gallery.real.card_health");
pub const CARD_PLAYER: NodeId = hash_node_id("gallery.real.card_player");
pub const CARD_CAMERA: NodeId = hash_node_id("gallery.real.card_camera");
pub const CARD_ANIMATION: NodeId = hash_node_id("gallery.real.card_animation");
pub const CARD_BRUSH: NodeId = hash_node_id("gallery.real.card_brush");
pub const CARD_BONES: NodeId = hash_node_id("gallery.real.card_bones");
pub const CARD_CONSTRAINT: NodeId = hash_node_id("gallery.real.card_constraint");
pub const CARD_LIGHT: NodeId = hash_node_id("gallery.real.card_light");
pub const CARD_AUDIO: NodeId = hash_node_id("gallery.real.card_audio");
pub const CARD_PIXEL: NodeId = hash_node_id("gallery.real.card_pixel");
pub const CARD_SCRIPT: NodeId = hash_node_id("gallery.real.card_script");

// ── Image ──
pub const IMG_ASSET: NodeId = hash_node_id("gallery.real.img_asset");
pub const IMG_ASSET_0: NodeId = hash_node_id("gallery.real.img_asset_0");
pub const IMG_ASSET_1: NodeId = hash_node_id("gallery.real.img_asset_1");
pub const IMG_ASSET_2: NodeId = hash_node_id("gallery.real.img_asset_2");
pub const USE_STILL: NodeId = hash_node_id("gallery.real.use_still");
pub const USE_REPEATED: NodeId = hash_node_id("gallery.real.use_repeated");
pub const USE_ANIMATED: NodeId = hash_node_id("gallery.real.use_animated");
pub const IMG_FRAME: NodeId = hash_node_id("gallery.real.img_frame");
pub const IMG_W: NodeId = hash_node_id("gallery.real.img_w");
pub const IMG_H: NodeId = hash_node_id("gallery.real.img_h");
pub const IMG_COLS: NodeId = hash_node_id("gallery.real.img_cols");
pub const IMG_ROWS: NodeId = hash_node_id("gallery.real.img_rows");
pub const IMG_FPS: NodeId = hash_node_id("gallery.real.img_fps");

// ── Physics Body ──
pub const RB_DYNAMIC: NodeId = hash_node_id("gallery.real.rb_dynamic");
pub const RB_KINEMATIC: NodeId = hash_node_id("gallery.real.rb_kinematic");
pub const RB_STATIC: NodeId = hash_node_id("gallery.real.rb_static");
pub const RB_MASS: NodeId = hash_node_id("gallery.real.rb_mass");
pub const RB_GRAVITY: NodeId = hash_node_id("gallery.real.rb_gravity");
pub const RB_LINEAR: NodeId = hash_node_id("gallery.real.rb_linear");
pub const RB_ANGULAR: NodeId = hash_node_id("gallery.real.rb_angular");
pub const RB_LOCK: NodeId = hash_node_id("gallery.real.rb_lock");

// ── Collider ──
pub const COL_BOX: NodeId = hash_node_id("gallery.real.col_box");
pub const COL_CIRCLE: NodeId = hash_node_id("gallery.real.col_circle");
pub const COL_CAPSULE: NodeId = hash_node_id("gallery.real.col_capsule");
pub const COL_W: NodeId = hash_node_id("gallery.real.col_w");
pub const COL_H: NodeId = hash_node_id("gallery.real.col_h");
pub const COL_RADIUS: NodeId = hash_node_id("gallery.real.col_radius");
pub const COL_HALF: NodeId = hash_node_id("gallery.real.col_half");
pub const COL_OX: NodeId = hash_node_id("gallery.real.col_ox");
pub const COL_OY: NodeId = hash_node_id("gallery.real.col_oy");
pub const COL_FRICTION: NodeId = hash_node_id("gallery.real.col_friction");
pub const COL_BOUNCE: NodeId = hash_node_id("gallery.real.col_bounce");
pub const COL_TRIGGER: NodeId = hash_node_id("gallery.real.col_trigger");

// ── Health ──
pub const HP_MAX: NodeId = hash_node_id("gallery.real.hp_max");
pub const HP_START: NodeId = hash_node_id("gallery.real.hp_start");
pub const HP_REGEN: NodeId = hash_node_id("gallery.real.hp_regen");
pub const HP_DELAY: NodeId = hash_node_id("gallery.real.hp_delay");
pub const HP_OVERHEAL: NodeId = hash_node_id("gallery.real.hp_overheal");

// ── Platform Player ──
pub const PL_SPEED: NodeId = hash_node_id("gallery.real.pl_speed");
pub const PL_ACCEL: NodeId = hash_node_id("gallery.real.pl_accel");
pub const PL_AIR: NodeId = hash_node_id("gallery.real.pl_air");
pub const PL_JUMP: NodeId = hash_node_id("gallery.real.pl_jump");
pub const PL_AIR_JUMPS: NodeId = hash_node_id("gallery.real.pl_air_jumps");
pub const PL_COYOTE: NodeId = hash_node_id("gallery.real.pl_coyote");
pub const PL_BUFFER: NodeId = hash_node_id("gallery.real.pl_buffer");

// ── Camera ──
pub const CAM_TARGET: NodeId = hash_node_id("gallery.real.cam_target");
pub const CAM_PICK: NodeId = hash_node_id("gallery.real.cam_pick");
pub const CAM_FORMAT: NodeId = hash_node_id("gallery.real.cam_format");
pub const CAM_FORMAT_0: NodeId = hash_node_id("gallery.real.cam_format_0");
pub const CAM_FORMAT_1: NodeId = hash_node_id("gallery.real.cam_format_1");
pub const CAM_FORMAT_2: NodeId = hash_node_id("gallery.real.cam_format_2");
pub const CAM_FRAME_ALWAYS: NodeId = hash_node_id("gallery.real.cam_frame_always");
pub const CAM_FRAME_SELECTED: NodeId = hash_node_id("gallery.real.cam_frame_selected");
pub const CAM_DEAD_W: NodeId = hash_node_id("gallery.real.cam_dead_w");
pub const CAM_DEAD_H: NodeId = hash_node_id("gallery.real.cam_dead_h");
pub const CAM_DAMPING: NodeId = hash_node_id("gallery.real.cam_damping");
pub const CAM_LOOKAHEAD: NodeId = hash_node_id("gallery.real.cam_lookahead");
pub const CAM_ACTIVE: NodeId = hash_node_id("gallery.real.cam_active");
pub const CAM_LOOK: NodeId = hash_node_id("gallery.real.cam_look");

// ── Animation ──
pub const ANIM_ME: NodeId = hash_node_id("gallery.real.anim_me");
pub const ANIM_SCRIPT: NodeId = hash_node_id("gallery.real.anim_script");
pub const ANIM_RULES: NodeId = hash_node_id("gallery.real.anim_rules");
pub const ANIM_CLIP: NodeId = hash_node_id("gallery.real.anim_clip");
pub const ANIM_CLIP_0: NodeId = hash_node_id("gallery.real.anim_clip_0");
pub const ANIM_CLIP_1: NodeId = hash_node_id("gallery.real.anim_clip_1");
pub const ANIM_CLIP_2: NodeId = hash_node_id("gallery.real.anim_clip_2");
pub const ANIM_CLIP_3: NodeId = hash_node_id("gallery.real.anim_clip_3");
pub const ANIM_PLAY: NodeId = hash_node_id("gallery.real.anim_play");
pub const ANIM_FPS: NodeId = hash_node_id("gallery.real.anim_fps");
pub const RULE_ADD: NodeId = hash_node_id("gallery.real.rule_add");

/// Uma regra: `quando` (caixa + 4 opções), `toca` (caixa + 4 opções), menos.
#[derive(Clone, Copy, Debug)]
pub struct RuleRow {
    pub when: NodeId,
    pub whens: [NodeId; 4],
    pub play: NodeId,
    pub plays: [NodeId; 4],
    pub remove: NodeId,
}

pub const RULES: [RuleRow; 3] = [
    RuleRow {
        when: hash_node_id("gallery.rule0.when"),
        whens: [
            hash_node_id("gallery.rule0.when0"),
            hash_node_id("gallery.rule0.when1"),
            hash_node_id("gallery.rule0.when2"),
            hash_node_id("gallery.rule0.when3"),
        ],
        play: hash_node_id("gallery.rule0.play"),
        plays: [
            hash_node_id("gallery.rule0.play0"),
            hash_node_id("gallery.rule0.play1"),
            hash_node_id("gallery.rule0.play2"),
            hash_node_id("gallery.rule0.play3"),
        ],
        remove: hash_node_id("gallery.rule0.remove"),
    },
    RuleRow {
        when: hash_node_id("gallery.rule1.when"),
        whens: [
            hash_node_id("gallery.rule1.when0"),
            hash_node_id("gallery.rule1.when1"),
            hash_node_id("gallery.rule1.when2"),
            hash_node_id("gallery.rule1.when3"),
        ],
        play: hash_node_id("gallery.rule1.play"),
        plays: [
            hash_node_id("gallery.rule1.play0"),
            hash_node_id("gallery.rule1.play1"),
            hash_node_id("gallery.rule1.play2"),
            hash_node_id("gallery.rule1.play3"),
        ],
        remove: hash_node_id("gallery.rule1.remove"),
    },
    RuleRow {
        when: hash_node_id("gallery.rule2.when"),
        whens: [
            hash_node_id("gallery.rule2.when0"),
            hash_node_id("gallery.rule2.when1"),
            hash_node_id("gallery.rule2.when2"),
            hash_node_id("gallery.rule2.when3"),
        ],
        play: hash_node_id("gallery.rule2.play"),
        plays: [
            hash_node_id("gallery.rule2.play0"),
            hash_node_id("gallery.rule2.play1"),
            hash_node_id("gallery.rule2.play2"),
            hash_node_id("gallery.rule2.play3"),
        ],
        remove: hash_node_id("gallery.rule2.remove"),
    },
];

// ── Brush ──
pub const BR_PRESET: NodeId = hash_node_id("gallery.real.br_preset");
pub const BR_PRESET_0: NodeId = hash_node_id("gallery.real.br_preset_0");
pub const BR_PRESET_1: NodeId = hash_node_id("gallery.real.br_preset_1");
pub const BR_PRESET_2: NodeId = hash_node_id("gallery.real.br_preset_2");
pub const BR_PRESET_3: NodeId = hash_node_id("gallery.real.br_preset_3");
pub const BR_SIZE: NodeId = hash_node_id("gallery.real.br_size");
pub const BR_WATER: NodeId = hash_node_id("gallery.real.br_water");
pub const BR_OIL: NodeId = hash_node_id("gallery.real.br_oil");
pub const BR_DILUTION: NodeId = hash_node_id("gallery.real.br_dilution");
pub const BR_VOLUME: NodeId = hash_node_id("gallery.real.br_volume");
pub const BR_PRESS: NodeId = hash_node_id("gallery.real.br_press");
pub const BR_REWET: NodeId = hash_node_id("gallery.real.br_rewet");
pub const BR_BODY: NodeId = hash_node_id("gallery.real.br_body");

// ── Bones ──
pub const BO_LENGTH: NodeId = hash_node_id("gallery.real.bo_length");
pub const BO_BEND_CCW: NodeId = hash_node_id("gallery.real.bo_bend_ccw");
pub const BO_BEND_CW: NodeId = hash_node_id("gallery.real.bo_bend_cw");
pub const BO_LIMIT: NodeId = hash_node_id("gallery.real.bo_limit");
pub const BO_MIN: NodeId = hash_node_id("gallery.real.bo_min");
pub const BO_MAX: NodeId = hash_node_id("gallery.real.bo_max");
pub const BO_SMART: NodeId = hash_node_id("gallery.real.bo_smart");
pub const BO_IK: NodeId = hash_node_id("gallery.real.bo_ik");
pub const BO_MIX: NodeId = hash_node_id("gallery.real.bo_mix");
pub const BO_CHAIN: NodeId = hash_node_id("gallery.real.bo_chain");

// ── Constraint ──
pub const CS_ADD: NodeId = hash_node_id("gallery.real.cs_add");
pub const CS_EYE: NodeId = hash_node_id("gallery.real.cs_eye");
pub const CS_REMOVE: NodeId = hash_node_id("gallery.real.cs_remove");
pub const CS_KIND: NodeId = hash_node_id("gallery.real.cs_kind");
pub const CS_KIND_0: NodeId = hash_node_id("gallery.real.cs_kind_0");
pub const CS_KIND_1: NodeId = hash_node_id("gallery.real.cs_kind_1");
pub const CS_KIND_2: NodeId = hash_node_id("gallery.real.cs_kind_2");
pub const CS_TARGET: NodeId = hash_node_id("gallery.real.cs_target");
pub const CS_PICK: NodeId = hash_node_id("gallery.real.cs_pick");
pub const CS_INFLUENCE: NodeId = hash_node_id("gallery.real.cs_influence");
pub const CS_SPACE: NodeId = hash_node_id("gallery.real.cs_space");
pub const CS_SPACE_WORLD: NodeId = hash_node_id("gallery.real.cs_space_world");
pub const CS_SPACE_LOCAL: NodeId = hash_node_id("gallery.real.cs_space_local");

// ── Light 2D ──
pub const LT_POINT: NodeId = hash_node_id("gallery.real.lt_point");
pub const LT_AMBIENT: NodeId = hash_node_id("gallery.real.lt_ambient");
pub const LT_SWATCH: NodeId = hash_node_id("gallery.real.lt_swatch");
pub const LT_INTENSITY: NodeId = hash_node_id("gallery.real.lt_intensity");
pub const LT_SIZE: NodeId = hash_node_id("gallery.real.lt_size");
pub const LT_RANGE: NodeId = hash_node_id("gallery.real.lt_range");
pub const LT_SHADOWS: NodeId = hash_node_id("gallery.real.lt_shadows");

// ── Audio ──
pub const AU_GAIN: NodeId = hash_node_id("gallery.real.au_gain");
pub const AU_MUTE: NodeId = hash_node_id("gallery.real.au_mute");
pub const AU_ADD: NodeId = hash_node_id("gallery.real.au_add");
pub const AU_ADD_0: NodeId = hash_node_id("gallery.real.au_add_0");
pub const AU_ADD_1: NodeId = hash_node_id("gallery.real.au_add_1");
pub const AU_ADD_2: NodeId = hash_node_id("gallery.real.au_add_2");
pub const AU_ADD_3: NodeId = hash_node_id("gallery.real.au_add_3");

/// Uma linha do rack: a linha que abre o painel do lado, olho, menos, o tipo (no painel), os campos.
#[derive(Clone, Copy, Debug)]
pub struct RackRow {
    pub open: NodeId,
    pub eye: NodeId,
    pub remove: NodeId,
    pub kind: NodeId,
    pub options: [NodeId; 4],
    pub mix: NodeId,
    pub damp: NodeId,
}

pub const RACK: [RackRow; 3] = [
    RackRow {
        open: hash_node_id("gallery.rack0.open"),
        eye: hash_node_id("gallery.rack0.eye"),
        remove: hash_node_id("gallery.rack0.remove"),
        kind: hash_node_id("gallery.rack0.kind"),
        options: [
            hash_node_id("gallery.rack0.kind0"),
            hash_node_id("gallery.rack0.kind1"),
            hash_node_id("gallery.rack0.kind2"),
            hash_node_id("gallery.rack0.kind3"),
        ],
        mix: hash_node_id("gallery.rack0.mix"),
        damp: hash_node_id("gallery.rack0.damp"),
    },
    RackRow {
        open: hash_node_id("gallery.rack1.open"),
        eye: hash_node_id("gallery.rack1.eye"),
        remove: hash_node_id("gallery.rack1.remove"),
        kind: hash_node_id("gallery.rack1.kind"),
        options: [
            hash_node_id("gallery.rack1.kind0"),
            hash_node_id("gallery.rack1.kind1"),
            hash_node_id("gallery.rack1.kind2"),
            hash_node_id("gallery.rack1.kind3"),
        ],
        mix: hash_node_id("gallery.rack1.mix"),
        damp: hash_node_id("gallery.rack1.damp"),
    },
    RackRow {
        open: hash_node_id("gallery.rack2.open"),
        eye: hash_node_id("gallery.rack2.eye"),
        remove: hash_node_id("gallery.rack2.remove"),
        kind: hash_node_id("gallery.rack2.kind"),
        options: [
            hash_node_id("gallery.rack2.kind0"),
            hash_node_id("gallery.rack2.kind1"),
            hash_node_id("gallery.rack2.kind2"),
            hash_node_id("gallery.rack2.kind3"),
        ],
        mix: hash_node_id("gallery.rack2.mix"),
        damp: hash_node_id("gallery.rack2.damp"),
    },
];

// ── Pixel Art ──
pub const PX_FG: NodeId = hash_node_id("gallery.real.px_fg");
pub const PX_BG: NodeId = hash_node_id("gallery.real.px_bg");
pub const PX_SWAP: NodeId = hash_node_id("gallery.real.px_swap");
pub const PX_GRID_W: NodeId = hash_node_id("gallery.real.px_grid_w");
pub const PX_GRID_H: NodeId = hash_node_id("gallery.real.px_grid_h");
pub const PX_GRID_ON: NodeId = hash_node_id("gallery.real.px_grid_on");

/// A paleta indexada, linha a linha (8 × 2).
pub const PALETTE: [NodeId; 16] = [
    hash_node_id("gallery.pixel.palette0"),
    hash_node_id("gallery.pixel.palette1"),
    hash_node_id("gallery.pixel.palette2"),
    hash_node_id("gallery.pixel.palette3"),
    hash_node_id("gallery.pixel.palette4"),
    hash_node_id("gallery.pixel.palette5"),
    hash_node_id("gallery.pixel.palette6"),
    hash_node_id("gallery.pixel.palette7"),
    hash_node_id("gallery.pixel.palette8"),
    hash_node_id("gallery.pixel.palette9"),
    hash_node_id("gallery.pixel.palette10"),
    hash_node_id("gallery.pixel.palette11"),
    hash_node_id("gallery.pixel.palette12"),
    hash_node_id("gallery.pixel.palette13"),
    hash_node_id("gallery.pixel.palette14"),
    hash_node_id("gallery.pixel.palette15"),
];

// ── Script ──
pub const SC_FILE: NodeId = hash_node_id("gallery.real.sc_file");
pub const SC_FILE_0: NodeId = hash_node_id("gallery.real.sc_file_0");
pub const SC_FILE_1: NodeId = hash_node_id("gallery.real.sc_file_1");
pub const SC_FILE_2: NodeId = hash_node_id("gallery.real.sc_file_2");
pub const SC_EDIT: NodeId = hash_node_id("gallery.real.sc_edit");
pub const SC_SPEED: NodeId = hash_node_id("gallery.real.sc_speed");
pub const SC_JUMP: NodeId = hash_node_id("gallery.real.sc_jump");
pub const SC_DOUBLE: NodeId = hash_node_id("gallery.real.sc_double");
pub const SC_PATROL: NodeId = hash_node_id("gallery.real.sc_patrol");
pub const SC_DAMAGE: NodeId = hash_node_id("gallery.real.sc_damage");
pub const SC_VALUE: NodeId = hash_node_id("gallery.real.sc_value");

/// Os campos numéricos: nomes e unidades do código real; valores de um objeto fictício.
pub const CAMPOS: [Campo; 63] = [
    (IMG_FRAME, 0.0, 1.0, 0.1),       // LITERAL-PX-OK: dado do objeto fictício
    (IMG_W, 128.0, 1.0, 1.0),         // LITERAL-PX-OK: dado do objeto fictício
    (IMG_H, 32.0, 1.0, 1.0),          // LITERAL-PX-OK: dado do objeto fictício
    (IMG_COLS, 4.0, 1.0, 0.1),        // LITERAL-PX-OK: dado do objeto fictício
    (IMG_ROWS, 2.0, 1.0, 0.1),        // LITERAL-PX-OK: dado do objeto fictício
    (IMG_FPS, 12.0, 1.0, 0.1),        // LITERAL-PX-OK: dado do objeto fictício
    (RB_MASS, 1.0, 0.1, 0.01),        // LITERAL-PX-OK: dado do objeto fictício
    (RB_GRAVITY, 1.0, 0.1, 0.01),     // LITERAL-PX-OK: dado do objeto fictício
    (RB_LINEAR, 0.0, 0.1, 0.01),      // LITERAL-PX-OK: dado do objeto fictício
    (RB_ANGULAR, 0.0, 0.1, 0.01),     // LITERAL-PX-OK: dado do objeto fictício
    (COL_W, 32.0, 1.0, 1.0),          // LITERAL-PX-OK: dado do objeto fictício
    (COL_H, 48.0, 1.0, 1.0),          // LITERAL-PX-OK: dado do objeto fictício
    (COL_RADIUS, 16.0, 1.0, 1.0),     // LITERAL-PX-OK: dado do objeto fictício
    (COL_HALF, 24.0, 1.0, 1.0),       // LITERAL-PX-OK: dado do objeto fictício
    (COL_OX, 0.0, 1.0, 1.0),          // LITERAL-PX-OK: dado do objeto fictício
    (COL_OY, 0.0, 1.0, 1.0),          // LITERAL-PX-OK: dado do objeto fictício
    (COL_FRICTION, 0.5, 0.1, 0.01),   // LITERAL-PX-OK: dado do objeto fictício
    (COL_BOUNCE, 0.0, 0.1, 0.01),     // LITERAL-PX-OK: dado do objeto fictício
    (HP_MAX, 100.0, 1.0, 1.0),        // LITERAL-PX-OK: dado do objeto fictício
    (HP_START, 100.0, 1.0, 1.0),      // LITERAL-PX-OK: dado do objeto fictício
    (HP_REGEN, 0.0, 1.0, 0.1),        // LITERAL-PX-OK: dado do objeto fictício
    (HP_DELAY, 2.0, 0.1, 0.01),       // LITERAL-PX-OK: dado do objeto fictício
    (PL_SPEED, 8.0, 0.1, 0.01),       // LITERAL-PX-OK: dado do objeto fictício
    (PL_ACCEL, 8.0, 0.1, 0.01),       // LITERAL-PX-OK: dado do objeto fictício
    (PL_AIR, 4.0, 0.1, 0.01),         // LITERAL-PX-OK: dado do objeto fictício
    (PL_JUMP, 2.5, 0.1, 0.01),        // LITERAL-PX-OK: dado do objeto fictício
    (PL_AIR_JUMPS, 0.0, 1.0, 0.1),    // LITERAL-PX-OK: dado do objeto fictício
    (PL_COYOTE, 0.1, 0.01, 0.001),    // LITERAL-PX-OK: dado do objeto fictício
    (PL_BUFFER, 0.1, 0.01, 0.001),    // LITERAL-PX-OK: dado do objeto fictício
    (CAM_DEAD_W, 64.0, 1.0, 1.0),     // LITERAL-PX-OK: dado do objeto fictício
    (CAM_DEAD_H, 48.0, 1.0, 1.0),     // LITERAL-PX-OK: dado do objeto fictício
    (CAM_DAMPING, 0.2, 0.01, 0.001),  // LITERAL-PX-OK: dado do objeto fictício
    (CAM_LOOKAHEAD, 0.0, 1.0, 1.0),   // LITERAL-PX-OK: dado do objeto fictício
    (ANIM_FPS, 12.0, 1.0, 0.1),       // LITERAL-PX-OK: dado do objeto fictício
    (BR_SIZE, 40.0, 1.0, 1.0),        // LITERAL-PX-OK: dado do objeto fictício
    (BR_DILUTION, 0.2, 0.01, 0.001),  // LITERAL-PX-OK: dado do objeto fictício
    (BR_VOLUME, 100.0, 1.0, 1.0),     // LITERAL-PX-OK: dado do objeto fictício
    (BR_PRESS, 0.5, 0.01, 0.001),     // LITERAL-PX-OK: dado do objeto fictício
    (BR_REWET, 0.5, 0.01, 0.001),     // LITERAL-PX-OK: dado do objeto fictício
    (BR_BODY, 0.5, 0.01, 0.001),      // LITERAL-PX-OK: dado do objeto fictício
    (BO_LENGTH, 120.0, 1.0, 1.0),     // LITERAL-PX-OK: dado do objeto fictício
    (BO_MIN, -45.0, 1.0, 1.0),        // LITERAL-PX-OK: dado do objeto fictício
    (BO_MAX, 45.0, 1.0, 1.0),         // LITERAL-PX-OK: dado do objeto fictício
    (BO_MIX, 1.0, 0.01, 0.001),       // LITERAL-PX-OK: dado do objeto fictício
    (BO_CHAIN, 2.0, 1.0, 0.1),        // LITERAL-PX-OK: dado do objeto fictício
    (CS_INFLUENCE, 100.0, 1.0, 1.0),  // LITERAL-PX-OK: dado do objeto fictício
    (LT_INTENSITY, 1.0, 0.1, 0.01),   // LITERAL-PX-OK: dado do objeto fictício
    (LT_SIZE, 0.5, 0.1, 0.01),        // LITERAL-PX-OK: dado do objeto fictício
    (LT_RANGE, 8.0, 0.1, 0.01),       // LITERAL-PX-OK: dado do objeto fictício
    (AU_GAIN, 0.0, 1.0, 0.1),         // LITERAL-PX-OK: dado do objeto fictício
    (RACK[0].mix, 30.0, 1.0, 1.0),    // LITERAL-PX-OK: dado do objeto fictício
    (RACK[0].damp, 0.5, 0.01, 0.001), // LITERAL-PX-OK: dado do objeto fictício
    (RACK[1].mix, 30.0, 1.0, 1.0),    // LITERAL-PX-OK: dado do objeto fictício
    (RACK[1].damp, 0.5, 0.01, 0.001), // LITERAL-PX-OK: dado do objeto fictício
    (RACK[2].mix, 30.0, 1.0, 1.0),    // LITERAL-PX-OK: dado do objeto fictício
    (RACK[2].damp, 0.5, 0.01, 0.001), // LITERAL-PX-OK: dado do objeto fictício
    (PX_GRID_W, 16.0, 1.0, 0.1),      // LITERAL-PX-OK: dado do objeto fictício
    (PX_GRID_H, 16.0, 1.0, 0.1),      // LITERAL-PX-OK: dado do objeto fictício
    (SC_SPEED, 4.0, 0.1, 0.01),       // LITERAL-PX-OK: dado do objeto fictício
    (SC_JUMP, 6.0, 0.1, 0.01),        // LITERAL-PX-OK: dado do objeto fictício
    (SC_PATROL, 200.0, 1.0, 1.0),     // LITERAL-PX-OK: dado do objeto fictício
    (SC_DAMAGE, 10.0, 1.0, 0.1),      // LITERAL-PX-OK: dado do objeto fictício
    (SC_VALUE, 1.0, 1.0, 0.1),        // LITERAL-PX-OK: dado do objeto fictício
];
