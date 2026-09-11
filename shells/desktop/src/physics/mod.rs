//! **A família `physics` dentro da shell** — o que dela PRECISA da `App`.
//!
//! ⚠️ **Esta pasta é o que SOBRA, e é por isso que ela é pequena.** A W2/L2 tirou
//! 108 ficheiros (22,1 k linhas) desta família para
//! [`ph2d_app_physics`](../../../../crates/ph2d-app-physics/src/lib.rs): tudo o que
//! só sabia povoar um `World`. O que ficou aqui ficou por uma razão **nomeada**,
//! não por inércia — cada ficheiro precisa de algo que é genuinamente da shell:
//!
//! - o **gesto do ponteiro** (`joint_draw`, `joint_anchor_drag`, `body_*`,
//!   `player_input`) — eles lêem `last_pointer`, `modifiers`, `input_actions`;
//! - a **timeline** (`physics_smoke_player`, `physics_smoke_joint_anim`) — elas
//!   autoram tracks, e o `TimelineDoc` é de outro módulo;
//! - o **playhead** (`physics_smoke_rigs`) e o **readout** (`physics_smoke_out`);
//! - o **inspector da roldana** (`physics_smoke_pulley_*`, `physics_smoke_part`) —
//!   os gates deles dirigem `render_loop::inspector_joint_wheel`;
//! - e o **roteador** (`physics_smoke`), que é quem segura a `App`.
//!
//! ⭐ **Agrupá-los numa pasta torna o corte da Fase B o movimento de UMA pasta**, em
//! vez da reconciliação de 19 linhas dispersas num `main.rs` que outras cinco
//! linhas estão a editar ao mesmo tempo.

pub(crate) mod body_fk;
pub(crate) mod body_grab;
pub(crate) mod body_pose;
pub(crate) mod joint_anchor_drag;
pub(crate) mod joint_draw;
pub(crate) mod joint_rig;
pub(crate) mod joint_rig_drag;
pub(crate) mod physics_smoke;
pub(crate) mod physics_smoke_joint_anim;
pub(crate) mod physics_smoke_out;
pub(crate) mod physics_smoke_part;
pub(crate) mod physics_smoke_player;
pub(crate) mod physics_smoke_pulley_comp;
pub(crate) mod physics_smoke_pulley_diff;
pub(crate) mod physics_smoke_pulley_tackle;
pub(crate) mod physics_smoke_rig;
pub(crate) mod physics_smoke_rigs;
pub(crate) mod physics_state;
pub(crate) mod player_input;
