//! **§14 — os ids do bloco *Animations*** (GameEngineFuncional N1): que animação toca em cada estado do
//! personagem (`CharacterAnimations`).

use ph2d_a11y::NodeId;
use ph2d_tool_registry::hash_node_id;

/// O interruptor que liga as animações por estado (anexa / remove o componente).
pub const INSP_PLAYER_ANIM: NodeId = hash_node_id("insp_player_anim");

/// As opções do interruptor: `0` = Off, `1` = On.
pub const INSP_PLAYER_ANIM_IDS: [NodeId; 2] = [
    hash_node_id("insp_player_anim_off"),
    hash_node_id("insp_player_anim_on"),
];

/// O botão do estado PARADO — cada clique passa à animação seguinte da lista.
pub const INSP_PLAYER_ANIM_IDLE: NodeId = hash_node_id("insp_player_anim_idle");
/// O botão do estado A CORRER.
pub const INSP_PLAYER_ANIM_RUN: NodeId = hash_node_id("insp_player_anim_run");
/// O botão do estado A SUBIR num salto.
pub const INSP_PLAYER_ANIM_JUMP: NodeId = hash_node_id("insp_player_anim_jump");
/// O botão do estado A CAIR.
pub const INSP_PLAYER_ANIM_FALL: NodeId = hash_node_id("insp_player_anim_fall");

/// Os quatro botões, pela ordem do `CharacterAnimations` — o índice é o de `PlayerFieldEdit::AnimNext`.
pub const INSP_PLAYER_ANIM_STATE_IDS: [NodeId; 4] = [
    INSP_PLAYER_ANIM_IDLE,
    INSP_PLAYER_ANIM_RUN,
    INSP_PLAYER_ANIM_JUMP,
    INSP_PLAYER_ANIM_FALL,
];

/// A partir de que velocidade andar é correr (m/s).
pub const INSP_PLAYER_ANIM_RUN_FROM: NodeId = hash_node_id("insp_player_anim_run_from");

/// O espelho pelo lado para que ele olha.
pub const INSP_PLAYER_ANIM_FLIP: NodeId = hash_node_id("insp_player_anim_flip");

/// As opções do espelho: `0` = Off, `1` = On.
pub const INSP_PLAYER_ANIM_FLIP_IDS: [NodeId; 2] = [
    hash_node_id("insp_player_anim_flip_off"),
    hash_node_id("insp_player_anim_flip_on"),
];
