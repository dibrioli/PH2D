//! **As partes do painel do Platform Player** — quais rows moram em cada cartão e dobra, pela ordem da tela.
//! Irmão de `player_cartoes.rs` pelo tecto de LOC.

use super::*;

/// As rows de cada parte, por id: a ordem é a da tela.
pub(super) const WALK: [NodeId; 2] = [ids::INSP_PLAYER_SPEED, ids::INSP_PLAYER_ACCEL];
pub(super) const WALK_MORE: [NodeId; 3] = [
    ids::INSP_PLAYER_AIR_ACCEL,
    ids::INSP_PLAYER_BRAKE,
    ids::INSP_PLAYER_MAX_SLOPE,
];
pub(super) const JUMP: [NodeId; 2] = [ids::INSP_PLAYER_JUMP_HEIGHT, ids::INSP_PLAYER_AIR_JUMPS];
pub(super) const JUMP_MORE: [NodeId; 7] = [
    ids::INSP_PLAYER_AIR_JUMP_H,
    ids::INSP_PLAYER_TAKEOFF_G,
    ids::INSP_PLAYER_TAKEOFF_SPEED,
    ids::INSP_PLAYER_PEAK_G,
    ids::INSP_PLAYER_PEAK_SPEED,
    ids::INSP_PLAYER_FALL_G,
    ids::INSP_PLAYER_CUT_G,
];
pub(super) const WALLS: [NodeId; 5] = [
    ids::INSP_PLAYER_WALL_SLIDE,
    ids::INSP_PLAYER_WALL_JUMP,
    ids::INSP_PLAYER_WALL_PUSH,
    ids::INSP_PLAYER_WALL_LOCK,
    ids::INSP_PLAYER_WALL_GRAB,
];
pub(super) const WALL_SENSOR: [NodeId; 3] = [
    ids::INSP_PLAYER_WALL_REACH,
    ids::INSP_PLAYER_WALL_SAMPLES,
    ids::INSP_PLAYER_WALL_SPREAD,
];
pub(super) const DASH: [NodeId; 3] = [
    ids::INSP_PLAYER_DASH_SPEED,
    ids::INSP_PLAYER_DASH_TIME,
    ids::INSP_PLAYER_DASH_COOL,
];
pub(super) const CROUCH: [NodeId; 2] = [
    ids::INSP_PLAYER_CROUCH_HEIGHT,
    ids::INSP_PLAYER_CROUCH_SPEED,
];
pub(super) const SWIM: [NodeId; 3] = [
    ids::INSP_PLAYER_SWIM_SPEED,
    ids::INSP_PLAYER_SWIM_ACCEL,
    ids::INSP_PLAYER_SWIM_ENTER,
];
pub(super) const LEDGE: [NodeId; 5] = [
    ids::INSP_PLAYER_LEDGE_GRAB,
    ids::INSP_PLAYER_LEDGE_REACH_Y,
    ids::INSP_PLAYER_LEDGE_SPAN,
    ids::INSP_PLAYER_LEDGE_OFFSET_Y,
    ids::INSP_PLAYER_LEDGE_SPEED,
];
pub(super) const GLIDE: [NodeId; 1] = [ids::INSP_PLAYER_GLIDE_FALL];
pub(super) const FORGIVE: [NodeId; 6] = [
    ids::INSP_PLAYER_COYOTE,
    ids::INSP_PLAYER_BUFFER,
    ids::INSP_PLAYER_CORNER,
    ids::INSP_PLAYER_CORNER_SAMPLES,
    ids::INSP_PLAYER_CORNER_AHEAD,
    ids::INSP_PLAYER_LIFT,
];
pub(super) const LEG: [NodeId; 6] = [
    ids::INSP_PLAYER_FLOAT,
    ids::INSP_PLAYER_CLING,
    ids::INSP_PLAYER_STIFFNESS,
    ids::INSP_PLAYER_DAMPING,
    ids::INSP_PLAYER_FOOT_SAMPLES,
    ids::INSP_PLAYER_FOOT_SPREAD,
];
pub(super) const REACT: [NodeId; 3] = [
    ids::INSP_PLAYER_REACT_SUPPORT,
    ids::INSP_PLAYER_REACT_MOVEMENT,
    ids::INSP_PLAYER_REACT_PUSH,
];
pub(super) const FALL: [NodeId; 1] = [ids::INSP_PLAYER_MAX_FALL];

/// Todas as rows que este pintor põe na tela, para o teste que prova que nenhuma da tabela sumiu.
#[cfg(test)]
const PINTADAS: [&[NodeId]; 15] = [
    &WALK,
    &WALK_MORE,
    &JUMP,
    &JUMP_MORE,
    &WALLS,
    &WALL_SENSOR,
    &DASH,
    &CROUCH,
    &SWIM,
    &LEDGE,
    &GLIDE,
    &FORGIVE,
    &LEG,
    &REACT,
    &FALL,
];

#[cfg(test)]
#[test]
fn cada_row_da_tabela_tem_um_e_um_so_lugar_no_painel() {
    let pintadas: Vec<NodeId> = PINTADAS.iter().flat_map(|l| l.iter().copied()).collect();
    for (_, _, rows) in super::super::table::PLAYER_CARDS {
        for r in rows {
            let n = pintadas.iter().filter(|&&id| id == r.1).count();
            assert_eq!(n, 1, "{} aparece {n} vezes", r.0.tr());
        }
    }
    assert_eq!(pintadas.len(), super::super::player_row_count());
}
